# Update

Flux updates from its config in `/etc/flux`:

```sh
cd /etc/flux
nix flake update
sudo nixos-rebuild switch --flake /etc/flux#flux
```

`nix flake update` fetches the latest Flux and NixOS. `nixos-rebuild switch` builds the new system and turns it on.

Each update makes a new generation. The old one stays: if something breaks, [roll back](/docs/rollback).
