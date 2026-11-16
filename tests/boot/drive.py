#!/usr/bin/env python3
"""Boot the Flux ISO in QEMU, install it to a blank disk, boot the result and screenshot every screen.

Standard library only. Run `drive.py --help` for the options. Phases, in order:

  live-grub, live-plymouth, live-desktop, live-terminal, install,
  target-loader, target-plymouth, target-greeter, target-login, target-verify

Readiness signals. The ISO cannot be changed for the test, so the driver attaches a second,
read-only virtio disk (serial `fluxscript`) that holds a bash script, opens a terminal with QMP
key presses (niri: Mod+Return, then Mod+D and `ghostty` on even retries) and types
`sudo bash /dev/disk/by-id/virtio-fluxscript live`. The script writes FLUX-* markers to
/dev/ttyS0, which QEMU logs with `-serial file:`. Those markers are the deterministic signals:
the shell is up, the install progress lines (tee'd from `--progress json`), the exit code,
the failed units. The installed system runs the same script in `verify` mode after login.

Screens without a marker (GRUB, Plymouth, greeter, desktop) are found from QMP screendumps:
GRUB is the first still screen with content across it (arrow keys are sent until then so its timeout
stops); Plymouth is one flat colour with content in a small area; a session is a screen that
stays still for 10 to 12 seconds. Every distinct frame is also kept under frames/.

GPU. niri refuses software EGL renderers, so on a plain virtio-vga the session stays black.
The default `--gpu virgl` gives the guest virtio-vga-gl with `-display egl-headless`, which
needs a host render node. When the session never shows, the run records a failure and falls
back to the text console (Ctrl+Alt+F2) so the install is still exercised.

A phase whose signal never arrives fails the run with a message naming it. Heuristic
screenshots that cannot be matched become notices; `--strict` turns notices into failures.
"""

import argparse
import collections
import json
import os
import shutil
import signal
import socket
import struct
import subprocess
import sys
import tempfile
import time
import zlib
from pathlib import Path

PASSWORD = "test"
PASSWORD_HASH = "$6$fluxboottest$3WsXJsBwlwupIYcUpR9Biw1D0tApOeKymQbgJnD357iGK3I2xkUgCHaAKZi0zUpnckq3UVAANShYPuULhrD/Q."
SCRIPT_SERIAL = "fluxscript"
SCRIPT_DEVICE = f"/dev/disk/by-id/virtio-{SCRIPT_SERIAL}"
STEPS = {"partition", "format", "config", "install", "bootloader", "done", "error"}

OVMF_CANDIDATES = [
    ("/usr/share/OVMF/OVMF_CODE_4M.fd", "/usr/share/OVMF/OVMF_VARS_4M.fd"),
    ("/usr/share/OVMF/OVMF_CODE.fd", "/usr/share/OVMF/OVMF_VARS.fd"),
    ("/usr/share/edk2/ovmf/OVMF_CODE.fd", "/usr/share/edk2/ovmf/OVMF_VARS.fd"),
    ("/usr/share/edk2/x64/OVMF_CODE.4m.fd", "/usr/share/edk2/x64/OVMF_VARS.4m.fd"),
    ("/run/libvirt/nix-ovmf/OVMF_CODE.fd", "/run/libvirt/nix-ovmf/OVMF_VARS.fd"),
]

