# Flux en un NixOS existente

¿Ya usa NixOS? Añada Flux como entrada de su flake e importe su módulo:

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

O parta de la plantilla:

```sh
nix flake init -t github:FacileStudio/Flux
```

Para un servidor o una VM sin escritorio, añada `flux.profile = "headless";`.
