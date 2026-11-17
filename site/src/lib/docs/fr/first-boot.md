# Premier démarrage

Connectez-vous avec le mot de passe choisi dans l’installateur. Flux s’ouvre sur un bureau vide, avec une barre en haut.

**Mod** est la touche Super : la touche Windows sur la plupart des claviers, Cmd sur un clavier Mac.

## Se repérer

- **Ouvrir une application :** appuyez sur **Mod+Espace**, tapez quelques lettres, puis Entrée. La barre a aussi un bouton pour cela.
- **Fermer une fenêtre :** **Mod+Q** ou **Alt+F4**. La barre a aussi un bouton pour cela.
- **Passer d’une fenêtre à l’autre :** les fenêtres sont côte à côte, sur une rangée qui défile. **Mod+Gauche** et **Mod+Droite** vont de l’une à l’autre. **Alt+Tab** revient à la précédente.
- **Voir tout ce qui est ouvert :** **Mod+Tab**.
- **Verrouiller l’écran :** **Mod+L**.
- **Se déconnecter, redémarrer ou éteindre :** **Mod+Échap**.
- **Voir les principaux raccourcis :** **Mod+Maj+/**. La liste complète est sur la page [Raccourcis clavier](/docs/shortcuts).

## Réglages

Appuyez sur **Mod+Virgule**. Les Réglages couvrent le Wi-Fi, le Bluetooth, le son, les écrans, l’apparence, l’énergie, les utilisateurs et les mises à jour. Les choix de logiciels de l’installateur s’y modifient aussi.

Pour changer votre mot de passe, ouvrez Utilisateurs, puis **Changer le mot de passe**.

## Ajouter des applications

Firefox, Fichiers et les visionneuses sont déjà installés. Les autres applications viennent de [flathub.org](https://flathub.org). Flux n’a pas encore de boutique d’applications. Trouvez l’application sur Flathub, copiez son identifiant, puis lancez dans le terminal (**Mod+Entrée**) :

```sh
kit add flathub:org.gimp.GIMP
```

L’application apparaît ensuite dans le lanceur. `kit del flathub:org.gimp.GIMP` la retire, `kit list` montre ce que vous avez ajouté, `kit update` la met à jour.

Deux choses à savoir :

- N’ajoutez pas avec `kit add` une application que Flux fournit déjà, comme Firefox. Vous en auriez une seconde copie, avec son propre profil.
- kit installe pour votre compte seulement. `flatpak install` installe pour tous les comptes, et kit ne liste pas ces applications.

## Outils Facile

Si vous avez gardé **Outils Facile** dans l’installateur, kit télécharge les outils Facile en ligne de commande dans votre compte après votre première connexion, une fois l’ordinateur en ligne. Cela prend quelques minutes. Lancez ensuite `sablier`, `nuage` ou `casier` dans le terminal, et connectez-vous avec `facile login`.

- `systemctl --user status kit-sync` dit si le téléchargement a marché. `kit sync` le relance.
- kit vérifie toutes les heures que rien ne manque. Il ne met pas les outils à jour tout seul : lancez `kit update`.
- Ardoise, Boîte et Gare ne sont pas dans la liste de départ, car ils ont besoin de programmes que Flux ne fournit pas. Installez ces programmes, puis lancez `kit add ardoise`.
- Votre premier `kit add` ou `kit del` donne à votre compte sa propre liste, dans `~/.config/kit/kit.yml`. À partir de là, un outil qu’une mise à jour de Flux ajoute à la liste de départ n’arrive plus tout seul : ajoutez-le avec `kit add`.
