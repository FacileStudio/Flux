# Flux plan

Flux is the Facile Linux distribution: one flake that exports a NixOS module. The desktop, the
installer ISO and the Boite VM image are thin hosts on top of that one module.

## Layout and contracts

- `flake.nix` discovers everything. Do not list files in it.
  - every directory in `hosts/` becomes `nixosConfigurations.<host>-<system>`
  - every directory in `pkgs/` becomes `pkgs.<name>` through `overlays.default`, and `packages.<system>.<name>`
  - `packages.<system>.vm`, `.iso` and `.boite` come from `system.build.vm`, `.isoImage` and `.images.qemu`
- `modules/default.nix` imports every `modules/*.nix`. Add a file, never edit the import list.
- `flux.profile` is `desktop` (niri and the graphical stack) or `headless` (shell and tools only).
  Every graphical setting sits behind `lib.mkIf (config.flux.profile == "desktop")`.
- Systems: `x86_64-linux` and `aarch64-linux`. Every package and host evaluates on both.

## Workstreams

| Stream | Owns | Output |
|---|---|---|
| tools | `pkgs/facile-tools/`, `modules/kit.nix`, `scripts/gen-facile-tools.*` | `kit` and `facile` shipped from release binaries; `/etc/kit/kit.yml` seeds every Facile CLI, which `kit sync` installs per user at login and every hour; lockfile generated from `kit/crates/kit-core/catalog/tools.yml` |
| shell | `shell/` (widgets fork with history), `pkgs/flux-shell/`, `modules/desktop-shell.nix` | statusbar, sidebar, launcher, exit built by Nix, started by niri, styled with muse |
| theme | `theme/`, `pkgs/flux-theme/`, `modules/theme.nix`, colors in `modules/niri.kdl` | muse tokens turned into GTK palette, GTK app theme, Ghostty theme, niri colors |
| iso | `hosts/iso/`, `pkgs/flux-install/`, `template/` | live desktop ISO plus a `flux-install` script that writes a machine flake from `template/` |
| boite | `hosts/boite/`, plus a branch in the boite repo | headless qcow2 image, boite distro `flux` as the default |

## Theme contract

`flux-theme` installs `$out/share/flux/palette.css`: GTK `@define-color` lines, one per muse color
token, named `fc_<token>` with dashes turned to underscores (`--color-fc-page` becomes `fc_page`).
`modules/theme.nix` links it to `/etc/flux/palette.css`. The shell reads that file, so widget CSS uses
only `@fc_*` names.

The system UI font is Inter. Magnetik is licensed for web use only; shipping it in an OS image needs a
desktop licence, so it stays opt-in through `flux.theme.font`.

## Later

- Binary cache (attic on ruche) so users do not compile the shell and the Rust tools.
- Install Nix on ruche; today builds run in the `nixos/nix` Docker image.
- CI: evaluate every host on both systems, bump `facile-tools` when a tool releases.
- Upload the Boite image to `boite.facile.studio/flux.qcow2` and pin its SHA256 in boite.
