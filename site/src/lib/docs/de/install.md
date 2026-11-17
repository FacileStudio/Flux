# Flux installieren

Das Installationsprogramm öffnet sich von selbst, wenn die Live-Sitzung startet. Falls Sie es geschlossen haben, drücken Sie **Mod+Leertaste**, tippen Sie `install` und drücken Sie die Eingabetaste.

Lesen Sie zuerst [Vor dem Umstieg](/docs/before-you-switch): Die Installation löscht eine ganze Festplatte.

Das Installationsprogramm ist vorerst auf Englisch und Französisch. Die Schaltflächen stehen hier mit dem Namen, den Sie auf dem Bildschirm sehen.

## Die Schritte

1. **Willkommen.** Wählen Sie **Install Flux**. **Try Flux** schließt das Installationsprogramm und ändert nichts.
2. **Sprache und Tastatur.** Wählen Sie beides und tippen Sie in das Testfeld, um die Belegung zu prüfen.
3. **Festplatte.** Wählen Sie die Festplatte für Flux. Flux nimmt die ganze Festplatte, und das Installationsprogramm lässt Sie bestätigen, dass sie gelöscht wird.
4. **Ihr Konto.** Ihr Name, ein Benutzername, ein Passwort und ein Name für den Rechner.
5. **Zeitzone.** Klicken Sie auf die Karte oder suchen Sie eine Stadt.
6. **Was installiert wird.** Die Facile-Werkzeuge und die Alltagsprogramme sind eingeschaltet. Spiele sind aus, bis Sie sie einschalten. Der NVIDIA-Treiber wird nur angeboten, wenn der Rechner eine NVIDIA-Karte hat.
7. **Bereit zur Installation.** Prüfen Sie Ihre Auswahl und wählen Sie **Install Flux**.

Die Festplatte wird gelöscht, wenn Sie im letzten Schritt **Install Flux** wählen, nicht vorher. Die Installation dauert ein paar Minuten. Lassen Sie den Rechner eingeschaltet und am Strom.

Wenn das Installationsprogramm **Flux is installed** meldet, ziehen Sie den USB-Stick ab und wählen Sie **Restart now**.

## Internet

Mit der Vorauswahl des Installationsprogramms wird Flux vom USB-Stick kopiert, ohne Internet. Jede andere Auswahl lädt herunter, was der Stick nicht enthält. Verbinden Sie sich also vorher: Öffnen Sie die Einstellungen mit **Mod+Komma**, dann Network.

Die Facile-Werkzeuge werden später geladen, nach Ihrer ersten Anmeldung.

## Wenn es abbricht

Das Installationsprogramm zeigt, was schiefging. **Copy log** kopiert die Details in die Zwischenablage, damit Sie [den Fehler melden](/docs/bugs) können. Gehen Sie zurück und versuchen Sie es erneut, sobald die Ursache behoben ist.

## Im Terminal

```sh
sudo flux-install
```

Der Befehl stellt dieselben Fragen als Text und lässt Sie den Namen der Festplatte zweimal eingeben, bevor er etwas löscht. `flux-install --help` listet die Optionen für eine automatisierte Installation.

Die Installation schreibt die Konfiguration Ihres Rechners nach `/etc/flux`, ein Flake, das Flux importiert. Sie gehört Ihnen: Sie können sie lesen und ändern.
