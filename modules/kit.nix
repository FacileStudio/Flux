{ config, lib, pkgs, ... }:

let
  cfg = config.flux.kit;
  inherit (pkgs.facile-tools) tools catalog;
  unknown = lib.subtractLists catalog cfg.exclude;
  /** Tools whose catalog entry requires programs Flux does not ship (typst, qemu, podman and caddy): kit refuses to install them, so they stay out of the seed. */
  needsHostTools = [ "ardoise" "boite" "gare" ];
  seeded = lib.subtractLists ([ "facile" "kit" ] ++ needsHostTools ++ cfg.exclude) (lib.attrNames tools);
  sync = pkgs.writeShellScript "kit-sync" ''
    ${lib.optionalString config.networking.networkmanager.enable "${pkgs.networkmanager}/bin/nm-online --quiet --timeout=300 || exit 1"}
    exec ${lib.getExe tools.kit} sync --quiet
  '';
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
      description = "Ship kit and facile, seed /etc/kit/kit.yml with the Facile tools, and run kit sync for each user. kit reads the seed only until an account has its own list: the first kit add or kit del copies it to ~/.config/kit/kit.yml, and later changes to the seed no longer reach that account.";
    };
    exclude = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      example = [ "boite" "gare" ];
      description = "Facile tools to leave out of the kit seed, by catalog name. Tools that have no release for this machine's processor, and ardoise, boite and gare, which need programs Flux does not ship, are always left out; add one of those with kit add once its requirements are installed.";
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
    environment.profiles = [ "$HOME/.local/state/kit/nix-profile" ];
    environment.sessionVariables.PATH = [ "$HOME/.local/bin" ];

    systemd.user.services.kit-sync = lib.mkIf (tools ? kit) {
      description = "Install the tools listed in kit.yml";
      wantedBy = [ "default.target" ];
      path = [ "/run/current-system/sw" ];
      unitConfig = {
        StartLimitIntervalSec = "1h";
        StartLimitBurst = 5;
      };
      serviceConfig = {
        Type = "exec";
        ExecStart = sync;
        Restart = "on-failure";
        RestartSec = 60;
      };
    };

    systemd.user.timers.kit-sync = lib.mkIf (tools ? kit) {
      description = "Run kit sync every hour";
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnCalendar = "hourly";
        Persistent = true;
        RandomizedDelaySec = "10m";
      };
    };
  };
}
