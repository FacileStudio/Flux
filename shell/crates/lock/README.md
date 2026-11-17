# flux-lock

The Flux lock screen. GTK 4 on the Wayland `ext-session-lock-v1` protocol through
`gtk4-session-lock` (the session-lock half of gtk4-layer-shell 1.1+), with PAM for passwords.

```text
flux-lock                      lock now, stay in the foreground until unlocked
flux-lock --daemon             lock, return once the compositor confirms the lock
flux-lock --suspend-then-lock  lock, then run systemctl suspend once the lock is confirmed
```

`flux-idle` (crate `settings-power`) runs swayidle with `flux-lock --daemon` on the idle
timeout, on logind's `Lock` signal (`loginctl lock-session`) and on `before-sleep`. swayidle
runs with `-w`, so sleep waits until the lock is up.

## Security model

Threat: someone at the keyboard of an unattended, locked session.

- **The compositor enforces the lock.** With `ext-session-lock-v1` the compositor shows only
  lock surfaces until the locker asks to unlock. If flux-lock crashes, is killed or hangs, the
  outputs stay locked; it never falls back to an unlocked desktop.
- **One unlock path.** `Instance::unlock` is called in one place, on `Verdict::Unlock`, which
  `auth::verify` returns only when `pam_authenticate` returns `PAM_SUCCESS`. Wrong passwords,
  PAM errors, a failed conversation, a panicking worker: all keep the session locked.
- **The conversation fails closed.** It answers one password prompt. A second one (expired
  password, one-time code) or an unknown message style fails the conversation, so PAM fails.
  Empty passwords and passwords holding a NUL byte never reach PAM, and `pam_authenticate` runs
  with `PAM_DISALLOW_NULL_AUTHTOK`.
- **No bypass.** No flag, environment variable or debug build skips authentication. The
  screenshot preview is a separate example binary (`examples/lock-preview.rs`) that never
  locks and is not installed.
- **Signals.** SIGTERM, SIGINT and SIGHUP are ignored: the session stays locked. SIGKILL ends
  the process; the compositor keeps the outputs locked, and a new `flux-lock` (from a TTY with
  `WAYLAND_DISPLAY` set) can take the lock over if the compositor allows it.
- **Rate limiting is PAM's.** `pam_unix` delays failures and `pam_faillock` (when configured)
  locks accounts. flux-lock adds no counter of its own that a restart could reset.
- **The password is wiped.** The text moves from the `GtkPasswordEntry` (whose buffer GTK
  erases on clear) into a `Zeroizing` buffer; the `GString` copy is overwritten before it is
  freed; the buffer is dropped, and zeroed, as soon as PAM returns. The PAM reply is a `malloc`
  copy that PAM wipes and frees. The password is never logged.
- **Keys typed during a check are held, not dropped.** PAM takes about two seconds to refuse a
  password. Keys pressed meanwhile wait in a fixed-size buffer that never reallocates
  (`keys::KeyBuffer`). A refusal puts them into the field; an unlock wipes them unread. Enter
  is never held, so nothing is submitted without a key press after the answer.
- **Single instance.** An `flock` on `$XDG_RUNTIME_DIR/flux-lock.lock`, released by the kernel
  when the process ends. A second `flux-lock` returns at once (and reports "locked" to a
  `--daemon` parent). Another locker holding the session makes the compositor refuse the lock,
  and flux-lock exits 1.
- **Authentication runs off the GTK thread** (`gio::spawn_blocking`), so PAM's failure delay
  never freezes the clock or the input.

The PAM service is `flux-lock` (`security.pam.services.flux-lock` in `modules/power.nix`).

## What it shows

One surface per output, created on the library's `monitor` signal (hotplug included): the
blurred wallpaper (`~/.config/flux/wallpaper`, else the Flux default) under a muse scrim, the
clock and date, then, on any key or click, the user's name and the password field. The hint
and the field share one slot, so the clock does not move. Caps Lock is announced under the
field, with what the last attempt said. The niri keyboard layout and the battery sit in the
footer. Light and dark follow the shell palette. The content fades in over 160 ms and the field
fades in place of the hint; both are instant when GTK animations are off.

## Preview

```sh
FLUX_LOCK_PREVIEW=fr:wrong cargo run -p lock --example lock-preview
```

States: `idle`, `typing`, `wrong`, `unavailable`, `caps`.
