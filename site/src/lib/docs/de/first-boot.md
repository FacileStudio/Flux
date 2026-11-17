# Erster Start

Melden Sie sich mit dem Passwort an, das Sie im Installationsprogramm gewählt haben. Flux öffnet einen leeren Desktop mit einer Leiste oben.

**Mod** ist die Super-Taste: die Windows-Taste auf den meisten Tastaturen, Cmd auf einer Mac-Tastatur.

## Sich zurechtfinden

- **Ein Programm öffnen:** Drücken Sie **Mod+Leertaste**, tippen Sie ein paar Buchstaben und drücken Sie die Eingabetaste. Die Leiste hat dafür auch eine Schaltfläche.
- **Ein Fenster schließen:** **Mod+Q** oder **Alt+F4**. Die Leiste hat dafür auch eine Schaltfläche.
- **Zwischen Fenstern wechseln:** Die Fenster liegen nebeneinander in einer Reihe, die sich verschiebt. **Mod+Links** und **Mod+Rechts** gehen von einem zum nächsten. **Alt+Tab** geht zum vorherigen zurück.
- **Alles Geöffnete sehen:** **Mod+Tab**.
- **Den Bildschirm sperren:** **Mod+L**.
- **Abmelden, neu starten oder ausschalten:** **Mod+Esc**.
- **Die wichtigsten Tastenkürzel sehen:** **Mod+Umschalt+/**. Die ganze Liste steht auf der Seite [Tastenkürzel](/docs/shortcuts).

## Einstellungen

Drücken Sie **Mod+Komma**. Die Einstellungen decken WLAN, Bluetooth, Ton, Bildschirme, Aussehen, Energie, Benutzer und Updates ab. Auch die Software-Auswahl aus dem Installationsprogramm lässt sich dort ändern.

Um Ihr Passwort zu ändern, öffnen Sie Users und dann **Change password**.

## Weitere Programme holen

Firefox, Dateien und die Betrachter sind schon installiert. Weitere Programme kommen von [flathub.org](https://flathub.org). Flux hat noch keinen App-Store. Suchen Sie das Programm auf Flathub, kopieren Sie seine ID und führen Sie im Terminal (**Mod+Eingabe**) aus:

```sh
kit add flathub:org.gimp.GIMP
```

Das Programm erscheint danach im Starter. `kit del flathub:org.gimp.GIMP` entfernt es, `kit list` zeigt, was Sie hinzugefügt haben, `kit update` aktualisiert es.

Zwei Dinge sollten Sie wissen:

- Fügen Sie mit `kit add` kein Programm hinzu, das Flux schon mitbringt, etwa Firefox. Sie hätten eine zweite Kopie mit eigenem Profil.
- kit installiert nur für Ihr Konto. `flatpak install` installiert für alle Konten, und kit listet diese Programme nicht.

## Facile-Werkzeuge

Wenn Sie **Facile tools** im Installationsprogramm behalten haben, lädt kit die Facile-Befehlszeilenwerkzeuge nach Ihrer ersten Anmeldung in Ihr Konto, sobald der Rechner online ist. Das dauert ein paar Minuten. Starten Sie dann `sablier`, `nuage` oder `casier` im Terminal und melden Sie sich mit `facile login` an.

- `systemctl --user status kit-sync` sagt, ob der Download geklappt hat. `kit sync` wiederholt ihn.
- kit prüft jede Stunde, ob etwas fehlt. Es aktualisiert die Werkzeuge nicht von selbst: Führen Sie `kit update` aus.
- Ardoise, Boîte und Gare stehen nicht in der Startliste, weil sie Programme brauchen, die Flux nicht mitbringt. Installieren Sie diese Programme und führen Sie dann `kit add ardoise` aus.
- Ihr erstes `kit add` oder `kit del` gibt Ihrem Konto eine eigene Liste in `~/.config/kit/kit.yml`. Von da an kommt ein Werkzeug, das ein Flux-Update zur Startliste hinzufügt, nicht mehr von selbst: Fügen Sie es mit `kit add` hinzu.
