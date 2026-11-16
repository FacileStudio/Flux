# Flux auf einem bestehenden NixOS

Schon auf NixOS? Fügen Sie Flux als Flake-Input hinzu und importieren Sie sein Modul:

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

Oder starten Sie mit der Vorlage:

```sh
nix flake init -t github:FacileStudio/Flux
```

Für einen Server oder eine VM ohne Desktop setzen Sie `flux.profile = "headless";`.
