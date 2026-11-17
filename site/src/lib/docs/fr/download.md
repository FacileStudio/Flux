# Télécharger et vérifier

Téléchargez l’ISO qui correspond à votre processeur sur la [page de téléchargement](/downloads) :

- `flux-0.2.0-x86_64-linux.iso` pour Intel et AMD ;
- `flux-0.2.0-aarch64-linux.iso` pour ARM64 UEFI (VM et serveurs).

Vérifiez ensuite que le fichier est intact. Dans le dossier qui contient l’ISO :

```sh
curl -LO https://flux.facile.studio/releases/v0.2.0/SHA256SUMS
sha256sum --check --ignore-missing SHA256SUMS
```

La commande doit afficher `OK`. Sinon, téléchargez l’ISO à nouveau.
