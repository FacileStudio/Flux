# Mettre à jour

Flux se met à jour depuis sa configuration, dans `/etc/flux` :

```sh
cd /etc/flux
nix flake update
sudo nixos-rebuild switch --flake /etc/flux#flux
```

`nix flake update` récupère la dernière version de Flux et de NixOS. `nixos-rebuild switch` construit le nouveau système et l’active.

Chaque mise à jour crée une nouvelle génération. L’ancienne reste disponible : si quelque chose casse, [revenez en arrière](/docs/rollback).
