# Flux

Flux is the Linux distribution of Facile Studio. It is NixOS with the Facile tools, a niri desktop
and a shell designed with muse, all defined in one flake.

The same flake builds three things:

- the desktop you install on a machine
- a live ISO with an installer
- the headless image that every [Boîte](https://github.com/FacileStudio/boite) dev VM runs

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
machine config to `/etc/flux`, as a flake that imports Flux. That config is yours to edit with
`sudoedit`. Its `flake.lock` pins Flux and nixpkgs to the ISO's own revisions, and the ISO carries
that system, so the install copies from the USB stick instead of downloading or compiling.

To update to the latest release:

```sh
cd /etc/flux
sudo nix flake update
sudo nixos-rebuild switch --flake /etc/flux#flux
```

## Use Flux on an existing NixOS machine

Add Flux as a flake input and import its module:

```nix
{
  inputs.flux.url = "github:FacileStudio/Flux/stable";

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
| `nix build .#boite` | headless qcow2 image for Boîte |
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

## Releasing

1. Move the `[Unreleased]` changelog entries under the new version and bump `version`.
2. Push a `v<version>` tag. The release workflow first checks the tag against `version` and `CHANGELOG.md`, then builds both ISOs and the Boîte image, runs the boot test, attests the images, signs `SHA256SUMS` when the minisign key is set, and creates a draft GitHub prerelease with the notes, `SHA256SUMS` and `SHA256SUMS.minisig`. The workflow artifacts are kept 90 days.
3. On ruche, run `scripts/publish-assets.sh v<version>`. It downloads the images from that run, checks them against `SHA256SUMS`, publishes them at `https://flux.facile.studio/releases/v<version>/`, checks that every public URL answers 200, then publishes the draft release. The images exceed GitHub's 2 GiB asset limit, so the site serves them.
4. Bump `VERSION` in `site/src/lib/release.ts` and merge it. The CI site job checks that every release link there answers 200, so this step fails until step 3 is done.
5. Move the `stable` branch to the tag: `git push origin v<version>^{commit}:refs/heads/stable` (a fast-forward; create the branch the same way on the first release). Machine flakes follow `github:FacileStudio/Flux/stable`, so `nix flake update` on an installed machine moves to this release and never to `main`. Move it only after the release is public and the boot test passed.

### Signing key

The workflow signs `SHA256SUMS` with minisign when the `MINISIGN_SECRET_KEY` repository secret is set, and skips signing otherwise. To create the key once, offline:

```sh
minisign -G -W -p minisign.pub -s minisign.key
gh secret set MINISIGN_SECRET_KEY -R FacileStudio/Flux < minisign.key
```

Commit `minisign.pub` at the repository root and copy it to `site/static/minisign.pub`, so it is served at `https://flux.facile.studio/minisign.pub`. Keep `minisign.key` offline, out of the repository. Users verify with `minisign -Vm SHA256SUMS -p minisign.pub`, then `sha256sum --check --ignore-missing SHA256SUMS`. Every image also carries a GitHub build provenance attestation: `gh attestation verify flux-<version>-x86_64-linux.iso -R FacileStudio/Flux`.

## Status

Open beta since 0.1.0. The stable release is planned for around March 2027. Expect rough edges and
breaking changes until then, and report bugs at https://github.com/FacileStudio/Flux/issues.
