# Revenir en arrière

Chaque mise à jour garde la version précédente du système. Revenir en arrière ne change que le système : vos fichiers et vos réglages ne bougent pas.

## Flux démarre encore

Ouvrez les Réglages avec **Mod+Virgule**, puis Mises à jour. Sous **Versions précédentes**, choisissez **Revenir** à côté de la version voulue.

Depuis un terminal :

```sh
pkexec flux-config rollback
```

## Flux ne démarre plus, ou l’écran reste noir

1. Maintenez le bouton d’alimentation jusqu’à ce que l’ordinateur s’éteigne.
2. Rallumez-le. Le menu de démarrage s’affiche tout seul, car le dernier démarrage ne s’est pas terminé par un arrêt normal. Il attend 30 secondes.
3. Choisissez **Flux - All configurations**, puis la ligne datée d’avant la mise à jour.

Flux démarre tel qu’il était à cette date. Ce choix vaut pour un seul démarrage. Pour le garder, ouvrez les Réglages, puis Mises à jour, et choisissez **Revenir** à côté de la même version.

## Ouvrir le menu de démarrage soi-même

Le menu de démarrage est caché lors d’un démarrage normal. Pour l’afficher, allumez l’ordinateur et appuyez tout de suite sur **Échap**, plusieurs fois. Sur les ordinateurs où Échap ouvre le menu du firmware, maintenez **Maj** à la place.

## Combien de temps les versions restent

Le menu de démarrage liste les dix versions les plus récentes. Une fois par semaine, Flux supprime les versions de plus de 30 jours et les retire du menu.
