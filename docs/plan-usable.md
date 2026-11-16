# Flux "usable" plan

Goal: a complete desktop a friend can install and use on day one, with our own Rust GTK 4 components,
all in muse, with subtle motion. Review reports this plan is built from live in
`~/.cache/facile-wt/flux-review/` (shell.md, dotfiles.md, install-release.md, nix.md when it lands).

## Rules for every agent

- Work only in the paths you own. Shared files have one owner each (listed below).
- Compile your Rust: `scripts/dev.sh cargo check -p <crate>` and `scripts/dev.sh cargo test -p <crate>`.
  It runs cargo in the shared Nix dev shell. Never `nix build` an ISO, an image or the whole shell here.
  Keep `df -h /` above 40 GB; stop and report if it drops.
- muse: colors only through `@fc_*` names from `/etc/flux/palette-<scheme>.css` (loaded by
  `shell_common`), no borders anywhere, Inter, 12px radius on surfaces and 8px on controls, light and dark.
- Motion: 120-200 ms ease-out transitions on hover, press, reveal and page change (GTK CSS `transition`,
  `GtkRevealer`, `GtkStack` crossfade or slide). Nothing bounces, nothing loops. Honour the GTK
  "enable animations" setting: with it off, everything is instant.
- Every blocking call (subprocess, D-Bus, file decode) runs off the GTK main thread.
- No `unwrap`/`expect` on I/O, D-Bus or parsing. A missing service shows an empty state, never a crash.
- Strings in English and French through the crate's key table.
- No inline code comments. Conventional Commits, author saravenpi, no Co-Authored-By. Do not push.
- Report: files, commands run with results, what is unverified, open questions.

## Settings architecture

One app, `flux-settings` (crate `shell/crates/settings`), with a left navigation and one page per
domain. Each page is its own library crate so agents never share a file:

```rust
pub fn page() -> gtk4::Widget;          // the whole page, self-contained
pub const ID: &str; pub const ICON: &str; pub fn title() -> String;
```

Shared look comes from `shell_common::ui` (owned by the settings-core agent): `section(title)`,
`row(title, subtitle, suffix_widget)`, `switch_row`, `slider_row`, `choice_row`, `empty_state`,
`spinner_row`, and CSS classes `.fx-page .fx-section .fx-row .fx-title .fx-subtitle .fx-danger`.
Until it lands, page crates build these few helpers locally in a `ui.rs` with the same names, and the
lead swaps them at merge.

System state NixOS owns declaratively (hostname, users, keyboard, locale) lives in
`/etc/flux/settings.json`, read by the machine config with `lib.importJSON`. The root helper
`flux-config` (polkit-gated) edits it and runs the rebuild; pages call it and show progress.

## Workstreams

| Agent | Owns | Delivers |
|---|---|---|
| shell-fix | `shell/crates/{statusbar,launcher,exit,sidebar}`, `shell-common` except `ui.rs`, `modules/desktop-shell.nix`, `modules/niri.kdl` | every finding in shell.md; niri binds for media, brightness, screenshots, workspaces 5-10, lock; bar on every monitor |
| settings-core | `shell/crates/settings`, `shell-common/src/ui.rs`, `pkgs/flux-shell` | app window, navigation, page registry, Appearance page (theme, wallpaper, accent-free), About page |
| net | `shell/crates/settings-network` | Wi-Fi list, connect, forget, hidden network, wired state, airplane toggle (NetworkManager D-Bus) |
| bluetooth | `shell/crates/settings-bluetooth` | power, scan, pair with PIN confirm, connect, forget, battery (BlueZ D-Bus) |
| audio | `shell/crates/settings-audio` | output and input device choice, volume, mute, per-app volume, test sound |
| display | `shell/crates/settings-display` | per-output mode, scale, rotation, arrangement, night light; keyboard layout and touchpad options (niri) |
| system | `shell/crates/settings-system`, `pkgs/flux-config`, `modules/settings.nix` | hostname, users, date and time, language, updates and rollback, `settings.json` plumbing |
| lock-power | `shell/crates/lock`, `shell/crates/settings-power`, `modules/power.nix` | our lock screen (ext-session-lock + PAM), idle and lid rules, power profiles, battery page |
| defaults | `modules/shell.nix`, `modules/editors.nix` (new), Ghostty lines of `modules/theme.nix` | the adopt list in dotfiles.md |
| platform | `modules/apps.nix`, `modules/hardware.nix`, `modules/services.nix` (new), `hosts/vm` | default apps and mime, NetworkManager, BlueZ, printing, fwupd, udisks, keyring, polkit agent, fonts, zram, firewall, software rendering in VMs |
| install-hardening | `pkgs/flux-install`, `template/`, `scripts/publish-assets.sh`, `.github/workflows/release.yml`, `site/nginx.conf` | the Critical and Important items of install-release.md (after the installer GUI branch merges) |

Wave 2, after these merge: quick menus in the statusbar (Wi-Fi, Bluetooth, volume popovers reusing the
page crates), our own polkit agent, on-screen display for volume and brightness, a first-run welcome.

## Done when

CI builds every package on both systems, the boot test installs from the ISO and screenshots every
screen, and a person can: connect to Wi-Fi, pair a Bluetooth device, switch audio output, change
scale and wallpaper and theme, rename the machine, lock, suspend, update, and roll back, without a
terminal.
