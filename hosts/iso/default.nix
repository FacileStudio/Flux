{ lib, pkgs, modulesPath, ... }:

let
  version = lib.fileContents ../../version;
in
{
  imports = [ "${modulesPath}/installer/cd-dvd/installation-cd-base.nix" ];

  flux.profile = "desktop";
  system.stateVersion = lib.mkForce lib.trivial.release;

  image.baseName = lib.mkForce "flux-${version}-${pkgs.stdenv.hostPlatform.system}";
  isoImage.volumeID = lib.mkForce "FLUX_${lib.replaceStrings [ "." ] [ "_" ] version}";
  isoImage.squashfsCompression = "zstd -Xcompression-level 19";

  users.users.flux = {
    isNormalUser = true;
    extraGroups = [ "wheel" "networkmanager" "video" "audio" ];
    initialPassword = "flux";
  };
  services.getty.autologinUser = lib.mkForce "flux";

  services.greetd.settings.initial_session = {
    command = "niri-session";
    user = "flux";
  };

  networking.networkmanager.enable = true;
  networking.wireless.enable = lib.mkForce false;

  environment.systemPackages = [
    pkgs.flux-install
    (pkgs.makeDesktopItem {
      name = "flux-installer";
      desktopName = "Install Flux";
      comment = "Install Flux on this computer";
      exec = "flux-installer";
      icon = "system-software-install";
      categories = [ "System" ];
    })
  ];

  systemd.user.services.flux-installer = {
    description = "Flux installer";
    wantedBy = [ "graphical-session.target" ];
    partOf = [ "graphical-session.target" ];
    after = [ "graphical-session.target" ];
    path = [ "/run/wrappers" "/run/current-system/sw" ];
    serviceConfig = {
      ExecStart = "${pkgs.flux-shell}/bin/flux-installer";
      Restart = "no";
    };
  };
}
