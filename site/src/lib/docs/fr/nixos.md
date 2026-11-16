# Flux sur un NixOS existant

Déjà sur NixOS ? Ajoutez Flux comme entrée de votre flake et importez son module :

```nix
{
  inputs.flux.url = "github:FacileStudio/Flux";

  outputs = { flux, ... }: {
    nixosConfigurations.my-machine = flux.inputs.nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [ flux.nixosModules.flux ./configuration.nix ];
    };
  };
}
```

Ou partez du modèle :

```sh
nix flake init -t github:FacileStudio/Flux
```

Pour un serveur ou une VM sans bureau, ajoutez `flux.profile = "headless";`.
