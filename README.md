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
- **Tools:** `kit` installs the public Facile CLIs in each account after the first login, once the machine is online; `kit update` updates them. Flux ships `kit` and `facile` (for `facile login`). Leave tools out with `flux.kit.exclude`. Ardoise, Boîte and Gare stay out of the default list because they need programs Flux does not ship
- **Dev basics:** git, mise, bun, ripgrep, fd, fzf, jq

## Try it in a VM

You need Nix with flakes enabled.

```sh
nix run github:FacileStudio/Flux#vm
```

Log in as `flux` with the password `flux`.

## Install it

Boot the ISO. The graphical installer opens by itself: language and keyboard, disk, account, time
zone, software, then a summary. From a terminal, `sudo flux-install` asks the same questions as text.

The installer wipes the disk you choose, after you confirm. It writes your machine config to
`/etc/flux`, as a flake that imports Flux. That config is yours to edit with `sudoedit`. Its
`flake.lock` pins Flux and nixpkgs to the ISO's own revisions, and the ISO carries that system, so
an install with the default choices copies from the USB stick instead of downloading or compiling.

To update to the latest release, open Settings, then Updates. From a terminal:

```sh
pkexec flux-config update
```

Each update keeps the version before it. Go back from Settings, Updates, or with
`pkexec flux-config rollback`. When Flux does not start, hold the power button until the machine
turns off and start it again: the boot menu shows by itself after a start that did not end in a
clean shutdown. Tapping Esc at power-on shows it at any time.

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
| `nix build .#facile-tools` | kit, facile and the locked Facile CLIs |
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
3. On ruche, run `scripts/publish-assets.sh v<version>`. It downloads the images from that run, checks them against `SHA256SUMS`, publishes them at `https://flux.facile.studio/releases/v<version>/`, checks that every public URL answers 200, publishes the draft release, then fast-forwards the `stable` branch to the tag. The images exceed GitHub's 2 GiB asset limit, so the site serves them. Machine flakes follow `github:FacileStudio/Flux/stable`, so an update on an installed machine moves to this release and never to `main`. The release workflow cannot move `stable` itself: it ends at a draft, before the images are public. The script refuses to move `stable` backwards or onto a tag that is not a descendant of it.
4. Bump `VERSION` in `site/src/lib/release.ts` and merge it. The CI site job checks that every release link there answers 200, so this step fails until step 3 is done.

### Signing key

The workflow signs `SHA256SUMS` with minisign when the `MINISIGN_SECRET_KEY` repository secret is set, and skips signing otherwise. To create the key once, offline:

```sh
minisign -G -W -p minisign.pub -s minisign.key
gh secret set MINISIGN_SECRET_KEY -R FacileStudio/Flux < minisign.key
```

Commit `minisign.pub` at the repository root and copy it to `site/static/minisign.pub`, so it is served at `https://flux.facile.studio/minisign.pub`. Keep `minisign.key` offline, out of the repository. Users verify with `minisign -Vm SHA256SUMS -p minisign.pub`, then `sha256sum --check --ignore-missing SHA256SUMS`. Every image also carries a GitHub build provenance attestation: `gh attestation verify flux-<version>-x86_64-linux.iso -R FacileStudio/Flux`.

## Status

Open beta since 0.1.1. The stable release is planned for around March 2027. Expect rough edges and
breaking changes until then, and report bugs at https://github.com/FacileStudio/Flux/issues.
