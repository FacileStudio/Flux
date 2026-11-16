# Installer Flux

Dans la session live, ouvrez un terminal avec **Mod+Entrée** et lancez :

```sh
sudo flux-install
```

L’installateur :

1. liste les disques de la machine ;
2. vous demande lequel utiliser, puis de **retaper son nom** pour confirmer ;
3. **efface ce disque** et y installe Flux ;
4. écrit la configuration de votre machine dans `/etc/flux`.

`/etc/flux` est un flake qui importe Flux. Cette configuration vous appartient : vous pouvez la lire et la modifier.

Retirez la clé USB et redémarrez quand l’installation est terminée.
