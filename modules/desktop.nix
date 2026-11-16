{ config, lib, pkgs, ... }:

lib.mkIf (config.flux.profile == "desktop") {
  programs.niri.enable = true;
  environment.etc."niri/config.kdl".source = ./niri.kdl;

  services.greetd = {
    enable = true;
    settings.default_session.command =
      "${lib.getExe pkgs.tuigreet} --time --remember --cmd niri-session";
  };

  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    pulse.enable = true;
  };

  environment.systemPackages = with pkgs; [
    ghostty
    fuzzel
    wl-clipboard
    pulseaudio
    xwayland-satellite
  ];

  fonts.packages = with pkgs; [ inter jetbrains-mono noto-fonts noto-fonts-color-emoji ];
}
