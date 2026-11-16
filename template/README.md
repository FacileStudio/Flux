# My Flux machine

This flake describes one machine running Flux. `flux-install` writes it to `/etc/flux`. You own it: edit `configuration.nix`, then rebuild.

## Update

```sh
cd /etc/flux
sudo nix flake update
sudo nixos-rebuild switch --flake /etc/flux#flux
```

`nix flake update` moves Flux and nixpkgs to their latest commits and records them in `flake.lock`. `nixos-rebuild switch` builds and activates the result. Roll back with `sudo nixos-rebuild switch --rollback`, or pick an older generation in the boot menu.

## Start from scratch

```sh
nix flake init -t github:FacileStudio/Flux
sudo nixos-generate-config --show-hardware-config > hardware-configuration.nix
```

Then set `hostName`, `userName` and `hashedPassword` at the top of `configuration.nix`. Generate the hash with `mkpasswd`. The default `"!"` locks password login. On an aarch64 machine, change `system` in `flake.nix` to `"aarch64-linux"`.
