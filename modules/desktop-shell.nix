{ config, lib, pkgs, ... }:

let
  shellService = description: command: {
    inherit description;
    partOf = [ "graphical-session.target" ];
    after = [ "graphical-session.target" ];
    wantedBy = [ "graphical-session.target" ];
    path = [ "/run/current-system/sw" ];
    serviceConfig = {
      ExecStart = command;
      Restart = "on-failure";
      RestartSec = 2;
    };
  };

  wallpaper = pkgs.writeShellScript "flux-wallpaper-start" ''
    choice="$HOME/.config/flux/wallpaper"
    image=""
    if [ -L "$choice" ]; then
      image=$(readlink -f "$choice")
    elif [ -f "$choice" ]; then
      IFS= read -r image < "$choice" || true
    fi
    if [ -z "$image" ] || [ ! -f "$image" ]; then
      image=${pkgs.flux-wallpapers}/share/backgrounds/flux/default.jpg
    fi
    exec ${pkgs.swaybg}/bin/swaybg --mode fill --image "$image"
  '';
in
lib.mkIf (config.flux.profile == "desktop") {
  environment.systemPackages = [
    pkgs.flux-shell
    pkgs.flux-wallpapers
    pkgs.adwaita-icon-theme
    pkgs.brightnessctl
    pkgs.playerctl
  ];
  services.udev.packages = [ pkgs.brightnessctl ];
  environment.pathsToLink = [ "/share/backgrounds" ];

  systemd.user.services.flux-statusbar =
    shellService "Flux status bar" "${pkgs.flux-shell}/bin/statusbar";

  systemd.user.services.flux-sidebar =
    shellService "Flux notification sidebar" "${pkgs.flux-shell}/bin/sidebar";

  systemd.user.services.flux-wallpaper =
    shellService "Flux wallpaper" "${wallpaper}";
}