GUEST_SCRIPT = r"""#!/usr/bin/env bash
set -u
say() { printf '%s\n' "$*" > /dev/ttyS0; }
mode=${1:-live}
say "FLUX-TEST-START mode=$mode"
say "FLUX-TEST-INFO os=$(. /etc/os-release; printf '%s' "$PRETTY_NAME") host=$(hostname) kernel=$(uname -r)"
say "FLUX-TEST-INFO cmdline=$(cat /proc/cmdline)"
failed=$(systemctl --failed --no-legend --plain 2>/dev/null | awk '{print $1}' | tr '\n' ' ')
say "FLUX-TEST-FAILED-UNITS ${failed:-none}"
case "$mode" in
  live)
    installer=$(command -v flux-install || true)
    if [ -z "$installer" ]; then
      say "FLUX-INSTALL-UNSUPPORTED flux-install is not on PATH"
    elif ! grep -q -- '--progress' "$installer"; then
      say "FLUX-INSTALL-UNSUPPORTED flux-install has no --progress json mode yet"
    elif [ "@INSTALL@" != yes ]; then
      say "FLUX-INSTALL-SKIPPED driver ran with --no-install"
    else
      flux-install --disk /dev/vda --hostname flux-test --full-name "Flux Test" --user test \
        --password-hash '@HASH@' --timezone Europe/Paris --keymap us --locale en_US.UTF-8 \
        --yes --progress json 2>/tmp/flux-install.log | while IFS= read -r line; do
          printf '%s\n' "$line"
          say "FLUX-PROGRESS $line"
        done
      rc=${PIPESTATUS[0]}
      tail -n 200 /tmp/flux-install.log | sed 's/^/FLUX-LOG /' > /dev/ttyS0
      say "FLUX-INSTALL-EXIT $rc"
    fi
    ;;
  verify)
    say "FLUX-TEST-INFO user=$(id test 2>&1)"
    say "FLUX-TEST-INFO session=$(pgrep -x niri >/dev/null && echo niri-running || echo niri-missing)"
    say "FLUX-TEST-INFO loader=$( [ -d /boot/grub ] && echo grub || echo other) efi=$(ls /boot/EFI 2>/dev/null | tr '\n' ' ')"
    say "FLUX-VERIFY-DONE"
    ;;
esac
say "FLUX-TEST-END mode=$mode"
sleep 6
systemctl poweroff
exit 0
"""

QCODES = {
    " ": "spc", "\n": "ret", "\t": "tab", "-": "minus", "=": "equal", "[": "bracket_left",
    "]": "bracket_right", "\\": "backslash", ";": "semicolon", "'": "apostrophe",
    "`": "grave_accent", ",": "comma", ".": "dot", "/": "slash",
}
SHIFTED = {
    "_": "minus", "+": "equal", "{": "bracket_left", "}": "bracket_right", "|": "backslash",
    ":": "semicolon", '"': "apostrophe", "~": "grave_accent", "<": "comma", ">": "dot",
    "?": "slash", "!": "1", "@": "2", "#": "3", "$": "4", "%": "5", "^": "6", "&": "7",
    "*": "8", "(": "9", ")": "0",
}


class PhaseError(Exception):
    """A phase did not reach its signal; the message names the phase."""

    def __init__(self, phase, message):
        super().__init__(f"phase {phase} failed: {message}")
        self.phase = phase


class Frame:
    """A sampled screendump: full RGB bytes plus a coarse grid for comparisons."""

    STEP = 8

    def __init__(self, width, height, rgb):
        self.width, self.height, self.rgb = width, height, rgb
        stride = width * 3
        rows = []
        for y in range(0, height, self.STEP):
            row = rgb[y * stride:(y + 1) * stride]
            r, g, b = row[0::3 * self.STEP], row[1::3 * self.STEP], row[2::3 * self.STEP]
            rows.extend(zip(r, g, b))
        self.samples = rows

    @classmethod
    def from_ppm(cls, path):
        data = Path(path).read_bytes()
        fields, pos = [], 0
        while len(fields) < 4:
            while data[pos:pos + 1].isspace():
                pos += 1
            if data[pos:pos + 1] == b"#":
                pos = data.index(b"\n", pos)
                continue
            end = pos
            while not data[end:end + 1].isspace():
                end += 1
            fields.append(data[pos:end])
            pos = end
        if fields[0] != b"P6" or fields[3] != b"255":
            raise ValueError(f"unsupported screendump format in {path}")
        width, height = int(fields[1]), int(fields[2])
        return cls(width, height, data[pos + 1:pos + 1 + width * height * 3])

    def dominant(self):
        counts = collections.Counter((r >> 4, g >> 4, b >> 4) for r, g, b in self.samples)
        return counts.most_common(1)[0][1] / len(self.samples)

    def lit(self):
        return sum(1 for p in self.samples if max(p) > 48) / len(self.samples)

    def spread(self):
        """Fraction of the screen covered by the bounding box of the pixels off the dominant color."""
        counts = collections.Counter((r >> 4, g >> 4, b >> 4) for r, g, b in self.samples)
        main = counts.most_common(1)[0][0]
        cols = len(range(0, self.width, self.STEP))
        xs, ys = [], []
        for i, (r, g, b) in enumerate(self.samples):
            if (r >> 4, g >> 4, b >> 4) != main:
                xs.append(i % cols)
                ys.append(i // cols)
        if not xs:
            return 0.0
        rows = len(self.samples) / cols
        return (max(xs) - min(xs) + 1) * (max(ys) - min(ys) + 1) / (cols * rows)

    def uniform(self):
        return self.dominant() > 0.995

    def diff(self, other):
        if other is None or (self.width, self.height) != (other.width, other.height):
            return 1.0
        changed = sum(
            1 for a, b in zip(self.samples, other.samples)
            if abs(a[0] - b[0]) > 24 or abs(a[1] - b[1]) > 24 or abs(a[2] - b[2]) > 24
        )
        return changed / len(self.samples)

    def write_png(self, path):
        stride = self.width * 3
        raw = b"".join(b"\x00" + self.rgb[y * stride:(y + 1) * stride] for y in range(self.height))

        def chunk(kind, body):
            return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body))

        header = struct.pack(">IIBBBBB", self.width, self.height, 8, 2, 0, 0, 0)
        Path(path).write_bytes(
            b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(raw, 6)) + chunk(b"IEND", b"")
        )


