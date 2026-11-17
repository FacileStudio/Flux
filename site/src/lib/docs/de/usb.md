# Einen bootfähigen USB-Stick erstellen

Alles auf dem Stick wird gelöscht.

## Mit einer App

- Unter Linux schreibt **Impression** das ISO in drei Klicks.
- Unter macOS und Windows erledigt **balenaEtcher** dasselbe.

ISO wählen, Stick wählen, Schreiben starten.

## Auf der Kommandozeile

Finden Sie den Namen des Sticks mit `lsblk` und ersetzen Sie `/dev/sdX` dadurch:

```sh
sudo dd if=flux-0.2.0-x86_64-linux.iso of=/dev/sdX bs=4M status=progress oflag=sync
```

Prüfen Sie den Namen zweimal: `dd` überschreibt das angegebene Laufwerk ohne Rückfrage.
