{ config, lib, ... }:

let
  cfg = config.flux.software;
in
{
  options.flux = {
    software = {
      facile = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "The Facile command line tools: Sablier, Nuage, Casier, Bulle and the rest of the suite.";
      };
      apps = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Everyday desktop apps: Firefox, Files, image, PDF and video viewers, text editor, calculator, archive manager, system monitor and disk usage. Turn off for a minimal desktop.";
      };
      gaming = lib.mkEnableOption "Steam with Proton, GameMode, Gamescope, MangoHud and the ntsync kernel module";
    };
    unfree = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      description = "Names of the unfree packages Flux allows. The gaming and NVIDIA options add theirs; nothing else unfree is accepted.";
    };
  };

  config = {
    flux.kit.enable = lib.mkDefault cfg.facile;
    nixpkgs.config.allowUnfreePredicate = pkg: lib.elem (lib.getName pkg) config.flux.unfree;
  };
}
