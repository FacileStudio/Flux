# Aktualisieren

Öffnen Sie die Einstellungen mit **Mod+Komma**, dann Updates.

1. **Check for updates** sucht nach einer neueren Version von Flux.
2. **Update now** lädt sie herunter und installiert sie. Sie können die Einstellungen schließen: Das Update läuft weiter.
3. Wenn die Einstellungen **Restart to finish** anzeigen, starten Sie den Rechner neu.

Teile von Flux werden bei einem Update auf Ihrem Rechner gebaut, das kann dauern. Lassen Sie den Rechner am Strom.

**Update automatically** erledigt das einmal pro Woche, solange der Rechner am Strom hängt. Die Option ist aus, bis Sie sie einschalten.

Jedes Update behält die Version davor. Wenn etwas kaputtgeht, [gehen Sie zurück](/docs/rollback).

## Im Terminal

```sh
pkexec flux-config update
```

## Was ein Update tut

Die Konfiguration Ihres Rechners ist ein Flake in `/etc/flux`. Sein Eingang `flux` folgt dem Zweig `stable` von Flux, der nur auf getestete Versionen vorrückt. Ein Update setzt diesen Eingang auf die neueste Version, hält sie in `flake.lock` fest, baut das neue System und wechselt dorthin.

`/etc/flux` gehört root. Ein einfaches `nix flake update` scheitert dort mit einem Berechtigungsfehler: Nehmen Sie den Befehl oben.
