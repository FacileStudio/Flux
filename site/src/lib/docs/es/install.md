# Instalar Flux

En la sesión live, abra un terminal con **Mod+Intro** y ejecute:

```sh
sudo flux-install
```

El instalador:

1. lista los discos del equipo
2. le pregunta cuál usar y le pide **escribir su nombre otra vez** para confirmar
3. **borra ese disco** e instala Flux en él
4. escribe la configuración de su equipo en `/etc/flux`

`/etc/flux` es un flake que importa Flux. Esa configuración es suya: puede leerla y modificarla.

Retire la memoria USB y reinicie cuando termine la instalación.
