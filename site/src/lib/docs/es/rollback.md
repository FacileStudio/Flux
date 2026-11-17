# Volver atrás

Cada actualización conserva la versión anterior del sistema. Volver atrás solo cambia el sistema: sus archivos y sus ajustes no se tocan.

## Flux todavía arranca

Abra los Ajustes con **Mod+Coma** y luego Updates. En **Previous versions**, elija **Go back** junto a la versión que quiere.

Desde un terminal:

```sh
pkexec flux-config rollback
```

## Flux no arranca, o la pantalla se queda en negro

1. Mantenga pulsado el botón de encendido hasta que el ordenador se apague.
2. Enciéndalo de nuevo. El menú de arranque aparece solo, porque el último arranque no terminó con un apagado normal. Espera 30 segundos.
3. Elija **Flux - All configurations** y luego la línea con fecha anterior a la actualización.

Flux arranca como estaba en esa fecha. Esta elección vale para un solo arranque. Para conservarla, abra los Ajustes, luego Updates, y elija **Go back** junto a la misma versión.

## Abrir el menú de arranque usted mismo

El menú de arranque está oculto en un arranque normal. Para mostrarlo, encienda el ordenador y pulse **Esc** enseguida, varias veces. En los ordenadores donde Esc abre el menú del firmware, mantenga pulsada **Mayús** en su lugar.

## Cuánto tiempo se conservan las versiones

El menú de arranque lista las diez versiones más recientes. Una vez por semana, Flux elimina las versiones de más de 30 días y las quita del menú.
