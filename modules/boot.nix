{ config, lib, pkgs, ... }:

let
  version = lib.fileContents ../version;
  inherit (config.flux.theme) scheme;
  plymouth = pkgs.flux-plymouth.override { inherit scheme; };
  grub = config.boot.loader.grub;
  editenv = "${if grub.efiSupport then pkgs.grub2_efi else pkgs.grub2}/bin/grub-editenv /boot/grub/grubenv";
in
{
  config = lib.mkMerge [
    {
      system.nixos = {
        distroName = "Flux";
        distroId = "flux";
        vendorId = "facile-studio";
        vendorName = "Facile Studio";
        extraOSReleaseArgs = {
          PRETTY_NAME = "Flux ${version}";
          VERSION = version;
          HOME_URL = "https://flux.facile.studio";
          SUPPORT_URL = "https://github.com/FacileStudio/Flux";
          BUG_REPORT_URL = "https://github.com/FacileStudio/Flux/issues";
        };
      };

      services.getty.greetingLine = "Flux ${version} (\\m) - \\l";
      documentation.nixos.enable = lib.mkDefault false;
    }

    (lib.mkIf (config.flux.profile == "desktop") {
      boot.loader = {
        timeout = lib.mkDefault 2;
        efi.canTouchEfiVariables = lib.mkDefault true;
        grub = {
          enable = lib.mkDefault (!config.boot.loader.systemd-boot.enable);
          efiSupport = lib.mkDefault true;
          device = lib.mkDefault "nodev";
          theme = lib.mkDefault (pkgs.flux-grub-theme.override { inherit scheme; });
          splashImage = null;
          timeoutStyle = lib.mkDefault "hidden";
          configurationLimit = lib.mkDefault 10;
          extraConfig = ''
            if [ -s $prefix/grubenv ]; then
              if [ "$flux_boot" = started ]; then
                set timeout_style=menu
                set timeout=30
              fi
              set flux_boot=started
              save_env flux_boot
            fi
          '';
        };
      };

      systemd.services.flux-boot-mark = lib.mkIf grub.enable {
        description = "Show the boot menu after a start that did not end in a clean shutdown";
        wantedBy = [ "multi-user.target" ];
        unitConfig.RequiresMountsFor = "/boot";
        serviceConfig = {
          Type = "oneshot";
          RemainAfterExit = true;
          ExecStart = "${editenv} set flux_boot=started";
          ExecStop = "${editenv} unset flux_boot";
        };
      };

      boot.plymouth = {
        enable = lib.mkDefault true;
        theme = lib.mkDefault "flux";
        themePackages = [ plymouth ];
        logo = lib.mkDefault "${plymouth}/share/plymouth/themes/flux/mark.png";
        font = lib.mkDefault "${pkgs.inter}/share/fonts/truetype/InterVariable.ttf";
      };

      boot.consoleLogLevel = 0;
      boot.kernelParams = [ "quiet" "rd.udev.log_level=3" "udev.log_priority=3" "plymouth.use-simpledrm" ];
    })
  ];
}
