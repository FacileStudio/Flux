{ config, lib, pkgs, ... }:

lib.mkIf (config.flux.profile == "desktop" && config.flux.software.gaming) (lib.mkMerge [
  {
    assertions = [{
      assertion = pkgs.stdenv.hostPlatform.isx86_64;
      message = "flux.software.gaming needs an Intel or AMD (x86_64) machine: Steam has no ARM64 version. Set it to false on this machine.";
    }];
  }

  (lib.mkIf pkgs.stdenv.hostPlatform.isx86_64 {
    flux.unfree = [ "steam" "steam-unwrapped" "steam-original" "steam-run" ];

    programs.steam = {
      enable = true;
      extest.enable = true;
    };
    programs.gamemode.enable = true;
    programs.gamescope.enable = true;
    environment.systemPackages = [ pkgs.mangohud ];
    boot.kernelModules = [ "ntsync" ];

    flux.niri.platformConfig = ''
      window-rule {
          match app-id="steam" title=r#"^notificationtoasts_\d+_desktop$"#
          default-floating-position x=10 y=10 relative-to="bottom-right"
      }
    '';
  })
])
