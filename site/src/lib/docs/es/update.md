# Actualizar

Abra los Ajustes con **Mod+Coma** y luego Updates.

1. **Check for updates** busca una versión más reciente de Flux.
2. **Update now** la descarga y la instala. Puede cerrar los Ajustes: la actualización sigue.
3. Cuando los Ajustes digan **Restart to finish**, reinicie el ordenador.

Algunas partes de Flux se compilan en su ordenador durante una actualización, así que puede tardar. Mantenga el ordenador enchufado.

**Update automatically** hace todo esto una vez por semana, con el ordenador enchufado. Está desactivado hasta que usted lo active.

Cada actualización conserva la versión anterior. Si algo falla, [vuelva atrás](/docs/rollback).

## Desde un terminal

```sh
pkexec flux-config update
```

## Qué hace una actualización

La configuración de su equipo es un flake en `/etc/flux`. Su entrada `flux` sigue la rama `stable` de Flux, que solo avanza a versiones probadas. Una actualización mueve esa entrada a la última versión, la registra en `flake.lock`, construye el nuevo sistema y cambia a él.

`/etc/flux` pertenece a root. Un simple `nix flake update` ahí falla con un error de permisos: use la orden de arriba.