class QMP:
    """Minimal QMP client over a unix socket."""

    def __init__(self, path, alive, timeout=30):
        deadline = time.monotonic() + timeout
        while True:
            try:
                self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                self.sock.connect(path)
                break
            except OSError:
                self.sock.close()
                if not alive() or time.monotonic() > deadline:
                    raise ConnectionError(f"cannot reach QMP at {path}")
                time.sleep(0.2)
        self.sock.settimeout(60)
        self.buf = b""
        self.events = []
        self._read()
        self.command("qmp_capabilities")

    def _read(self):
        while b"\n" not in self.buf:
            chunk = self.sock.recv(65536)
            if not chunk:
                raise ConnectionError("QMP socket closed")
            self.buf += chunk
        line, self.buf = self.buf.split(b"\n", 1)
        return json.loads(line)

    def command(self, name, **arguments):
        message = {"execute": name}
        if arguments:
            message["arguments"] = arguments
        self.sock.sendall(json.dumps(message).encode() + b"\n")
        while True:
            reply = self._read()
            if "event" in reply:
                self.events.append(reply)
                continue
            if "error" in reply:
                raise RuntimeError(f"QMP {name}: {reply['error'].get('desc')}")
            return reply.get("return")

    def close(self):
        try:
            self.sock.close()
        except OSError:
            pass


