{ config, lib, pkgs, ... }:

let
  cfg = config.flux.power;
  shell = pkgs.flux-shell;
  idle = lib.escapeShellArgs ([
    "${shell}/bin/flux-idle"
    "run"
    "--swayidle"
    "${pkgs.swayidle}/bin/swayidle"
    "--lock"
    "${shell}/bin/flux-lock"
    "--niri"
    "${config.programs.niri.package}/bin/niri"
    "--inhibit"
    "${config.systemd.package}/bin/systemd-inhibit"
  ] ++ lib.optional (!cfg.lock) "--no-lock");
in
{
  options.flux.power.lock = lib.mkOption {
    type = lib.types.bool;
    default = true;
    description = "Lock the session with flux-lock when idle, on loginctl lock-session and before sleep.";
  };

  config = lib.mkIf (config.flux.profile == "desktop") {
    services.upower.enable = true;
    services.power-profiles-daemon.enable = true;
    powerManagement.enable = true;

    security.pam.services.flux-lock = { };

    services.logind.settings.Login = {
      HandleLidSwitch = lib.mkDefault "suspend";
      HandleLidSwitchExternalPower = lib.mkDefault "suspend";
      HandleLidSwitchDocked = lib.mkDefault "ignore";
    };

    systemd.user.services.flux-idle = {
      description = "Flux idle, lock and sleep rules";
      partOf = [ "graphical-session.target" ];
      after = [ "graphical-session.target" ];
      wantedBy = [ "graphical-session.target" ];
      serviceConfig = {
        ExecStart = idle;
        Restart = "on-failure";
        RestartSec = 2;
        KillMode = "process";
      };
    };
  };
}
