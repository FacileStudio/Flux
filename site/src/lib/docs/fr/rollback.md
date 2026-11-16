# Revenir en arrière

Chaque mise à jour garde la version précédente du système, appelée génération.

## Depuis le menu de démarrage

Redémarrez, puis choisissez la génération précédente dans le menu. Flux démarre exactement comme avant la mise à jour.

## Depuis le terminal

```sh
sudo nixos-rebuild switch --rollback
```

Le système revient à la génération précédente sans redémarrer.

Vos fichiers personnels ne bougent pas : seul le système change de version.
