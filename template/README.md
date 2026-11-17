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

The boot menu is hidden on a normal start. It shows by itself after a start that did not end in a clean shutdown: when Flux does not start, hold the power button until the machine turns off, then turn it on again. Tapping Esc at power-on, or holding Shift, shows it at any time. Pick `Flux - All configurations`, then the version you want.

## Password

The configuration holds no password, so no hash ever reaches `/etc/flux` or the world-readable Nix store. `flux-install` sets the password in `/etc/shadow` after the install. Change it later in Settings, Users, or with `passwd`.

## Software

- **Unfree packages.** Flux accepts only the unfree packages named in `flux.unfree`; the gaming and NVIDIA options add theirs. To allow another one, add its name in `configuration.nix`: `flux.unfree = [ "vscode" ];`. Do not set `nixpkgs.config.allowUnfreePredicate` yourself: Nix keeps one predicate, yours would replace Flux's, and Steam or the NVIDIA driver would stop building.
- **Steam on your network.** The gaming option opens no port in the firewall, because the firewall cannot tell your home network from a public one. Steam itself works without it. To stream games to another device with Remote Play, or to copy games from another PC on your network, set `programs.steam.remotePlay.openFirewall = true;` and `programs.steam.localNetworkGameTransfers.openFirewall = true;` in `configuration.nix`.
- **Facile tools.** `/etc/kit/kit.yml` is the list of Facile tools each new account starts with. `flux.kit.exclude = [ "sonar" ];` leaves tools out of it. kit reads that list only until an account has its own: the first `kit add` or `kit del` copies it to `~/.config/kit/kit.yml`, and later changes to the system list no longer reach that account. Add or remove the tool in that account with `kit add` or `kit del`.
- **Apps from Nix.** Apps a user installs with `kit add nix:<name>` appear in the launcher like any other app.
- **Flatpak.** Flux adds Flathub for the whole system, and kit adds it again for each account that uses it. `flatpak install` puts an app in front of every account; `kit add flathub:<id>` installs for one account and does not see the system-wide apps. `kit add firefox` picks the Flathub build although Flux already ships Firefox: do not add apps Flux already has.

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
