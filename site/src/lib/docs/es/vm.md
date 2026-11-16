# Probar en una VM

Con Nix y los flakes activados, un comando arranca el escritorio Flux en QEMU:

```sh
nix run github:FacileStudio/Flux#vm
```

Inicie sesión con el usuario `flux` y la contraseña `flux`.

La VM no toca su sistema. Cierre su ventana para detenerla.
