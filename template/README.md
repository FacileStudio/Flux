# My Flux machine

This flake describes one machine running Flux. `flux-install` writes it to `/etc/flux`. You own it: edit `configuration.nix` with `sudoedit`, then rebuild.

## Update

```sh
cd /etc/flux
sudo nix flake update
sudo nixos-rebuild switch --flake /etc/flux#flux
```

The `flux` input follows the `stable` branch of Flux, which moves only to tested releases. `flux-install` writes a `flake.lock` that pins Flux and nixpkgs to the exact revision of the ISO you installed from, so the first boot runs what the ISO tested. `nix flake update` moves both to the latest release and records them in `flake.lock`. `nixos-rebuild switch` builds and activates the result. Roll back with `sudo nixos-rebuild switch --rollback`, or pick an older generation in the boot menu.

## Password

The configuration holds no password, so no hash ever reaches `/etc/flux` or the world-readable Nix store. `flux-install` sets the password in `/etc/shadow` after the install. Change it later with `passwd`.

## Start from scratch

```sh
nix flake init -t github:FacileStudio/Flux
sudo nixos-generate-config --show-hardware-config > hardware-configuration.nix
```

Then set `hostName`, `userName`, `fullName`, `timeZone`, `keyMap` (an xkb layout such as `fr`, also used on the console) and `locale` at the top of `configuration.nix`. On an aarch64 machine, change `system` in `flake.nix` to `"aarch64-linux"`.

Your account has no password until you set one, and you cannot log in without it. Set it before you reboot:

```sh
sudo nixos-install --flake .#flux
sudo nixos-enter --root /mnt -c 'passwd <userName>'
```

On a machine that already runs NixOS, run `sudo nixos-rebuild switch --flake .#flux`, then `sudo passwd <userName>` before you log out.
