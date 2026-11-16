# Essayer dans une VM

Avec Nix et les flakes activés, une commande démarre le bureau Flux dans QEMU :

```sh
nix run github:FacileStudio/Flux#vm
```

Connectez-vous avec l’utilisateur `flux` et le mot de passe `flux`.

La VM ne touche pas à votre système. Fermez sa fenêtre pour l’arrêter.
