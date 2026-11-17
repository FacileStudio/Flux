{ config, lib, pkgs, ... }:

let
  cfg = config.flux;
  x86 = pkgs.stdenv.hostPlatform.isx86;
  niriProfile = {
    rules = [{
      pattern = { feature = "procname"; matches = "niri"; };
      profile = "Limit Free Buffer Pool On Wayland Compositors";
    }];
    profiles = [{
      name = "Limit Free Buffer Pool On Wayland Compositors";
      settings = [{ key = "GLVidHeapReuseRatio"; value = 0; }];
    }];
  };
in
{
  options.flux = {
    hardware.nvidia = {
      enable = lib.mkEnableOption "the proprietary NVIDIA driver with the settings niri needs";
      open = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Use NVIDIA's open kernel module. Needs a Turing (GTX 16xx, RTX 20xx) or newer GPU; set false for older cards.";
      };
    };
    cursor = {
      theme = lib.mkOption {
        type = lib.types.str;
        default = "Adwaita";
        description = "XCursor theme name, from a package in environment.systemPackages.";
      };
      size = lib.mkOption {
        type = lib.types.ints.between 16 128;
        default = 24;
        description = "Cursor size in logical pixels. 32 or 48 help on large or distant screens.";
      };
    };
    niri.platformConfig = lib.mkOption {
      type = lib.types.lines;
      default = "";
      description = "KDL written to /etc/flux/niri/platform.kdl, which niri.kdl includes. Hosts add debug flags here.";
    };
  };

  config = lib.mkIf (cfg.profile == "desktop") (lib.mkMerge [
    {
      programs.niri.package = pkgs.niri.overrideAttrs (old: {
        patches = (old.patches or [ ]) ++ [ ./patches/niri-software-rendering.patch ];
      });

      hardware.graphics = {
        enable = true;
        enable32Bit = lib.mkDefault false;
        extraPackages = lib.optional x86 pkgs.intel-media-driver;
      };
      hardware.enableRedistributableFirmware = lib.mkDefault true;
      hardware.cpu.intel.updateMicrocode = lib.mkIf x86 (lib.mkDefault true);
      hardware.cpu.amd.updateMicrocode = lib.mkIf x86 (lib.mkDefault true);

      hardware.bluetooth = {
        enable = true;
        powerOnBoot = true;
        settings.General = {
          Experimental = true;
          FastConnectable = true;
        };
      };

      flux.niri.platformConfig = lib.mkBefore ''
        cursor {
            xcursor-theme "${cfg.cursor.theme}"
            xcursor-size ${toString cfg.cursor.size}
        }
      '';
      environment.etc."flux/niri/platform.kdl".text = cfg.niri.platformConfig;

      environment.sessionVariables = {
        XCURSOR_THEME = cfg.cursor.theme;
        XCURSOR_SIZE = toString cfg.cursor.size;
      };
      programs.dconf.profiles.user.databases = [{
        settings."org/gnome/desktop/interface" = {
          cursor-theme = cfg.cursor.theme;
          cursor-size = lib.gvariant.mkInt32 cfg.cursor.size;
        };
      }];
    }

    (lib.mkIf cfg.hardware.nvidia.enable {
      flux.unfree = [ "nvidia-x11" "nvidia-settings" "nvidia-persistenced" ];
      services.xserver.videoDrivers = [ "nvidia" ];
      hardware.nvidia = {
        modesetting.enable = true;
        open = cfg.hardware.nvidia.open;
        nvidiaSettings = false;
        powerManagement.enable = true;
        package = config.boot.kernelPackages.nvidiaPackages.stable;
      };
      boot.kernelParams = [ "nvidia-drm.modeset=1" "nvidia-drm.fbdev=1" ];
      environment.etc."nvidia/nvidia-application-profiles-rc.d/50-limit-free-buffer-pool-in-wayland-compositors.json".text =
        builtins.toJSON niriProfile;
    })
  ]);
}
