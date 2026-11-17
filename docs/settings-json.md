# settings.json

`/etc/flux/settings.json` holds the machine state that NixOS owns declaratively and that people change from Settings: device name, time zone, language, keyboard layout, accounts and automatic updates. `flux-config` writes it; `modules/settings.nix` reads it and turns it into NixOS options.

## How the file reaches the system

A machine flake evaluates purely, so it cannot read `/etc` by absolute path. The flake in `/etc/flux` passes its own copy instead:

```nix
modules = [
  flux.nixosModules.flux
  ./hardware-configuration.nix
  ./configuration.nix
  { flux.settingsFile = ./settings.json; }
];
```

- `flux.settingsFile` defaults to `null`: nothing is applied.
- When the option names a file that does not exist yet, nothing is applied either, so a fresh template evaluates.
- If `/etc/flux` is a git repository, `settings.json` must be tracked (`git add`), or the flake does not see it.
- `flux-config` adds that module line to an older `flake.nix` the first time it writes the file, when `./configuration.nix` appears on a line of its own. It keeps the old file as `flake.nix.bak`.
- Values from the file are set at priority 90, so they win over plain definitions in `configuration.nix` (priority 100) and lose to `lib.mkForce`.

## Schema, version 1

```json
{
  "version": 1,
  "hostname": "ada-laptop",
  "timezone": "Europe/Paris",
  "locale": "fr_FR.UTF-8",
  "formats": null,
  "keymap": "fr",
  "users": [
    { "name": "ada", "fullName": "Ada Lovelace", "admin": true, "groups": ["networkmanager", "video", "audio"] }
  ],
  "updates": { "automatic": false },
  "software": { "facile": true, "apps": true, "gaming": false },
  "hardware": { "nvidia": false }
}
```

| Key | Type | Rule | NixOS options |
|---|---|---|---|
| `version` | number | must be `1` | none |
| `hostname` | string | RFC 1123 label: `^[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?$` | `networking.hostName` |
| `timezone` | string or `null` | an IANA zone that exists under zoneinfo; `null` means automatic | `time.timeZone`; `null` also enables `services.automatic-timezoned` |
| `locale` | string | one of the locales in `shell/crates/flux-config/rules.json` | `i18n.defaultLocale` |
| `formats` | string or `null` | same list; `null` means "same as language" | `i18n.extraLocaleSettings.LC_*` (address, measurement, monetary, numeric, paper, telephone, time) |
| `keymap` | string | one of the xkb layouts in `rules.json` | `services.xserver.xkb.layout`, `console.useXkbConfig`, `XKB_DEFAULT_LAYOUT` |
| `users[].name` | string | `^[a-z_][a-z0-9_-]{0,31}$`, not reserved (`rules.json`, `systemd-*`, `nixbld*`), unique | `users.users.<name>` with `isNormalUser`, shell zsh |
| `users[].fullName` | string | 1 to 64 characters, no `:`, `,` or control character, no outer space | `users.users.<name>.description` |
| `users[].admin` | bool | default `false`; at least one user must be `true` | adds `wheel` to `extraGroups` |
| `users[].groups` | list | default `["networkmanager", "video", "audio"]` when the key is missing; drawn from `rules.json` | `users.users.<name>.extraGroups` (replaces the list) |
| `updates.automatic` | bool | default `false` | read at run time by `flux-auto-update.service`; no rebuild needed |
| `software.facile` | bool | optional; Flux default `true` | `flux.software.facile` (the Facile command line tools) |
| `software.apps` | bool | optional; Flux default `true` | `flux.software.apps` (Firefox, Files and the everyday apps; `false` gives a minimal desktop) |
| `software.gaming` | bool | optional; Flux default `false` | `flux.software.gaming` (Steam, GameMode, Gamescope, MangoHud, ntsync) |
| `hardware.nvidia` | bool | optional; Flux default `false` | `flux.hardware.nvidia.enable` (the NVIDIA driver, open module on Turing and newer) |

### Absent `software` and `hardware` keys

Each of the four `software` and `hardware` keys is optional, and absent is not the same as its default. A key that is in the file is applied at priority 90. A key that is not in the file applies nothing, so `configuration.nix` decides (`flux.hardware.nvidia.enable = true;` there keeps working), and the Flux default applies only when `configuration.nix` is silent too.

`flux-config` never adds a key on its own. It writes one only when someone runs `flux-config set software.gaming true` (or the installer writes it), and it leaves out a `software` or `hardware` object that has no key. Renaming the device on a machine whose file has no `hardware` key writes a file that still has none.

New accounts made in Settings get `["networkmanager", "video", "audio", "lp"]`, the same list as the installer's first account. On a machine that has no file yet, the first change reads each person's groups from `/etc/group` and keeps the ones `rules.json` allows.

Unknown keys are refused on both sides. The module reports every broken rule at once, as assertions that start with `settings.json (flux.settingsFile):`.

Supported locales come from the default `i18n.supportedLocales`, which already includes `defaultLocale` and every `extraLocaleSettings` value.

## Passwords

Passwords never live in this file, in `/etc/flux` or in the Nix store. They stay in `/etc/shadow`:

- `users.mutableUsers` stays `true` (the module asserts it when the file lists users).
- Each listed user gets `hashedPassword = null` at priority 90, so an old `configuration.nix` hash stops being applied.
- `flux-config user add` reads the password on stdin, rebuilds so the account exists, then pipes `name:password` to `chpasswd`.
- `flux-config user set-password <name>` does the same without a rebuild.
- The installer sets the first password with `chpasswd -e` inside `nixos-enter` (see `docs/handoff-system.patch`).

