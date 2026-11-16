# Crear una memoria USB de arranque

Todo el contenido de la memoria se borra.

## Con una app

- En Linux, **Impression** graba la ISO en tres clics.
- En macOS y Windows, **balenaEtcher** hace lo mismo.

Elija la ISO, elija la memoria y empiece la grabación.

## Desde la línea de comandos

Busque el nombre de la memoria con `lsblk` y sustituya `/dev/sdX` por ese nombre:

```sh
sudo dd if=flux-0.1.1-x86_64-linux.iso of=/dev/sdX bs=4M status=progress oflag=sync
```

Compruebe el nombre dos veces: `dd` sobrescribe el disco indicado sin preguntar.
