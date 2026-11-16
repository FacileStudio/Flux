# Flux

Flux is the Linux distribution of Facile Studio. It is NixOS with the Facile tools, a niri desktop
and a shell designed with muse, all defined in one flake.

The same flake builds three things:

- the desktop you install on a machine
- a live ISO with an installer
- the headless image that every [Boite](https://github.com/FacileStudio/boite) dev VM runs

## What you get

- **Desktop:** niri (scrolling tiling Wayland compositor), Ghostty, and Flux's own statusbar,
  sidebar, launcher and exit menu in GTK 4
- **Design:** colors from the muse design system, applied to the shell, GTK apps, Ghostty and niri
- **Shell:** zsh by default, with vim, Neovim and vis
- **Tools:** every public Facile CLI preinstalled, pinned to its latest release
- **Dev basics:** git, mise, bun, ripgrep, fd, fzf, jq

## Try it in a VM

You need Nix with flakes enabled.

```sh
nix run github:FacileStudio/Flux#vm
```

Log in as `flux` with the password `flux`.

## Install it

Boot the ISO, open a terminal, and run:

```sh
sudo flux-install
```

The installer wipes the disk you choose, after you type its name a second time. It writes your
machine config to `/etc/flux`, as a flake that imports Flux. That config is yours to edit.

To update:

```sh
cd /etc/flux
nix flake update
sudo nixos-rebuild switch --flake /etc/flux#flux
```

## Use Flux on an existing NixOS machine

Add Flux as a flake input and import its module:

```nix
{
  inputs.flux.url = "github:FacileStudio/Flux";

  outputs = { flux, ... }: {
    nixosConfigurations.my-machine = flux.inputs.nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [ flux.nixosModules.flux ./configuration.nix ];
    };
  };
}
```

Or start from the template: `nix flake init -t github:FacileStudio/Flux`.

Set `flux.profile = "headless";` for a server or VM without the desktop.

## Build

| Command | Result |
|---|---|
| `nix run .#vm` | boots the desktop in QEMU |
| `nix build .#iso` | live installer ISO |
| `nix build .#boite` | headless qcow2 image for Boite |
| `nix build .#facile-tools` | the Facile CLIs |
| `nix build .#flux-shell` | statusbar, sidebar, launcher, exit |

Flux targets `x86_64-linux` and `aarch64-linux`.

## Layout

```text
flake.nix     discovers hosts/ and pkgs/, nothing to register
modules/      the Flux NixOS module, one file per concern
hosts/        vm, iso, boite
pkgs/         overlay packages: facile-tools, flux-shell, flux-theme, flux-install
shell/        the desktop shell (Rust, GTK 4), forked from saravenpi/widgets
theme/        muse tokens and the generator for every themed surface
template/     the machine flake the installer writes
```

Adding a file to `modules/` or a directory to `hosts/` or `pkgs/` is enough to wire it in.

## Status

Early work. The VM boots niri, zsh, Ghostty and the editors. The tools, shell, theme, ISO and Boite
image are in progress; see [docs/plan.md](docs/plan.md).
