{ pkgs, ... }:

{
  users.users.flux = {
    isNormalUser = true;
    extraGroups = [ "wheel" "networkmanager" "video" "audio" ];
    initialPassword = "flux";
  };
  security.sudo.wheelNeedsPassword = false;

  boot.loader.grub.enable = false;
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
  };

  virtualisation.vmVariant.virtualisation = {
    memorySize = 4096;
    cores = 4;
    qemu.options = [
      (if pkgs.stdenv.hostPlatform.isAarch64 then "-device virtio-gpu-gl-pci" else "-device virtio-vga-gl")
      "-display gtk,gl=on"
    ];
  };

  flux.niri.platformConfig = ''
    debug {
        disable-cursor-plane
    }
  '';
}
