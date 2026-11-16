{ pkgs, ... }:

let
  hostName = "flux";
  userName = "flux";
  hashedPassword = "!";
in
{
  networking.hostName = hostName;
  networking.networkmanager.enable = true;

  users.users.${userName} = {
    isNormalUser = true;
    extraGroups = [ "wheel" "networkmanager" "video" "audio" ];
    shell = pkgs.zsh;
    inherit hashedPassword;
  };

  system.stateVersion = "25.11";
}
