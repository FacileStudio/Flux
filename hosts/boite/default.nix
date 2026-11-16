{ lib, pkgs, modulesPath, ... }:

{
  imports = [ (modulesPath + "/profiles/qemu-guest.nix") ];

  flux.profile = "headless";

  networking.hostName = "boite";
  networking.useDHCP = lib.mkDefault true;

  boot.loader.grub = {
    enable = true;
    device = "/dev/vda";
  };
  boot.growPartition = true;
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
    autoResize = true;
  };

  image.modules.qemu = {
    image.efiSupport = false;
    image.format = "qcow2";
    virtualisation.diskSize = 8 * 1024;
  };

  users.groups.boite = { };
  users.users.boite = {
    isNormalUser = true;
    home = "/home/boite";
    group = "boite";
    extraGroups = [ "wheel" ];
    shell = pkgs.zsh;
  };

  security.sudo.wheelNeedsPassword = false;

  environment.systemPackages = with pkgs; [
    go
    cargo
    rustc
    rust-analyzer
    clippy
    rustfmt
    gcc
    gnumake
    pkg-config
    python3
    nodejs
    tmux
    htop
  ];

  services.openssh = {
    enable = true;
    ports = [ 22 ];
    settings = {
      PermitRootLogin = "prohibit-password";
      PasswordAuthentication = false;
    };
  };

  environment.interactiveShellInit = ''
    export WORKSPACE=/workspace
    if command -v tiroir >/dev/null 2>&1; then
      eval "$(tiroir export)"
    fi
  '';

  systemd.tmpfiles.rules = [
    "d /workspace 0755 boite boite -"
    "d /var/lib/boite 0755 root root -"
    "d /home/boite/.ssh 0700 boite boite -"
  ];

  systemd.services.boite-firstboot = {
    description = "Boite firstboot provisioning";
    wantedBy = [ "multi-user.target" ];
    before = [ "sshd.service" ];
    unitConfig.ConditionPathExists = "!/var/lib/boite/firstboot.done";
    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = false;
    };
    path = [ pkgs.util-linux pkgs.coreutils ];
    script = ''
      marker="/var/lib/boite/firstboot.done"
      if [ -f "$marker" ]; then
        exit 0
      fi

      mkdir -p /mnt/boitecfg
      blk=""
      label_dev="$(findfs LABEL=BOITECFG 2>/dev/null || true)"
      for attempt in 1 2 3 4 5 6; do
        for cand in "$label_dev" /dev/disk/by-label/BOITECFG /dev/vdb /dev/vdc /dev/sdb /dev/sdc /dev/sr0 /dev/sr1; do
          if [ -n "$cand" ] && [ -b "$cand" ] && mount -o ro "$cand" /mnt/boitecfg 2>/dev/null && [ -f /mnt/boitecfg/authorized_keys ]; then
            blk="$cand"
            break
          fi
          umount /mnt/boitecfg 2>/dev/null || true
        done
        if [ -n "$blk" ]; then
          break
        fi
        sleep 2
      done

      if [ -z "$blk" ] || [ ! -f /mnt/boitecfg/authorized_keys ]; then
        exit 1
      fi

      mkdir -p /home/boite/.ssh /var/lib/boite
      cp /mnt/boitecfg/authorized_keys /home/boite/.ssh/authorized_keys
      chown boite:boite /home/boite/.ssh/authorized_keys
      chmod 0600 /home/boite/.ssh/authorized_keys

      for f in /mnt/boitecfg/.tiroir*; do
        if [ -f "$f" ]; then
          fname="$(basename "$f")"
          cp "$f" "/home/boite/$fname"
          chown boite:boite "/home/boite/$fname"
          chmod 0600 "/home/boite/$fname"
        fi
      done

      umount /mnt/boitecfg
      touch "$marker"
    '';
  };
}
