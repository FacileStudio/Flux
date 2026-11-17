# Changelog

Every notable change to Flux is listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and Flux uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Login: ReGreet replaces tuigreet. A muse card over the default wallpaper with the Flux mark, clock, password field and quiet restart and power buttons, in light or dark from `flux.theme.scheme`. A machine with one user goes straight to the password.

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

[Unreleased]: https://github.com/FacileStudio/Flux/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/FacileStudio/Flux/releases/tag/v0.1.1
[0.1.0]: https://github.com/FacileStudio/Flux/releases/tag/v0.1.0
