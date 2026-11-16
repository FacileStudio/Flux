# Changelog

Every notable change to Flux is listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and Flux uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-11-16

First open beta. Expect rough edges and breaking changes before 1.0.

### Added

- One flake that exports the Flux NixOS module, for `x86_64-linux` and `aarch64-linux`.
- `flux.profile`: `desktop` for the full graphical system, `headless` for servers and VMs.
- Desktop: niri, Ghostty, greetd with tuigreet, PipeWire, and the Inter font.
- Flux shell in GTK 4: statusbar, sidebar, launcher and exit menu, started by niri.
- muse colors applied to the shell, GTK apps, Ghostty and niri through `flux-theme`.
- Every public Facile CLI preinstalled, pinned to its latest release, in `facile-tools`.
- zsh by default, with vim, Neovim and vis.
- Dev basics: git, mise, bun, ripgrep, fd, fzf and jq.
- Live installer ISO with `flux-install`, which writes your machine flake to `/etc/flux`.
- Headless qcow2 image for Boite dev VMs.
- `nix run .#vm` to try the desktop in QEMU.
- A machine flake template: `nix flake init -t github:FacileStudio/Flux`.
- Release ISOs for both systems, with a `SHA256SUMS` file.

[Unreleased]: https://github.com/FacileStudio/Flux/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/FacileStudio/Flux/releases/tag/v0.1.0
