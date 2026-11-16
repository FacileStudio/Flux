# Créer une clé USB de démarrage

Tout le contenu de la clé est effacé.

## Avec une app

- Sur Linux, **Impression** écrit l’ISO en trois clics.
- Sur macOS et Windows, **balenaEtcher** fait la même chose.

Choisissez l’ISO, choisissez la clé, puis lancez l’écriture.

## En ligne de commande

Trouvez le nom de la clé avec `lsblk`, puis remplacez `/dev/sdX` par ce nom :

```sh
sudo dd if=flux-0.1.0-x86_64-linux.iso of=/dev/sdX bs=4M status=progress oflag=sync
```

Vérifiez deux fois le nom : `dd` écrase le disque indiqué sans rien demander.
