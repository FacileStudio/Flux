{ config, lib, pkgs, ... }:

let
  cfg = config.flux.kit;
  inherit (pkgs.facile-tools) tools catalog;
  unknown = lib.subtractLists catalog cfg.exclude;
  seeded = lib.subtractLists ([ "facile" "kit" ] ++ cfg.exclude) catalog;
  yamlList = names: if names == [ ] then " []\n" else "\n" + lib.concatMapStrings (n: "  - ${n}\n") names;
in
{
  imports = [
    (lib.mkRenamedOptionModule [ "flux" "facile" "enable" ] [ "flux" "kit" "enable" ])
    (lib.mkRenamedOptionModule [ "flux" "facile" "exclude" ] [ "flux" "kit" "exclude" ])
  ];

  options.flux.kit = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Ship kit and facile, seed /etc/kit/kit.yml with the Facile tools, and run kit sync for each user.";
    };
    exclude = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      example = [ "boite" "gare" ];
      description = "Facile tools to leave out of the kit seed, by catalog name.";
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [{
      assertion = unknown == [ ];
      message = "flux.kit.exclude names unknown tools: ${lib.concatStringsSep ", " unknown}";
    }];

    environment.systemPackages = lib.attrValues (lib.getAttrs (lib.intersectLists [ "kit" "facile" ] (lib.attrNames tools)) tools);

    environment.etc."kit/kit.yml".text = "version: 1\nfacile:${yamlList seeded}nix: []\nflathub: []\n";

    programs.nix-ld.enable = true;
    environment.sessionVariables.PATH = [ "$HOME/.local/state/kit/nix-profile/bin" "$HOME/.local/bin" ];

    systemd.user.services.kit-sync = lib.mkIf (tools ? kit) {
      description = "Install the tools listed in kit.yml";
      wants = [ "network-online.target" ];
      after = [ "network-online.target" ];
      wantedBy = [ "default.target" ];
      path = [ "/run/current-system/sw" ];
      serviceConfig = {
        Type = "oneshot";
        ExecStart = "-${lib.getExe tools.kit} sync --quiet";
      };
    };

    systemd.user.timers.kit-sync = lib.mkIf (tools ? kit) {
      description = "Run kit sync daily";
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnCalendar = "daily";
        Persistent = true;
        RandomizedDelaySec = "1h";
      };
    };
  };
}