## Removing a user

Removing an entry drops `users.users.<name>`. The home folder stays on disk. Renaming changes `fullName` only; the login name never changes, because a new name would mean a new, empty home folder.

## flux-config

`flux-config` is the only writer. It validates every input with the same rules, writes the file atomically (temporary file, `fsync`, rename) after copying the previous one to `settings.json.bak`, and holds `/run/flux-config/lock` so two changes never overlap.

### Commands that change the file

```
flux-config set hostname <name>
flux-config set timezone <zone|auto>
flux-config set locale <locale>
flux-config set formats <locale|same>
flux-config set keymap <layout>
flux-config set auto-update <on|off>            no rebuild
flux-config set software.facile <true|false>
flux-config set software.apps <true|false>
flux-config set software.gaming <true|false>
flux-config set hardware.nvidia <true|false>
flux-config set <key> <value> <key> <value>...  several at once, one rebuild
```

`on`, `yes` and `true` mean the same, as do `off`, `no` and `false`. Every `set` except `auto-update` rebuilds, prints the progress lines below and exits the same way. The operation name in `state.json` is `set-` plus the first key, for example `set-software.gaming`.

### Reading the file

`flux-config get` prints one JSON object:

```json
{
  "exists": true,
  "settings": {
    "version": 1,
    "hostname": "ada-laptop",
    "...": "the other keys of the schema",
    "software": { "facile": true, "apps": true, "gaming": false },
    "hardware": { "nvidia": false }
  },
  "unset": ["software.facile", "software.apps", "hardware.nvidia"]
}
```

- `exists` is `false` when there is no file yet; `settings` is then read from the running system.
- `settings.software` and `settings.hardware` always hold all four values. A key the file does not set shows the Flux default (`facile` true, `apps` true, `gaming` false, `nvidia` false).
- `unset` names the keys that came from a default and not from the file, spelled as `set` takes them. For those, `configuration.nix` may say something else than the value shown.

Rust code that links the crate gets the same values from `Settings::resolved()` and `Settings::unset_choices()`. Never save a resolved copy: it would write the defaults into the file.

### When a rebuild fails

- If `nixos-rebuild` fails before the system switches, `flux-config` puts the previous `settings.json` back, atomically, and exits 1.
- If it exits non-zero after the switch (a service that did not start gives exit status 4 while the new version is already running and is the boot default), the change is kept. `flux-config` compares `/run/current-system` before and after to tell the two apart. It then finishes the operation (the password step of `user add` included), prints `flux-config: warning: the new version is in use, but: ...` on stderr, writes the same line to the log, ends with a `done` line and exits 0.

### flake.lock, updates and going back

- `flux-config update` copies `flake.lock` to `flake.lock.bak` before `nix flake update flux`. If the update or its rebuild fails before the switch, it puts that lock back, so `check` still offers the update and later changes rebuild the version in use.
- After every rebuild that ends with the system switched (`set`, `user`, `apply`, `update`), `flux-config` copies `flake.lock` to `/var/lib/flux-config/locks/<store name of the generation>.lock`. `update` also keeps the lock of the generation it starts from when none is kept yet.
- `flux-config rollback [generation]` puts the kept lock of the target generation back as `flake.lock` once the profile points at it. The next change in Settings then rebuilds the version gone back to, and `check` offers the newer one again. `settings.json` is not rolled back: the next rebuild applies the current file on the older version.
- A generation built before this existed has no kept lock; `rollback` then leaves `flake.lock` as it is and says so in the log.
- Kept locks of generations that no longer exist are deleted after each rebuild and by `gc`.

### Progress

Long operations print one JSON line per step on stdout, as `flux-install --progress json` does:

```
{"step":"start|fetch|evaluate|build|activate|bootloader|clean|password|done","percent":0-100,"message":"..."}
{"step":"error","percent":0,"message":"..."}
```

The last line is also kept in `/run/flux-config/state.json` with the operation name, the helper's pid and whether a restart is needed, and the full output in `/run/flux-config/log`, so a reopened Settings page reattaches to a running change.

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Done. A warning on stderr is possible (see "When a rebuild fails"). |
| 1 | The change failed or the input was refused. The reason is the last stderr line, and the `error` progress line when the operation had started. |
| 2 | Wrong usage: unknown command, key or option. |
| 3 | The change needs administrator rights and `flux-config` was not run as root. |
| 4 | Another change is running. |
| 126 | From `pkexec`, not `flux-config`: the password prompt was dismissed. |
| 127 | From `pkexec`, not `flux-config`: not authorized (wrong password, or the person is not allowed). Its last stderr line is "This incident has been reported.", which is not an error message to show. |

`flux-config` never exits 126 or 127 itself. The `pkexec` codes are read from https://raw.githubusercontent.com/polkit-org/polkit/main/src/programs/pkexec.c .

### Rights

Changes run through `pkexec flux-config ...` under the polkit action `studio.facile.flux.config`. Members of `wheel` confirm with their own password, kept for a few minutes; other people need an administrator's password. Read-only commands (`get`, `status`, `check`, `generations`) need no rights. `FLUX_CONFIG_ROOT=<dir>` moves every path under `<dir>` and never runs a program; `--dry-run` prints the programs and writes nothing on the real root.
