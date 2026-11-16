{
  inputs.flux.url = "github:FacileStudio/Flux";

  outputs = { flux, ... }: {
    nixosConfigurations.flux = flux.inputs.nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        flux.nixosModules.flux
        ./hardware-configuration.nix
        ./configuration.nix
      ];
    };
  };
}
