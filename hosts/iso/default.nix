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

  environment.systemPackages = [ pkgs.flux-install ];
}
