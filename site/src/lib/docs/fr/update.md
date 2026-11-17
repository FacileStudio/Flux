# Mettre à jour

Ouvrez les Réglages avec **Mod+Virgule**, puis Mises à jour.

1. **Rechercher des mises à jour** cherche une version plus récente de Flux.
2. **Mettre à jour** la télécharge et l’installe. Vous pouvez fermer les Réglages : la mise à jour continue.
3. Quand les Réglages affichent **Redémarrer pour terminer**, redémarrez l’ordinateur.

Des parties de Flux sont construites sur votre ordinateur pendant une mise à jour : cela peut prendre du temps. Gardez l’ordinateur branché.

**Mettre à jour automatiquement** fait tout cela une fois par semaine, quand l’ordinateur est branché. L’option est désactivée tant que vous ne l’activez pas.

Chaque mise à jour garde la version précédente. Si quelque chose casse, [revenez en arrière](/docs/rollback).

## Depuis un terminal

```sh
pkexec flux-config update
```

## Ce que fait une mise à jour

La configuration de votre machine est un flake dans `/etc/flux`. Son entrée `flux` suit la branche `stable` de Flux, qui n’avance que vers des versions testées. Une mise à jour déplace cette entrée vers la dernière version, l’inscrit dans `flake.lock`, construit le nouveau système et bascule dessus.

`/etc/flux` appartient à root. Un simple `nix flake update` y échoue avec une erreur de permission : utilisez la commande ci-dessus.
