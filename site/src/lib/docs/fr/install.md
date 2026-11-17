# Installer Flux

L’installateur s’ouvre tout seul au démarrage de la session live. Si vous l’avez fermé, appuyez sur **Mod+Espace**, tapez `install` puis Entrée.

Lisez d’abord [Avant de changer](/docs/before-you-switch) : l’installation efface un disque entier.

## Les étapes

1. **Bienvenue.** Choisissez **Installer Flux**. **Essayer Flux** ferme l’installateur et ne change rien.
2. **Langue et clavier.** Choisissez les deux, puis tapez dans le champ de test pour vérifier la disposition.
3. **Disque.** Choisissez le disque qui reçoit Flux. Flux prend tout le disque, et l’installateur vous demande de confirmer qu’il sera effacé.
4. **Votre compte.** Votre nom, un nom d’utilisateur, un mot de passe et un nom pour l’ordinateur.
5. **Fuseau horaire.** Cliquez sur la carte ou cherchez une ville.
6. **Que faut-il installer.** Les outils Facile et les applications du quotidien sont activés. Les jeux restent désactivés tant que vous ne les activez pas. Le pilote NVIDIA n’est proposé que si l’ordinateur a une carte NVIDIA.
7. **Prêt à installer.** Vérifiez vos choix, puis choisissez **Installer Flux**.

Le disque est effacé quand vous choisissez **Installer Flux** à la dernière étape, pas avant. L’installation prend quelques minutes. Gardez l’ordinateur allumé et branché.

Quand l’installateur affiche **Flux est installé**, retirez la clé USB et choisissez **Redémarrer maintenant**.

## Internet

Avec les choix de départ de l’installateur, Flux est copié depuis la clé USB, sans internet. Tout autre choix télécharge ce que la clé ne contient pas : connectez-vous d’abord, en ouvrant les Réglages avec **Mod+Virgule**, puis Réseau.

Les outils Facile se téléchargent plus tard, après votre première connexion.

## Si l’installation s’arrête

L’installateur affiche ce qui s’est passé. **Copier le journal** met le détail dans le presse-papiers pour [signaler le bug](/docs/bugs). Revenez en arrière et réessayez une fois la cause corrigée.

## Depuis un terminal

```sh
sudo flux-install
```

La commande pose les mêmes questions en texte, et vous fait taper deux fois le nom du disque avant d’effacer quoi que ce soit. `flux-install --help` liste les options pour une installation scriptée.

L’installation écrit la configuration de votre machine dans `/etc/flux`, un flake qui importe Flux. Elle vous appartient : vous pouvez la lire et la modifier.
