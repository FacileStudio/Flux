# Changelog

Every notable change to Flux is listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and Flux uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1] - 2026-11-17

The first published release of the 0.3 changes below. The 0.3.0 tag was never published: its boot test could not see the new dark splash.

### Fixed

- The boot test detects a dark splash screen after a black loader.
- A flaky Settings backend test that sometimes saw its own lock still held.

## [0.3.0] - 2026-11-17 [not published]

### Fixed

- The installer's buttons work again. In 0.2.0, Install Flux, Continue and Back did nothing, and the account fields never enabled Continue.
- Terminal, Settings and every other window open at full width instead of half the screen. Super + R still cycles half, two thirds and full.

### Changed

- Flux starts in dark mode, from the boot menu and splash screen to the desktop. Light mode stays one click away in Settings.
- New default wallpaper, a concrete maze courtyard, with five new wallpapers beside it.
- Solar is the Flux icon theme: settings, the installer and the bar use its glyphs, with Adwaita behind it for everything else.
- The installer shows which step you are on, icons on its choices and disks, and its own icon in the launcher.

## [0.2.0] - 2026-11-17

Flux now looks and works like its own system, from the boot menu to the desktop. Still an open beta: tested end to end in virtual machines, not yet on a wide range of real hardware.

### Added

- Graphical installer in muse: try or install, language and keyboard, disk with an erase confirmation, account, time zone and progress. It refuses the USB stick it runs from and checks the system before touching the disk.
- Settings app with Network, Bluetooth, Sound, Displays, Keyboard and mouse, Appearance, Power, System, Users, Updates and About.
- Our own lock screen, idle and lid rules, power modes and night light.
- Muse boot menu, Flux splash screen, quiet boot, and a muse login screen over the wallpaper.
- Default apps: Firefox, Files, image, PDF and video viewers, archive manager, text editor, calculator, clipboard history.
- Printing, Bluetooth, NetworkManager, polkit prompts, keyring, automatic USB mounting, firmware updates, zram and weekly store cleanup.
- Shell, editor, tmux and git defaults: a two-line zsh prompt, fzf and zoxide, Neovim with telescope, treesitter and gitsigns.
- Keys for volume, brightness, media, screenshots, lock, workspaces 5 to 10 and Settings (Super + comma).

### Changed

- Installs are pinned to the release on the ISO and work without a network. Updates follow the `stable` branch.
- Your password is never written to `/etc/flux` or the Nix store.
- The bar, launcher, sidebar and exit menu now work under niri, on every monitor.
- Flux runs in virtual machines without 3D acceleration.

### Fixed

- Many shell fixes: workspaces, volume, the launcher finding apps, the sidebar toggle, notifications urgency.

## [0.1.1] - 2026-11-16

### Security

- The live ISO no longer runs an SSH server. In 0.1.0 it accepted the known login `flux` / `flux` with passwordless sudo, so anyone on the same network could take over a machine running the live session. If you booted the 0.1.0 ISO on a network you do not trust, treat that session as exposed. Installed systems were not affected.

### Fixed

- Installed systems now include redistributable firmware, so Wi-Fi, Bluetooth and AMD graphics work after the first reboot.
- The `facile` CLI is preinstalled, and binaries it installs run thanks to nix-ld.

## [0.1.0] - 2026-11-16

First open beta. Expect rough edges and breaking changes before 1.0.

### Added

- One flake that exports the Flux NixOS module, for `x86_64-linux` and `aarch64-linux`.
- `flux.profile`: `desktop` for the full graphical system, `headless` for servers and VMs.
- Desktop: niri, Ghostty, greetd with tuigreet, PipeWire, and the Inter font.
- Flux shell in GTK 4: statusbar, sidebar, launcher and exit menu, started by niri.
- muse colors applied to the shell, GTK apps, Ghostty and niri through `flux-theme`.
- Light and dark schemes, switched live from the sidebar or with `flux-scheme light|dark|toggle`.
- Wallpapers with a picker in the sidebar and `flux-wallpaper set|list`. The default is a monstera photo by Gilles Lambert (Unsplash License).
- Every public Facile CLI preinstalled, pinned to its latest release, in `facile-tools`.
- zsh by default, with vim, Neovim and vis.
- Dev basics: git, mise, bun, ripgrep, fd, fzf and jq.
- Live installer ISO with `flux-install`, which writes your machine flake to `/etc/flux`.
- Headless qcow2 image for Boite dev VMs, with Go, Rust, Python and Node, published with each release.
- The flux.facile.studio site: install docs, downloads and the open beta announcement.
- `nix run .#vm` to try the desktop in QEMU.
- A machine flake template: `nix flake init -t github:FacileStudio/Flux`.
- Release ISOs for both systems, with a `SHA256SUMS` file.

[Unreleased]: https://github.com/FacileStudio/Flux/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/FacileStudio/Flux/releases/tag/v0.2.0
[0.1.1]: https://github.com/FacileStudio/Flux/releases/tag/v0.1.1
[0.1.0]: https://github.com/FacileStudio/Flux/releases/tag/v0.1.0
