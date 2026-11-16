{ config, lib, pkgs, ... }:

let
  cfg = config.flux;
  desktop = cfg.profile == "desktop";

  sessionService = description: command: {
    inherit description;
    partOf = [ "graphical-session.target" ];
    after = [ "graphical-session.target" ];
    wantedBy = [ "graphical-session.target" ];
    serviceConfig = {
      ExecStart = command;
      Restart = "on-failure";
      RestartSec = 2;
    };
  };
in
{
  options.flux = {
    flatpak.enable = lib.mkEnableOption "Flatpak with the Flathub remote added on first boot";
    scanning.enable = lib.mkEnableOption "Document Scanner. SANE with sane-airscan is already on through ipp-usb; users need the scanner and lp groups to scan";
  };

  config = lib.mkMerge [
    {
      nix.gc = {
        automatic = true;
        dates = "weekly";
        options = "--delete-older-than 30d";
        persistent = true;
      };
      nix.optimise.automatic = true;
      services.fstrim.enable = true;
      services.timesyncd.enable = lib.mkDefault true;
    }

    (lib.mkIf desktop {
      networking.networkmanager = {
        enable = true;
        wifi.backend = "wpa_supplicant";
        dns = "systemd-resolved";
      };
      services.resolved.enable = true;
      networking.firewall = {
        enable = true;
        allowPing = true;
        logRefusedConnections = false;
      };
      services.avahi = {
        enable = true;
        nssmdns4 = true;
        openFirewall = true;
      };

      security.polkit.enable = true;
      systemd.user.services.flux-polkit-agent = sessionService "Flux authentication agent"
        "${pkgs.polkit_gnome}/libexec/polkit-gnome-authentication-agent-1";

      services.gnome.gnome-keyring.enable = true;
      services.gnome.gcr-ssh-agent.enable = true;
      security.pam.services.greetd.enableGnomeKeyring = true;

      xdg.portal = {
        enable = true;
        config.niri = lib.mkForce {
          default = [ "gnome" "gtk" ];
          "org.freedesktop.impl.portal.Access" = [ "gtk" ];
          "org.freedesktop.impl.portal.FileChooser" = [ "gnome" ];
          "org.freedesktop.impl.portal.Notification" = [ "gtk" ];
          "org.freedesktop.impl.portal.ScreenCast" = [ "gnome" ];
          "org.freedesktop.impl.portal.Screenshot" = [ "gnome" ];
          "org.freedesktop.impl.portal.RemoteDesktop" = [ "gnome" ];
          "org.freedesktop.impl.portal.Settings" = [ "gnome" ];
          "org.freedesktop.impl.portal.Secret" = [ "gnome-keyring" ];
        };
      };

      programs.dconf.enable = true;
      services.gvfs.enable = true;
      services.udisks2.enable = true;
      systemd.user.services.flux-automount = sessionService "Flux removable media auto-mount"
        "${pkgs.udiskie}/bin/udiskie --automount --notify --no-tray";

      systemd.user.services.flux-clipboard = sessionService "Flux clipboard history"
        "${pkgs.wl-clipboard}/bin/wl-paste --watch ${pkgs.cliphist}/bin/cliphist store";

      services.fwupd.enable = true;
      services.upower.enable = true;

      services.printing = {
        enable = true;
        webInterface = true;
      };
      services.ipp-usb.enable = true;

      zramSwap = {
        enable = lib.mkDefault true;
        algorithm = "zstd";
        memoryPercent = 50;
      };
      systemd.oomd = {
        enable = true;
        enableRootSlice = true;
        enableUserSlices = true;
      };
    })

    (lib.mkIf (desktop && cfg.scanning.enable) {
      hardware.sane = {
        enable = true;
        extraBackends = [ pkgs.sane-airscan ];
      };
      environment.systemPackages = [ pkgs.simple-scan ];
    })

    (lib.mkIf (desktop && cfg.flatpak.enable) {
      services.flatpak.enable = true;
      systemd.services.flux-flathub = {
        description = "Add the Flathub remote";
        wantedBy = [ "multi-user.target" ];
        wants = [ "network-online.target" ];
        after = [ "network-online.target" ];
        path = [ config.services.flatpak.package ];
        serviceConfig = {
          Type = "oneshot";
          RemainAfterExit = true;
          Restart = "on-failure";
          RestartSec = 60;
        };
        script = ''
          flatpak remote-add --system --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
        '';
      };
    })
  ];
}
