{ lib, writeShellApplication, coreutils, util-linux, parted, dosfstools, e2fsprogs, gnused, gnugrep, mkpasswd
, nixos-install-tools, systemd, tzdata, curl, jq, lvm2, mdadm, nix, fluxLock ? null }:

writeShellApplication {
  name = "flux-install";
  runtimeInputs = [
    coreutils util-linux parted dosfstools e2fsprogs gnused gnugrep mkpasswd nixos-install-tools systemd
    curl jq lvm2 mdadm nix
  ];
  runtimeEnv = { FLUX_TEMPLATE = ../../template; } // lib.optionalAttrs (fluxLock != null) { FLUX_LOCK = fluxLock; };
  text = builtins.readFile ./flux-install.sh;
  derivationArgs.postCheck = ''
    PATH=${lib.getBin jq}/bin:$PATH TZDIR=${tzdata}/share/zoneinfo bash ${./test.sh} "$target"
  '';
}
