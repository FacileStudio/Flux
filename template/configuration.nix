{ pkgs, ... }:

let
  hostName = "flux";
  userName = "flux";
  fullName = "Flux";
  hashedPassword = "!";
  timeZone = "UTC";
  keyMap = "us";
  locale = "en_US.UTF-8";
in
{
  networking.hostName = hostName;
  networking.networkmanager.enable = true;

  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  time.timeZone = timeZone;
  i18n.defaultLocale = locale;
  services.xserver.xkb.layout = keyMap;
  console.useXkbConfig = true;
  environment.sessionVariables.XKB_DEFAULT_LAYOUT = keyMap;

  users.users.${userName} = {
    isNormalUser = true;
    description = fullName;
    extraGroups = [ "wheel" "networkmanager" "video" "audio" ];
    shell = pkgs.zsh;
    inherit hashedPassword;
  };

  system.stateVersion = "25.11";
}
