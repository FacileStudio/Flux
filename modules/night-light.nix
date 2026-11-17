{ config, lib, pkgs, ... }:

lib.mkIf (config.flux.profile == "desktop") {
  systemd.user.services.flux-night-light = {
    description = "Flux night light";
    partOf = [ "graphical-session.target" ];
    after = [ "graphical-session.target" ];
    wantedBy = [ "graphical-session.target" ];
    unitConfig.ConditionPathExists = "%h/.config/flux/night-light.env";
    serviceConfig = {
      EnvironmentFile = "-%h/.config/flux/night-light.env";
      ExecStart = "${pkgs.wlsunset}/bin/wlsunset $FLUX_NIGHT_LIGHT_ARGS";
      Restart = "on-failure";
      RestartSec = 2;
    };
  };
}
