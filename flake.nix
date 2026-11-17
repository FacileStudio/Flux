{
  description = "Flux, the Facile Linux distribution built on NixOS";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      lib = nixpkgs.lib;
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = lib.genAttrs systems;
      dirsIn = path: lib.attrNames (lib.filterAttrs (_: t: t == "directory") (builtins.readDir path));
      mkHost = system: host: lib.nixosSystem {
        inherit system;
        specialArgs.fluxFlake = self;
        modules = [ self.nixosModules.flux ./hosts/${host} ];
      };
      hosts = dirsIn ./hosts;
      build = system: host: (mkHost system host).config.system.build;
      hostPackages = system: lib.filterAttrs (_: v: v != null) {
        vm = (build system "vm").vm;
        iso = if lib.elem "iso" hosts then (build system "iso").isoImage else null;
        boite = if lib.elem "boite" hosts then (build system "boite").images.qemu else null;
      };
    in
    {
      overlays.default = import ./pkgs;

      nixosModules.flux = { imports = [ ./modules ]; nixpkgs.overlays = [ self.overlays.default ]; };
      nixosModules.default = self.nixosModules.flux;

      nixosConfigurations = lib.listToAttrs (lib.concatMap
        (system: map (host: lib.nameValuePair "${host}-${system}" (mkHost system host)) hosts)
        systems);

      packages = forAllSystems (system:
        let pkgs = import nixpkgs { inherit system; overlays = [ self.overlays.default ]; };
        in hostPackages system // lib.genAttrs (dirsIn ./pkgs) (name: pkgs.${name}));

      apps = forAllSystems (system: {
        vm = {
          type = "app";
          program = "${(build system "vm").vm}/bin/run-flux-vm";
        };
      });

      devShells = forAllSystems (system:
        let pkgs = import nixpkgs { inherit system; };
        in {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [ cargo rustc clippy rustfmt pkg-config wrapGAppsHook4 ];
            buildInputs = with pkgs; [ gtk4 gtk4-layer-shell pam librsvg libpulseaudio ];
          };
        });

      templates.default = {
        path = ./template;
        description = "A machine flake that imports Flux";
      };
    };
}
