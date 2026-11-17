{ config, lib, pkgs, ... }:

lib.mkIf (config.flux.profile == "desktop" && config.flux.software.gaming) {
  flux.unfree = [ "steam" "steam-unwrapped" "steam-original" "steam-run" ];

  programs.steam = {
    enable = true;
    remotePlay.openFirewall = true;
    localNetworkGameTransfers.openFirewall = true;
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
}
