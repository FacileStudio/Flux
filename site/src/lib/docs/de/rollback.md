# Zurückdrehen

Jedes Update behält die vorherige Version des Systems, eine sogenannte Generation.

## Im Bootmenü

Starten Sie neu und wählen Sie die vorherige Generation im Menü. Flux startet genau wie vor dem Update.

## Im Terminal

```sh
sudo nixos-rebuild switch --rollback
```

Das System kehrt ohne Neustart zur vorherigen Generation zurück.

Ihre persönlichen Dateien bleiben unberührt: Nur die Systemversion ändert sich.
