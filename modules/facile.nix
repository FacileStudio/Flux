{ config, lib, pkgs, ... }:

let
  cfg = config.flux.facile;
  unknown = lib.subtractLists pkgs.facile-tools.catalog cfg.exclude;
in
{
  options.flux.facile = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install the Facile CLIs from their GitHub release binaries.";
    };
    exclude = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      example = [ "boite" "gare" ];
      description = "Facile tools to leave out, by catalog name.";
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [{
      assertion = unknown == [ ];
      message = "flux.facile.exclude names unknown tools: ${lib.concatStringsSep ", " unknown}";
    }];
    environment.systemPackages = lib.attrValues (removeAttrs pkgs.facile-tools.tools cfg.exclude);
    programs.nix-ld.enable = true;
    environment.sessionVariables.PATH = [ "$HOME/.local/bin" ];
  };
}
