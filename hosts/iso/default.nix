{ lib, pkgs, modulesPath, fluxFlake, ... }:

let
  version = lib.fileContents ../../version;
  system = pkgs.stdenv.hostPlatform.system;
  nixpkgs = fluxFlake.inputs.nixpkgs;

  fluxLocked =
    if fluxFlake ? rev then {
      type = "github";
      owner = "FacileStudio";
      repo = "Flux";
      inherit (fluxFlake) rev narHash lastModified;
    } else {
      type = "path";
      path = fluxFlake.outPath;
      inherit (fluxFlake) narHash lastModified;
    };

  installLock = pkgs.writeText "flake.lock" (builtins.toJSON {
    nodes = {
      flux = {
        inputs.nixpkgs = "nixpkgs";
        locked = fluxLocked;
        original = { type = "github"; owner = "FacileStudio"; repo = "Flux"; ref = "stable"; };
      };
      nixpkgs = (lib.importJSON ../../flake.lock).nodes.nixpkgs;
      root.inputs.flux = "flux";
    };
    root = "root";
    version = 7;
  });

  reference = nixpkgs.lib.nixosSystem {
    inherit system;
    modules = [
      fluxFlake.nixosModules.flux
      ../../template/configuration.nix
      ./reference-hardware.nix
    ];
  };

  referenceBuild = reference.config.system.build;

  localBuildInputs =
    map (drv: drv.inputDerivation) [ referenceBuild.toplevel referenceBuild.etc referenceBuild.initialRamdisk ]
    ++ (with reference.pkgs; [ stdenvNoCC lndir nukeReferences ckbcomp makeShellWrapper jq ])
    ++ lib.optionals pkgs.stdenv.hostPlatform.isx86 (with reference.pkgs; [ microcode-intel microcode-amd ]);
in
{
  imports = [ "${modulesPath}/installer/cd-dvd/installation-cd-base.nix" ];

  flux.profile = "desktop";
  system.stateVersion = lib.mkForce lib.trivial.release;

  image.baseName = lib.mkForce "flux-${version}-${pkgs.stdenv.hostPlatform.system}";
  isoImage.volumeID = lib.mkForce "FLUX_${lib.replaceStrings [ "." ] [ "_" ] version}";
  isoImage.squashfsCompression = "zstd -Xcompression-level 19";

  system.nixos.label = version;
  isoImage.prependToMenuLabel = "Try or install ";
  isoImage.appendToMenuLabel = "";
  isoImage.grubTheme = pkgs.flux-grub-theme;
  isoImage.efiSplashImage = "${pkgs.flux-grub-theme.splash}/efi.png";
  isoImage.splashImage = "${pkgs.flux-grub-theme.splash}/bios.png";
  isoImage.syslinuxTheme = ''
    MENU RESOLUTION 800 600
    MENU CLEAR
    MENU ROWS 6
    MENU VSHIFT 17
    MENU MARGIN 22
    MENU CMDLINEROW -4
    MENU TIMEOUTROW -3
    MENU TABMSGROW -2
    MENU HELPMSGROW -1
    MENU HELPMSGENDROW -1

    MENU COLOR BORDER       30;44   #00000000 #00000000 none
    MENU COLOR SCREEN       37;40   #ff0a0a0a #00000000 none
    MENU COLOR TITLE        1;36;44 #ff0a0a0a #00000000 none
    MENU COLOR UNSEL        37;44   #ff0a0a0a #00000000 none
    MENU COLOR SEL          7;37;40 #ffffffff #ff0a0a0a none
    MENU COLOR HOTKEY       37;44   #ff0a0a0a #00000000 none
    MENU COLOR HOTSEL       7;37;40 #ffffffff #ff0a0a0a none
    MENU COLOR TABMSG       31;40   #ff636363 #00000000 none
    MENU COLOR TIMEOUT      1;37;40 #ff636363 #00000000 none
    MENU COLOR TIMEOUT_MSG  37;40   #ff636363 #00000000 none
    MENU COLOR CMDMARK      1;36;40 #ff636363 #00000000 none
    MENU COLOR CMDLINE      37;40   #ff0a0a0a #00000000 none
    MENU COLOR HELP         37;40   #ff636363 #00000000 none
  '';
  specialisation.safe-graphics.configuration = {
    boot.kernelParams = [ "nomodeset" ];
    isoImage.configurationName = "(safe graphics)";
  };

  users.users.nixos.enable = false;
  services.getty.helpLine = lib.mkForce "";
  documentation.nixos.enable = lib.mkForce false;

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

  isoImage.storeContents = [
    fluxFlake.outPath
    nixpkgs.outPath
    referenceBuild.toplevel
  ] ++ localBuildInputs;

  system.build.fluxInstallLock = installLock;

  environment.systemPackages = [
    (pkgs.flux-install.override { fluxLock = installLock; })
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
