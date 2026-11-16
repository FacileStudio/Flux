# Flux installieren

Öffnen Sie in der Live-Sitzung ein Terminal mit **Mod+Eingabe** und führen Sie aus:

```sh
sudo flux-install
```

Das Installationsprogramm:

1. listet die Festplatten des Rechners auf
2. fragt, welche verwendet werden soll, und lässt Sie **den Namen erneut eingeben**
3. **löscht diese Festplatte** und installiert Flux darauf
4. schreibt die Konfiguration Ihres Rechners nach `/etc/flux`

`/etc/flux` ist ein Flake, das Flux importiert. Diese Konfiguration gehört Ihnen: Sie können sie lesen und ändern.

Ziehen Sie nach der Installation den USB-Stick ab und starten Sie neu.