class Machine:
    """One QEMU process with its QMP socket, serial log and screenshot helpers."""

    def __init__(self, run, name, drives):
        self.run, self.name = run, name
        self.qmp_path = f"{name}.qmp"
        self.serial_path = run.out / f"serial-{name}.log"
        self.serial_pos = 0
        self.serial_lines = []
        self.partial = ""
        self.last_saved = None
        self.frame_index = 0
        args = run.qemu_base() + drives + [
            "-qmp", f"unix:{self.qmp_path},server=on,wait=off",
            "-serial", f"file:{self.serial_path}",
            "-S",
        ]
        log = open(run.out / f"qemu-{name}.log", "w")
        log.write(" ".join(args) + "\n\n")
        log.flush()
        self.proc = subprocess.Popen(args, cwd=run.work, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        run.log(f"[{name}] qemu pid {self.proc.pid}")
        self.qmp = QMP(self.qmp_path, self.alive)
        self.qmp.command("cont")
        self.started = time.monotonic()

    def alive(self):
        return self.proc.poll() is None

    def elapsed(self):
        return time.monotonic() - self.started

    def grab(self):
        if not self.alive():
            return None
        ppm = self.run.work / f"{self.name}.ppm"
        try:
            self.qmp.command("screendump", filename=str(ppm))
            return Frame.from_ppm(ppm)
        except (RuntimeError, OSError, ValueError, ConnectionError):
            return None

    def tick(self, phase):
        frame = self.grab()
        if frame is not None and frame.diff(self.last_saved) > 0.01:
            self.frame_index += 1
            frames = self.run.out / "frames"
            frames.mkdir(exist_ok=True)
            frame.write_png(frames / f"{self.name}-{self.frame_index:03d}-{phase}-{int(self.elapsed()):04d}s.png")
            self.last_saved = frame
        return frame

    def shot(self, name, frame=None):
        frame = frame or self.grab()
        if frame is None:
            return None
        path = self.run.out / f"{name}.png"
        frame.write_png(path)
        self.run.shots.append(path.name)
        self.run.log(f"[{self.name}] screenshot {path.name}")
        return path

    def serial(self):
        try:
            with open(self.serial_path, "rb") as handle:
                handle.seek(self.serial_pos)
                data = handle.read()
        except OSError:
            return []
        self.serial_pos += len(data)
        text = self.partial + data.decode("utf-8", "replace").replace("\r", "")
        *lines, self.partial = text.split("\n")
        self.serial_lines.extend(lines)
        return lines

    def find(self, prefix):
        self.serial()
        return [line[line.index(prefix):] for line in self.serial_lines if prefix in line]

    def key(self, *names, hold=50):
        keys = [{"type": "qcode", "data": n} for n in names]
        self.qmp.command("send-key", keys=keys, **{"hold-time": hold})
        time.sleep(hold / 1000 + 0.04)

    def type(self, text):
        for char in text:
            if char.isalpha() and char.isupper():
                self.key("shift", char.lower())
            elif char.isalnum():
                self.key(char)
            elif char in QCODES:
                self.key(QCODES[char])
            elif char in SHIFTED:
                self.key("shift", SHIFTED[char])
            else:
                raise ValueError(f"cannot type {char!r}")

    def wait_exit(self, timeout):
        try:
            self.proc.wait(timeout)
            return True
        except subprocess.TimeoutExpired:
            return False

    def stop(self):
        if self.alive():
            try:
                self.qmp.command("quit")
            except (RuntimeError, OSError, ConnectionError):
                pass
            if not self.wait_exit(10):
                os.kill(self.proc.pid, signal.SIGKILL)
                self.proc.wait()
        self.qmp.close()
        self.serial()


class Run:
    """The whole test: options, output directory, phase bookkeeping and the summary."""

    def __init__(self, opts):
        self.opts = opts
        self.out = Path(opts.out).resolve()
        self.out.mkdir(parents=True, exist_ok=True)
        self.work = Path(tempfile.mkdtemp(prefix="flux-boot-", dir=opts.workdir)).resolve()
        self.iso = Path(opts.iso).resolve()
        self.ovmf_paths = tuple(str(Path(p).resolve()) if p else None for p in (opts.ovmf_code, opts.ovmf_vars))
        os.chdir(self.work)
        self.shots, self.notices, self.phases, self.failures = [], [], [], []
        self.machines = []
        self.logfile = open(self.out / "driver.log", "a")

    def log(self, message):
        line = f"{time.strftime('%H:%M:%S')} {message}"
        print(line, flush=True)
        self.logfile.write(line + "\n")
        self.logfile.flush()

    def notice(self, phase, message):
        self.notices.append({"phase": phase, "message": message})
        self.log(f"NOTICE [{phase}] {message}")
        if os.environ.get("GITHUB_ACTIONS"):
            print(f"::warning title=boot-test {phase}::{message}", flush=True)

    def phase(self, name, status, detail=""):
        self.phases.append({"phase": name, "status": status, "detail": detail})
        self.log(f"PHASE {name}: {status} {detail}".rstrip())

    def qemu_base(self):
        o = self.opts
        return [
            o.qemu, "-machine", "q35", "-accel", o.accel, "-cpu", "host" if o.accel == "kvm" else "max",
            "-m", str(o.memory), "-smp", str(o.smp),
            "-drive", f"if=pflash,format=raw,unit=0,readonly=on,file={self.ovmf_code}",
            "-drive", f"if=pflash,format=raw,unit=1,file={self.vars}",
            *self.display(),
            "-device", "qemu-xhci", "-device", "usb-tablet",
            "-nic", "user,model=virtio-net-pci",
            "-no-reboot",
        ]

    def display(self):
        if self.opts.gpu == "plain":
            return ["-vga", "none", "-device", "virtio-vga", "-display", "none"]
        node = f",rendernode={self.opts.rendernode}" if self.opts.rendernode else ""
        return ["-vga", "none", "-device", "virtio-vga-gl", "-display", f"egl-headless{node}"]

    def target_drives(self, boot_index):
        return [
            "-drive", f"file={self.target},if=none,id=target,format=qcow2,discard=unmap",
            "-device", f"virtio-blk-pci,drive=target,bus=pcie.0,addr=0x10,bootindex={boot_index}",
            "-drive", f"file={self.script},if=none,id=script,format=raw,readonly=on",
            "-device", f"virtio-blk-pci,drive=script,serial={SCRIPT_SERIAL},bus=pcie.0,addr=0x11",
        ]

    def prepare(self):
        o = self.opts
        if not self.iso.is_file():
            raise PhaseError("prepare", f"ISO not found: {self.iso}")
        if o.accel == "kvm" and not os.access("/dev/kvm", os.R_OK | os.W_OK):
            raise PhaseError("prepare", "/dev/kvm is missing or not writable; pass --accel tcg to run without KVM")
        if shutil.which(o.qemu) is None:
            raise PhaseError("prepare", f"{o.qemu} not found")
        code, vars_template = self.ovmf_paths
        if not code or not vars_template:
            for c, v in OVMF_CANDIDATES:
                if Path(c).is_file() and Path(v).is_file():
                    code, vars_template = c, v
                    break
        if not code or not Path(code).is_file() or not Path(vars_template).is_file():
            raise PhaseError("prepare", "OVMF firmware not found; pass --ovmf-code and --ovmf-vars")
        self.ovmf_code = Path(code).resolve()
        self.vars = self.work / "OVMF_VARS.fd"
        shutil.copyfile(vars_template, self.vars)
        os.chmod(self.vars, 0o644)
        self.target = self.work / "target.qcow2"
        subprocess.run(["qemu-img", "create", "-q", "-f", "qcow2", str(self.target), o.disk_size], check=True)
        script = GUEST_SCRIPT.replace("@HASH@", PASSWORD_HASH).replace("@INSTALL@", "no" if o.no_install else "yes")
        self.script = self.work / "script.img"
        self.script.write_bytes(script.encode().ljust(1024 * 1024, b"\x00"))
        self.log(f"iso {self.iso}")
        self.log(f"ovmf {self.ovmf_code} vars {vars_template}")
        self.log(f"work {self.work}")
        self.phase("prepare", "ok")

    def boot(self, name, drives):
        try:
            machine = Machine(self, name, drives)
        except ConnectionError:
            if self.opts.gpu != "virgl":
                raise PhaseError(name, f"QEMU did not start; see qemu-{name}.log")
            reason = (self.out / f"qemu-{name}.log").read_text().strip().splitlines()[-1]
            self.soft_fail("gpu", f"QEMU could not start the virgl display ({reason}); retrying with plain virtio-vga, where niri cannot render")
            self.opts.gpu = "plain"
            machine = Machine(self, name, drives)
        self.machines.append(machine)
        return machine

    def watch(self, machine, phase, timeout, until, interval=0.5):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if not machine.alive():
                raise PhaseError(phase, f"QEMU exited (code {machine.proc.returncode}); see qemu-{machine.name}.log")
            frame = machine.tick(phase)
            if frame is not None:
                result = until(frame)
                if result:
                    return result
            time.sleep(interval)
        return None

    def settle(self, machine, phase, timeout, quiet, not_like=None):
        """Wait until the screen stops changing for `quiet` seconds; return that frame or None."""
        state = {"ref": None, "since": time.monotonic()}

        def until(frame):
            now = time.monotonic()
            if state["ref"] is None or frame.diff(state["ref"]) > 0.005:
                state["ref"], state["since"] = frame, now
                return None
            if frame.uniform():
                state["since"] = now
                return None
            if not_like is not None and frame.diff(not_like) < 0.02:
                state["since"] = now
                return None
            return frame if now - state["since"] >= quiet else None

        return self.watch(machine, phase, timeout, until, interval=1.0)

    def live_grub(self, m):
        state = {"prev": None}

        def until(frame):
            m.key("up")
            themed = not frame.uniform() and (frame.lit() > 0.35 or frame.spread() > 0.2)
            stable = state["prev"] is not None and frame.diff(state["prev"]) < 0.005
            state["prev"] = frame
            return frame if themed and stable and m.elapsed() > 2 else None

        frame = self.watch(m, "live-grub", self.opts.grub_timeout, until)
        if frame is None:
            self.notice("live-grub", f"no GRUB menu (a still screen with content across it) within {self.opts.grub_timeout}s; saved the current screen instead")
            frame = m.grab()
            self.phase("live-grub", "warn", "menu not detected")
        else:
            self.phase("live-grub", "ok")
        m.shot("01-live-grub", frame)
        m.key("ret")
        return frame

    def plymouth(self, m, phase, after, timeout, shot_name, loader_shot=None):
        state = {"last": after, "candidate": None}

        def until(frame):
            if after is not None and frame.diff(after) < 0.02:
                state["last"] = frame
                return None
            looks = 0.80 < frame.dominant() < 0.995 and frame.spread() < 0.2
            if looks and state["candidate"] is not None and frame.diff(state["candidate"]) < 0.03:
                return frame
            if looks:
                state["candidate"] = frame
            else:
                state["candidate"] = None
                state["last"] = frame
            return None

        frame = self.watch(m, phase, timeout, until, interval=0.5)
        if loader_shot:
            m.shot(loader_shot, state["last"] or m.grab())
        if frame is None:
            self.notice(phase, f"no splash-like screen (one flat color, content only in a small area) within {timeout}s; check frames/ for the boot timeline")
            self.phase(phase, "warn", "splash not detected")
            return None
        m.shot(shot_name, frame)
        self.phase(phase, "ok", "heuristic: flat screen with a small mark")
        return frame

    def launch_terminal(self, m, mode, attempt, login):
        if mode == "vt":
            m.key("ctrl", "alt", "f2")
            time.sleep(4)
            if login:
                m.type(login + "\n")
                time.sleep(2)
                m.type(PASSWORD + "\n")
                time.sleep(6)
        elif attempt % 2 == 1:
            m.key("meta_l", "ret")
        else:
            m.key("meta_l", "d")
            time.sleep(2)
            m.type("ghostty\n")
        time.sleep(5 + attempt * 2)
        m.type("q")
        m.key("ctrl", "u")

    def open_terminal(self, m, phase, command, marker, mode, login=None, attempts=4):
        """Open a shell (niri: Mod+Return or fuzzel; vt: Ctrl+Alt+F2), type `command`, wait for `marker` on serial."""
        for attempt in range(1, attempts + 1):
            self.log(f"[{phase}] attempt {attempt}: open a {mode} terminal and type the test command")
            self.launch_terminal(m, mode, attempt, login if attempt == 1 or mode == "niri" else None)
            m.type(command + "\n")
            deadline = time.monotonic() + 30 + attempt * 15
            while time.monotonic() < deadline:
                if not m.alive():
                    raise PhaseError(phase, "QEMU exited while waiting for the terminal")
                m.tick(phase)
                if m.find(marker):
                    return attempt
                time.sleep(1)
            m.shot(f"{phase}-attempt-{attempt}")
        raise PhaseError(phase, f"no {marker} on the serial port after {attempts} attempts to open a {mode} terminal and run the test script")

    def soft_fail(self, phase, message):
        self.failures.append(f"phase {phase} failed: {message}")
        self.phase(phase, "fail", message)
        if os.environ.get("GITHUB_ACTIONS"):
            print(f"::error title=boot-test {phase}::{message}", flush=True)

    def live(self):
        o = self.opts
        m = self.boot("live", [
            "-drive", f"file={self.iso},if=none,id=cd,media=cdrom,readonly=on,format=raw",
            "-device", "ide-cd,drive=cd,bus=ide.0,bootindex=1",
        ] + self.target_drives(2))
        grub = self.live_grub(m)
        self.plymouth(m, "live-plymouth", grub, o.boot_timeout, "02-live-plymouth")
        desktop = self.settle(m, "live-desktop", o.session_timeout, quiet=12, not_like=grub)
        mode = o.terminal
        if desktop is None:
            m.shot("03-live-desktop")
            self.soft_fail("live-desktop", f"no graphical session within {o.session_timeout}s (screen blank or never still); continuing on a text console")
            mode = "vt"
        else:
            m.shot("03-live-desktop", desktop)
            self.phase("live-desktop", "ok", "screen stable for 12s; the installer welcome should be on it")
        attempt = self.open_terminal(m, "live-terminal", f"sudo bash {SCRIPT_DEVICE} live", "FLUX-TEST-START", mode)
        time.sleep(3)
        m.shot("04-live-terminal")
        self.phase("live-terminal", "ok", f"test script started on attempt {attempt} ({mode})")
        for line in m.find("FLUX-TEST-INFO"):
            self.log(f"[live] {line}")
        self.check_failed_units(m, "live-terminal")
        installed = self.install(m)
        if not m.wait_exit(180):
            m.shot("live-no-poweroff")
            raise PhaseError("install" if installed else "live-terminal", "the live system did not power off within 180s")
        m.stop()
        return installed

    def check_failed_units(self, m, phase):
        for line in m.find("FLUX-TEST-FAILED-UNITS"):
            units = line.split(" ", 1)[1].strip() if " " in line else ""
            if units and units != "none":
                self.notice(phase, f"failed systemd units: {units}")

    def install(self, m):
        o = self.opts
        deadline = time.monotonic() + o.install_timeout
        seen, last_periodic, step_shots, cursor = 0, time.monotonic(), set(), 0
        while time.monotonic() < deadline:
            alive = m.alive()
            m.serial()
            lines, cursor = m.serial_lines[cursor:], len(m.serial_lines)
            for line in lines:
                if "FLUX-INSTALL-UNSUPPORTED" in line or "FLUX-INSTALL-SKIPPED" in line:
                    reason = line.split(" ", 1)[1] if " " in line else line
                    self.notice("install", f"install not run: {reason}; target phases skipped")
                    self.phase("install", "skip", reason)
                    return False
                if "FLUX-PROGRESS " in line:
                    seen += 1
                    payload = line.split("FLUX-PROGRESS ", 1)[1]
                    try:
                        event = json.loads(payload)
                    except json.JSONDecodeError:
                        raise PhaseError("install", f"progress line is not JSON: {payload!r}")
                    step = event.get("step")
                    if step not in STEPS:
                        raise PhaseError("install", f"unknown progress step {step!r} in {payload!r}")
                    self.log(f"[install] {step} {event.get('percent', '')} {event.get('message', '')}")
                    if step == "error":
                        m.shot("05-install-error")
                        raise PhaseError("install", f"installer reported: {event.get('message')}")
                    if step not in step_shots:
                        step_shots.add(step)
                        m.shot(f"05-install-{step}")
                if "FLUX-INSTALL-EXIT " in line:
                    rc = line.split("FLUX-INSTALL-EXIT ", 1)[1].strip()
                    m.shot("06-install-exit")
                    if rc != "0":
                        raise PhaseError("install", f"flux-install exited with {rc}; see FLUX-LOG lines in serial-live.log")
                    if "done" not in step_shots:
                        raise PhaseError("install", "flux-install exited 0 without a done progress step")
                    self.phase("install", "ok", f"{seen} progress lines")
                    return True
            if not alive:
                raise PhaseError("install", "QEMU exited before the install finished")
            if time.monotonic() - last_periodic > 60:
                last_periodic = time.monotonic()
                m.tick("install")
            time.sleep(2)
        m.shot("06-install-timeout")
        raise PhaseError("install", f"no FLUX-INSTALL-EXIT within {o.install_timeout}s")

    def target(self):
        o = self.opts
        m = self.boot("target", self.target_drives(1))
        first = self.watch(m, "target-loader", 30, lambda f: f if not f.uniform() else None)
        self.plymouth(m, "target-plymouth", first, o.boot_timeout, "08-target-plymouth", loader_shot="07-target-loader")
        self.phase("target-loader", "ok", "07-target-loader.png is the last screen before the splash; it should show no menu")
        mode, login = o.terminal, None
        greeter = self.settle(m, "target-greeter", o.session_timeout, quiet=10)
        if greeter is None:
            m.shot("09-target-greeter")
            self.soft_fail("target-greeter", f"no greeter within {o.session_timeout}s (screen blank or never still); continuing on a text console")
            mode, login = "vt", "test"
        else:
            m.shot("09-target-greeter", greeter)
            self.phase("target-greeter", "ok", "screen stable for 10s")
        if mode == "niri":
            m.type(PASSWORD + "\n")
            desktop = self.settle(m, "target-login", 120, quiet=10, not_like=greeter)
            if desktop is None:
                m.shot("10-target-login-stuck")
                self.soft_fail("target-login", "the screen did not change after typing the password; continuing on a text console")
                mode, login = "vt", "test"
            else:
                m.shot("10-target-desktop", desktop)
                self.phase("target-login", "ok")
        else:
            self.phase("target-login", "skip", "logging in on a text console instead")
            login = "test"
        command = f"echo {PASSWORD} | sudo -S bash {SCRIPT_DEVICE} verify"
        attempt = self.open_terminal(m, "target-verify", command, "FLUX-VERIFY-DONE", mode, login)
        m.shot("11-target-terminal")
        for line in m.find("FLUX-TEST-INFO"):
            self.log(f"[target] {line}")
        self.check_failed_units(m, "target-verify")
        info = " ".join(m.find("FLUX-TEST-INFO"))
        if mode == "niri" and "niri-running" not in info:
            self.notice("target-verify", "niri was not running in the logged-in session")
        self.phase("target-verify", "ok", f"verify script ran on attempt {attempt} ({mode})")
        if not m.wait_exit(120):
            self.notice("target-verify", "the installed system did not power off within 120s")
        m.stop()

    def summary(self, error):
        status = "fail" if error else ("warn" if self.notices else "ok")
        data = {"status": status, "error": error, "phases": self.phases, "notices": self.notices, "screenshots": self.shots}
        (self.out / "summary.json").write_text(json.dumps(data, indent=2) + "\n")
        lines = [f"## Boot test: {status}", ""]
        if error:
            lines += [f"**{error}**", ""]
        lines += ["| Phase | Status | Detail |", "|---|---|---|"]
        lines += [f"| {p['phase']} | {p['status']} | {p['detail']} |" for p in self.phases]
        if self.notices:
            lines += ["", "### Notices", ""] + [f"- {n['phase']}: {n['message']}" for n in self.notices]
        lines += ["", "### Screenshots", ""] + [f"- {s}" for s in self.shots]
        text = "\n".join(lines) + "\n"
        (self.out / "summary.md").write_text(text)
        step_summary = os.environ.get("GITHUB_STEP_SUMMARY")
        if step_summary:
            with open(step_summary, "a") as handle:
                handle.write(text)
        return status

    def cleanup(self):
        for machine in self.machines:
            try:
                machine.stop()
            except Exception as exc:
                self.log(f"cleanup {machine.name}: {exc}")
        if self.opts.keep_disks:
            self.log(f"kept {self.work}")
        else:
            shutil.rmtree(self.work, ignore_errors=True)


def parse(argv):
    p = argparse.ArgumentParser(description="Boot, install and screenshot a Flux ISO in QEMU.")
    p.add_argument("--iso", required=True, help="the ISO to boot; opened read-only")
    p.add_argument("--out", default="boot-screens", help="screenshots, serial logs and summary go here")
    p.add_argument("--ovmf-code", help="OVMF_CODE.fd (read-only pflash); auto-detected when omitted")
    p.add_argument("--ovmf-vars", help="OVMF_VARS.fd template; copied, never written")
    p.add_argument("--qemu", default="qemu-system-x86_64")
    p.add_argument("--accel", default="kvm", choices=["kvm", "tcg"])
    p.add_argument("--gpu", default="virgl", choices=["virgl", "plain"],
                   help="virgl gives the guest a 3D GPU (niri refuses software renderers); plain is virtio-vga without 3D")
    p.add_argument("--rendernode", help="host DRM render node for egl-headless (default: QEMU picks one)")
    p.add_argument("--terminal", default="niri", choices=["niri", "vt"],
                   help="how to reach a shell: niri keybindings, or the text console on Ctrl+Alt+F2")
    p.add_argument("--memory", type=int, default=4096)
    p.add_argument("--smp", type=int, default=4)
    p.add_argument("--disk-size", default="30G")
    p.add_argument("--workdir", help="parent directory for temporary disks (default: system temp)")
    p.add_argument("--no-install", action="store_true", help="stop after the live session checks")
    p.add_argument("--strict", action="store_true", help="treat notices (missing theme, splash, install backend) as failures")
    p.add_argument("--keep-disks", action="store_true")
    p.add_argument("--grub-timeout", type=int, default=60)
    p.add_argument("--boot-timeout", type=int, default=90)
    p.add_argument("--session-timeout", type=int, default=300)
    p.add_argument("--install-timeout", type=int, default=5400)
    return p.parse_args(argv)


def main(argv):
    opts = parse(argv)
    run = Run(opts)
    error = None
    try:
        run.prepare()
        if run.live():
            run.target()
        else:
            for name in ("target-loader", "target-plymouth", "target-greeter", "target-login", "target-verify"):
                run.phase(name, "skip", "no installed disk")
    except PhaseError as exc:
        error = str(exc)
        run.phase(exc.phase, "fail", error)
    except KeyboardInterrupt:
        error = "interrupted"
    except Exception as exc:
        error = f"driver error: {exc!r}"
    finally:
        run.cleanup()
    if error is None and run.failures:
        error = "; ".join(run.failures)
    status = run.summary(error)
    if error:
        run.log(f"FAIL {error}")
        if os.environ.get("GITHUB_ACTIONS"):
            print(f"::error title=boot-test::{error}", flush=True)
        return 1
    if status == "warn" and opts.strict:
        run.log("FAIL strict mode: notices count as failures")
        return 1
    run.log(f"done: {status}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
