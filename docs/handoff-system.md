# System stream handoff

The system stream adds `flux-config` (root helper), `modules/settings.nix` (reads `/etc/flux/settings.json`) and the `settings-system` crate (System, Users and Updates pages). The schema and the helper contract are in `docs/settings-json.md`. This page lists what the lead applies at merge, in files this stream does not own.

## 1. Template and installer: apply the patch

```sh
git apply docs/handoff-system.patch
```

It touches `template/` and `pkgs/flux-install/flux-install.sh`:

- `template/flake.nix` passes `{ flux.settingsFile = ./settings.json; }`.
- `template/settings.json` (new) holds the defaults that `configuration.nix` held.
- `template/configuration.nix` keeps only NetworkManager and `stateVersion`; no password hash.
- `template/README.md` explains `settings.json`, `flux-config update` and the scratch flow.
- `flux-install.sh` writes `settings.json` with `json_escape` for the full name, stops writing the hash into `/etc/flux`, and sets the password after `nixos-install` with `printf '%s:%s\n' user hash | nixos-enter --root /mnt -c 'chpasswd -e'`. This fixes the "password in the store" finding of the install review.

The patch applies on `feat/desktop-base` at `5e8aadf`. If the install-hardening stream changed these files since, re-apply by hand: the four points above are the whole change.

## 2. `pkgs/flux-shell/default.nix` (settings-core owns it)

The shell workspace now holds `flux-config`, which `pkgs/flux-config` builds alone. Keep it out of `flux-shell` so two packages do not ship the same binary:

```nix
cargoBuildFlags = [ "--workspace" "--exclude" "flux-config" ];
cargoTestFlags = [ "--workspace" "--exclude" "flux-config" ];
```

## 3. The Settings app (settings-core owns `shell/crates/settings`)

- Dependency: `settings-system = { path = "../settings-system" }`.
- Register `settings_system::system`, `settings_system::users` and `settings_system::updates` (each has `page()`, `ID`, `ICON`, `title()`).
- Load `settings_system::CSS` after `shell_common::ui::CSS`. Every rule in it is scoped under `.fx-sys`.

## 4. Swap the local ui copy

`shell/crates/settings-system/src/ui/shared.rs` copies the `shell_common::ui` API from the settings-core worktree (same names and signatures). Once `shell_common::ui` lands:

1. Delete `src/ui/shared.rs`.
2. In `src/ui.rs`, replace `mod shared;` and its `pub use` with `pub use shell_common::ui::{choice_row, danger_button, empty_state, page, primary_button, row, secondary_button, section, spinner_row, switch_row};`.
3. Add `shell-common.workspace = true` to `[dependencies]` (it is a dev-dependency today).
4. Delete `examples/shared.css` and load `shell_common::ui::CSS` in `examples/preview.rs` instead.

## 5. Depends on other streams

- A polkit agent in the session (platform stream). Without one, `pkexec` cannot ask for the password and every change ends as "That did not work".
- The update channel. "Check for updates" compares the locked `flux` revision with the tip of the branch the input follows (`original.ref` in `flake.lock`, else `HEAD`). With the template on `github:FacileStudio/Flux` that is `main`; the install review asks for a `stable` branch (install-hardening stream).

## Preview

```sh
FLUX_SETTINGS_PREVIEW=updates:progress cargo run -p settings-system --example preview
```

States: `system`, `system:zones`, `system:progress`, `system:error`, `users`, `users:add`, `users:manage`, `users:progress`, `updates`, `updates:progress`, `updates:restart`, `updates:confirm`, `updates:error`. With the variable set, the pages show sample data and run nothing.
