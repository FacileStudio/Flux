# Volver atrás

Cada actualización conserva la versión anterior del sistema, llamada generación.

## Desde el menú de arranque

Reinicie y elija la generación anterior en el menú. Flux arranca exactamente como antes de la actualización.

## Desde el terminal

```sh
sudo nixos-rebuild switch --rollback
```

El sistema vuelve a la generación anterior sin reiniciar.

Sus archivos personales no se tocan: solo cambia la versión del sistema.
