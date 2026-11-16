# Aktualisieren

Flux aktualisiert sich aus seiner Konfiguration in `/etc/flux`:

```sh
cd /etc/flux
nix flake update
sudo nixos-rebuild switch --flake /etc/flux#flux
```

`nix flake update` holt die neueste Version von Flux und NixOS. `nixos-rebuild switch` baut das neue System und aktiviert es.

Jedes Update erzeugt eine neue Generation. Die alte bleibt: Wenn etwas kaputtgeht, [gehen Sie zurück](/docs/rollback).
