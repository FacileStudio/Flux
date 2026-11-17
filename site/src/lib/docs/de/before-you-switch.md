# Vor dem Umstieg

Flux ersetzt das System auf der Festplatte, die Sie ihm geben. Lesen Sie diese Seite, bevor Sie etwas löschen.

## Was nicht laufen wird

- **Adobe-Programme.** Photoshop, Lightroom, Illustrator, InDesign und Premiere Pro gibt es nicht für Linux.
- **Die Desktop-Programme von Microsoft Office.** Word, Excel, PowerPoint und Outlook lassen sich auf Flux nicht installieren. Office im Web funktioniert in Firefox.
- **Spiele mit Kernel-Anti-Cheat.** Manche Online-Spiele starten unter Linux nicht. Valorant und Fortnite gehören derzeit dazu. Schlagen Sie jedes Ihrer Spiele auf [areweanticheatyet.com](https://areweanticheatyet.com/) nach.
- **Windows-Programme allgemein.** Flux führt `.exe`-Dateien nicht ohne Weiteres aus.

Viele Windows-Spiele laufen trotzdem über Steam, das das Installationsprogramm hinzufügen kann. Prüfen Sie jedes auf [protondb.com](https://www.protondb.com/).

Wenn Sie eines dieser Programme für Arbeit oder Ausbildung brauchen, behalten Sie einen Rechner, auf dem es läuft.

## Flux nimmt die ganze Festplatte

Das Installationsprogramm löscht die gewählte Festplatte: Dateien, Windows und jedes andere System darauf. Es kann Flux nicht neben Windows auf derselben Festplatte installieren.

- Sichern Sie Ihre Dateien vorher auf einer anderen Festplatte oder in der Cloud.
- Um Windows zu behalten, installieren Sie Flux auf einer zweiten Festplatte. Das System wählen Sie dann im Bootmenü des Rechners (oft F12, F10 oder Esc beim Start).

## Den Rechner prüfen

- Er muss im **UEFI**-Modus starten, und **Secure Boot muss** in den Firmware-Einstellungen **ausgeschaltet** sein. Flux startet nicht mit eingeschaltetem Secure Boot.
- Alles Weitere steht auf der Seite [Voraussetzungen](/docs/requirements).

## Erst ausprobieren

Der USB-Stick für die Installation startet Flux auch, ohne es zu installieren.

1. [Erstellen Sie den USB-Stick](/docs/usb) und [starten Sie davon](/docs/live).
2. Wählen Sie **Try Flux**. Am Rechner ändert sich nichts.
3. Prüfen Sie Bildschirm, WLAN, Ton, Bluetooth, Touchpad und Tastatur.

Die Live-Sitzung vergisst beim Herunterfahren alles. Was dort nicht funktioniert, funktioniert auch nach der Installation nicht: [Sagen Sie es uns](/docs/bugs), bevor Sie umsteigen.

## Was Sie bekommen

Firefox, einen Dateimanager, einen Texteditor, einen Taschenrechner und Betrachter für Bilder, PDFs und Videos. Weitere Programme kommen von Flathub. Flux hat noch keinen App-Store: Eine Installation braucht einen Befehl im Terminal, siehe [Erster Start](/docs/first-boot).

Das Installationsprogramm und die Einstellungen von Flux sind vorerst auf Englisch und Französisch. Flux ist in der offenen Beta: Rechnen Sie mit Ecken und Kanten.
