# Primer arranque

Inicie sesión con la contraseña que eligió en el instalador. Flux se abre en un escritorio vacío, con una barra arriba.

**Mod** es la tecla Super: la tecla Windows en la mayoría de los teclados, Cmd en un teclado Mac.

## Orientarse

- **Abrir una aplicación:** pulse **Mod+Espacio**, escriba unas letras y pulse Intro. La barra también tiene un botón para ello.
- **Cerrar una ventana:** **Mod+Q** o **Alt+F4**. La barra también tiene un botón para ello.
- **Pasar de una ventana a otra:** las ventanas están una junto a otra, en una fila que se desplaza. **Mod+Izquierda** y **Mod+Derecha** van de una a la siguiente. **Alt+Tab** vuelve a la anterior.
- **Ver todo lo que está abierto:** **Mod+Tab**.
- **Bloquear la pantalla:** **Mod+L**.
- **Cerrar sesión, reiniciar o apagar:** **Mod+Esc**.
- **Ver los atajos principales:** **Mod+Mayús+/**. La lista completa está en la página de [atajos](/docs/shortcuts).

## Ajustes

Pulse **Mod+Coma**. Los Ajustes cubren el Wi-Fi, el Bluetooth, el sonido, las pantallas, la apariencia, la energía, los usuarios y las actualizaciones. Las opciones de software del instalador también se cambian ahí.

Para cambiar su contraseña, abra Users y luego **Change password**.

## Añadir aplicaciones

Firefox, Archivos y los visores ya están instalados. Las demás aplicaciones vienen de [flathub.org](https://flathub.org). Flux aún no tiene tienda de aplicaciones. Busque la aplicación en Flathub, copie su identificador y ejecute en el terminal (**Mod+Intro**):

```sh
kit add flathub:org.gimp.GIMP
```

La aplicación aparece después en el lanzador. `kit del flathub:org.gimp.GIMP` la quita, `kit list` muestra lo que ha añadido y `kit update` la actualiza.

Dos cosas que conviene saber:

- No añada con `kit add` una aplicación que Flux ya incluye, como Firefox. Tendría una segunda copia, con su propio perfil.
- kit instala solo para su cuenta. `flatpak install` instala para todas las cuentas, y kit no lista esas aplicaciones.

## Herramientas Facile

Si mantuvo **Facile tools** en el instalador, kit descarga las herramientas Facile de línea de órdenes en su cuenta tras su primer inicio de sesión, cuando el ordenador tiene conexión. Tarda unos minutos. Después ejecute `sablier`, `nuage` o `casier` en el terminal e inicie sesión con `facile login`.

- `systemctl --user status kit-sync` dice si la descarga funcionó. `kit sync` la repite.
- kit comprueba cada hora que no falte nada. No actualiza las herramientas por sí solo: ejecute `kit update`.
- Ardoise, Boîte y Gare no están en la lista inicial, porque necesitan programas que Flux no incluye. Instale esos programas y ejecute `kit add ardoise`.
- Su primer `kit add` o `kit del` da a su cuenta su propia lista, en `~/.config/kit/kit.yml`. Desde entonces, una herramienta que una actualización de Flux añada a la lista inicial ya no llega sola: añádala con `kit add`.
