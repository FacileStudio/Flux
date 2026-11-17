# Avant de changer

Flux remplace le système du disque que vous lui donnez. Lisez cette page avant d’effacer quoi que ce soit.

## Ce qui ne marchera pas

- **Les applications Adobe.** Photoshop, Lightroom, Illustrator, InDesign et Premiere Pro n’ont pas de version Linux.
- **Les applications de bureau Microsoft Office.** Word, Excel, PowerPoint et Outlook ne s’installent pas sur Flux. Office sur le web marche dans Firefox.
- **Les jeux avec un anti-triche au niveau du noyau.** Certains jeux en ligne refusent de démarrer sous Linux. Valorant et Fortnite en font partie à l’heure où nous écrivons. Cherchez chacun de vos jeux sur [areweanticheatyet.com](https://areweanticheatyet.com/).
- **Les programmes Windows en général.** Flux ne lance pas les fichiers `.exe` tel quel.

Beaucoup de jeux Windows marchent quand même avec Steam, que l’installateur peut ajouter. Vérifiez chacun sur [protondb.com](https://www.protondb.com/).

Si vous avez besoin d’un de ces programmes pour le travail ou les études, gardez un ordinateur qui le fait tourner.

## Flux prend tout le disque

L’installateur efface le disque que vous choisissez : les fichiers, Windows et tout autre système présent. Il ne peut pas installer Flux à côté de Windows sur le même disque.

- Sauvegardez d’abord vos fichiers sur un autre disque ou dans le cloud.
- Pour garder Windows, installez Flux sur un second disque. Vous choisissez ensuite le système dans le menu de démarrage de l’ordinateur (souvent F12, F10 ou Échap au démarrage).

## Vérifiez l’ordinateur

- Il doit démarrer en mode **UEFI**, avec le **Secure Boot désactivé** dans les réglages du firmware. Flux ne démarre pas avec le Secure Boot activé.
- Le reste est sur la page [Configuration requise](/docs/requirements).

## Essayez d’abord

La clé USB d’installation fait aussi tourner Flux sans l’installer.

1. [Créez la clé USB](/docs/usb) et [démarrez dessus](/docs/live).
2. Choisissez **Essayer Flux**. Rien ne change sur l’ordinateur.
3. Vérifiez l’écran, le Wi-Fi, le son, le Bluetooth, le pavé tactile et le clavier.

La session live oublie tout à l’arrêt. Si quelque chose n’y marche pas, cela ne marchera pas non plus après l’installation : [dites-le-nous](/docs/bugs) avant de changer.

## Ce que vous obtenez

Firefox, un gestionnaire de fichiers, un éditeur de texte, une calculatrice et des visionneuses pour les images, les PDF et les vidéos. Les autres applications viennent de Flathub. Flux n’a pas encore de boutique d’applications : en installer une demande une commande dans le terminal, voir [Premier démarrage](/docs/first-boot).

Flux est en bêta ouverte. Attendez-vous à des défauts.
