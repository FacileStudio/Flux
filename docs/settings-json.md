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
| `users[].groups` | list | default `["networkmanager", "video", "audio"]`; drawn from `rules.json` | `users.users.<name>.extraGroups` (replaces the list) |
| `updates.automatic` | bool | default `false` | read at run time by `flux-auto-update.service`; no rebuild needed |
| `software.facile` | bool | default `true`; optional | `flux.software.facile` (the Facile command line tools) |
| `software.apps` | bool | default `true`; optional | `flux.software.apps` (Firefox, Files and the everyday apps; `false` gives a minimal desktop) |
| `software.gaming` | bool | default `false`; optional | `flux.software.gaming` (Steam, GameMode, Gamescope, MangoHud, ntsync) |
| `hardware.nvidia` | bool | default `false`; optional | `flux.hardware.nvidia.enable` (the NVIDIA driver, open module on Turing and newer) |

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

`flux-config` is the only writer. It validates every input with the same rules, writes the file atomically (temporary file, `fsync`, rename) after copying the previous one to `settings.json.bak`, and holds `/run/flux-config/lock` so two changes never overlap. If the rebuild fails, it puts the previous file back.

Long operations print one JSON line per step on stdout, as `flux-install --progress json` does:

```
{"step":"start|fetch|evaluate|build|activate|bootloader|clean|password|done","percent":0-100,"message":"..."}
{"step":"error","percent":0,"message":"..."}
```

The last line is also kept in `/run/flux-config/state.json` with the operation name, the helper's pid and whether a restart is needed, and the full output in `/run/flux-config/log`, so a reopened Settings page reattaches to a running change.

Changes run through `pkexec flux-config ...` under the polkit action `studio.facile.flux.config`. Members of `wheel` confirm with their own password, kept for a few minutes; other people need an administrator's password. Read-only commands (`get`, `status`, `check`, `generations`) need no rights. `FLUX_CONFIG_ROOT=<dir>` moves every path under `<dir>` and never runs a program; `--dry-run` prints the programs and writes nothing on the real root.
