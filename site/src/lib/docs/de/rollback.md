# Zurückdrehen

Jedes Update behält die Version des Systems davor. Zurückdrehen ändert nur das System: Ihre Dateien und Einstellungen bleiben, wo sie sind.

## Flux startet noch

Öffnen Sie die Einstellungen mit **Mod+Komma**, dann Updates. Wählen Sie unter **Previous versions** neben der gewünschten Version **Go back**.

Im Terminal:

```sh
pkexec flux-config rollback
```

## Flux startet nicht, oder der Bildschirm bleibt schwarz

1. Halten Sie den Einschaltknopf gedrückt, bis der Rechner ausgeht.
2. Schalten Sie ihn wieder ein. Das Bootmenü erscheint von selbst, weil der letzte Start nicht mit einem sauberen Herunterfahren endete. Es wartet 30 Sekunden.
3. Wählen Sie **Flux - All configurations** und dann die Zeile mit dem Datum vor dem Update.

Flux startet so, wie es an diesem Datum war. Diese Wahl gilt für einen Start. Um sie zu behalten, öffnen Sie die Einstellungen, dann Updates, und wählen Sie **Go back** neben derselben Version.

## Das Bootmenü selbst öffnen

Bei einem normalen Start ist das Bootmenü verborgen. Um es zu zeigen, schalten Sie den Rechner ein und drücken Sie sofort mehrmals **Esc**. Auf Rechnern, bei denen Esc das Firmware-Menü öffnet, halten Sie stattdessen **Umschalt** gedrückt.

## Wie lange Versionen bleiben

Das Bootmenü listet die zehn neuesten Versionen. Einmal pro Woche löscht Flux die Versionen, die älter als 30 Tage sind, und nimmt sie aus dem Menü.
