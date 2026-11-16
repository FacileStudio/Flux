# Flux on an existing NixOS

Already on NixOS? Add Flux as a flake input and import its module:

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

Or start from the template:

```sh
nix flake init -t github:FacileStudio/Flux
```

For a server or a VM without the desktop, set `flux.profile = "headless";`.
