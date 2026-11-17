# Instalar Flux

El instalador se abre solo al arrancar la sesión live. Si lo cerró, pulse **Mod+Espacio**, escriba `install` y pulse Intro.

Lea primero [Antes de cambiar](/docs/before-you-switch): la instalación borra un disco entero.

El instalador está en inglés y en francés por ahora. Los botones aparecen aquí con el nombre que verá en pantalla.

## Los pasos

1. **Bienvenida.** Elija **Install Flux**. **Try Flux** cierra el instalador y no cambia nada.
2. **Idioma y teclado.** Elija el idioma y el teclado, y escriba en el campo de prueba para comprobar la distribución.
3. **Disco.** Elija el disco que recibe Flux. Flux ocupa todo el disco, y el instalador le pide confirmar que se borrará.
4. **Su cuenta.** Su nombre, un nombre de usuario, una contraseña y un nombre para el ordenador.
5. **Zona horaria.** Haga clic en el mapa o busque una ciudad.
6. **Qué instalar.** Las herramientas Facile y las aplicaciones de uso diario están activadas. Los juegos están desactivados hasta que usted los active. El controlador NVIDIA solo se ofrece si el ordenador tiene una tarjeta NVIDIA.
7. **Listo para instalar.** Revise sus elecciones y elija **Install Flux**.

El disco se borra cuando elige **Install Flux** en el último paso, no antes. La instalación tarda unos minutos. Mantenga el ordenador encendido y enchufado.

Cuando el instalador diga **Flux is installed**, retire la memoria USB y elija **Restart now**.

## Internet

Con las opciones iniciales del instalador, Flux se copia desde la memoria USB, sin internet. Cualquier otra opción descarga lo que la memoria no contiene: conéctese antes, abriendo los Ajustes con **Mod+Coma** y luego Network.

Las herramientas Facile se descargan después, tras su primer inicio de sesión.

## Si se detiene

El instalador muestra qué ha fallado. **Copy log** copia los detalles al portapapeles para [informar del error](/docs/bugs). Vuelva atrás e inténtelo de nuevo cuando la causa esté resuelta.

## Desde un terminal

```sh
sudo flux-install
```

Hace las mismas preguntas en texto, y le pide escribir dos veces el nombre del disco antes de borrar nada. `flux-install --help` lista las opciones para una instalación automatizada.

La instalación escribe la configuración de su equipo en `/etc/flux`, un flake que importa Flux. Es suya: puede leerla y modificarla.
