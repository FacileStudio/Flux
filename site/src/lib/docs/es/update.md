# Actualizar

Flux se actualiza desde su configuración en `/etc/flux`:

```sh
cd /etc/flux
nix flake update
sudo nixos-rebuild switch --flake /etc/flux#flux
```

`nix flake update` trae la última versión de Flux y de NixOS. `nixos-rebuild switch` construye el nuevo sistema y lo activa.

Cada actualización crea una nueva generación. La anterior se conserva: si algo falla, [vuelva atrás](/docs/rollback).
