{ ... }:

{
  users.users.flux = {
    isNormalUser = true;
    extraGroups = [ "wheel" "video" "audio" ];
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
    qemu.options = [ "-device virtio-vga-gl" "-display gtk,gl=on" ];
  };

  environment.sessionVariables.WLR_NO_HARDWARE_CURSORS = "1";
}
