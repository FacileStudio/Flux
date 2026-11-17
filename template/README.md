# My Flux machine

This flake describes one machine running Flux. `flux-install` writes it to `/etc/flux`. You own it.

- `settings.json` holds the device name, time zone, language, keyboard layout and accounts. Settings edits it for you; `flux-config` does the same from a terminal. Passwords are not in it: they stay in `/etc/shadow`.
- `configuration.nix` is yours for anything else. A value in `settings.json` wins over the same option set here.

## Update

Open Settings, then Updates. From a terminal:

```sh
pkexec flux-config update
```

The `flux` input follows the `stable` branch of Flux, which moves only to tested releases. `flux-install` writes a `flake.lock` that pins Flux and nixpkgs to the exact revision of the ISO you installed from, so the first boot runs what the ISO tested. An update moves the `flux` input to the latest release, records it in `flake.lock` and switches to the result. `pkexec flux-config rollback` goes back one version; older versions also stay in the boot menu.

## Password

The configuration holds no password, so no hash ever reaches `/etc/flux` or the world-readable Nix store. `flux-install` sets the password in `/etc/shadow` after the install. Change it later in Settings, Users, or with `passwd`.

## Start from scratch

```sh
nix flake init -t github:FacileStudio/Flux
sudo nixos-generate-config --show-hardware-config > hardware-configuration.nix
```

Then edit `settings.json` (the schema is in `docs/settings-json.md` of the Flux repository). On an aarch64 machine, change `system` in `flake.nix` to `"aarch64-linux"`.

Your account has no password until you set one, and you cannot log in without it. Set it before you reboot:

```sh
sudo nixos-install --flake .#flux
sudo nixos-enter --root /mnt -c 'passwd <name>'
```

On a machine that already runs NixOS, run `sudo nixos-rebuild switch --flake .#flux`, then `sudo passwd <name>` before you log out.
