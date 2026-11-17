{ config, lib, pkgs, ... }:

let
  locales = [
    "C.UTF-8"
    "en_US.UTF-8"
    "en_GB.UTF-8"
    "fr_FR.UTF-8"
    "fr_BE.UTF-8"
    "fr_CA.UTF-8"
    "fr_CH.UTF-8"
    "es_ES.UTF-8"
    "de_DE.UTF-8"
    "de_AT.UTF-8"
    "de_CH.UTF-8"
    "it_IT.UTF-8"
    "pt_PT.UTF-8"
    "pt_BR.UTF-8"
    "nl_NL.UTF-8"
  ];
  configured = [ config.i18n.defaultLocale ] ++ lib.attrValues config.i18n.extraLocaleSettings;
  entry = l: "${l}/${lib.last (lib.splitString "." l)}";
in
lib.mkIf (config.flux.profile == "desktop") {
  programs.niri.enable = true;
  hardware.enableRedistributableFirmware = lib.mkDefault true;
  environment.etc."niri/config.kdl".source = ./niri.kdl;

  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    pulse.enable = true;
    wireplumber.enable = true;
  };

  environment.systemPackages = with pkgs; [
    ghostty
    fuzzel
    wl-clipboard
    pulseaudio
    xwayland-satellite
    xdg-utils
    xdg-user-dirs
  ];

  systemd.user.services.flux-user-dirs = {
    description = "Create Documents, Downloads, Pictures and the other home folders";
    wantedBy = [ "graphical-session-pre.target" ];
    before = [ "graphical-session-pre.target" ];
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${pkgs.xdg-user-dirs}/bin/xdg-user-dirs-update";
    };
  };

  environment.sessionVariables.NIXOS_OZONE_WL = "1";
  services.gnome.at-spi2-core.enable = true;

  i18n.supportedLocales = lib.unique (map entry (locales ++ configured));
  console = {
    packages = [ pkgs.terminus_font ];
    font = lib.mkDefault "ter-v22n";
  };

  fonts.packages = with pkgs; [
    inter
    jetbrains-mono
    noto-fonts
    noto-fonts-cjk-sans
    noto-fonts-color-emoji
    nerd-fonts.symbols-only
  ];
  fonts.fontconfig = {
    antialias = true;
    hinting = {
      enable = true;
      style = "slight";
    };
    subpixel.rgba = "none";
    defaultFonts = {
      sansSerif = lib.mkAfter [ "Noto Sans" "Noto Sans CJK SC" ];
      serif = [ "Noto Serif" "Noto Serif CJK SC" ];
      monospace = lib.mkAfter [ "Noto Sans Mono" "Symbols Nerd Font Mono" ];
      emoji = [ "Noto Color Emoji" ];
    };
  };
}
