{ writeShellApplication, coreutils, util-linux, parted, dosfstools, e2fsprogs, gnused, mkpasswd, nixos-install-tools, systemd, tzdata }:

writeShellApplication {
  name = "flux-install";
  runtimeInputs = [ coreutils util-linux parted dosfstools e2fsprogs gnused mkpasswd nixos-install-tools systemd ];
  runtimeEnv.FLUX_TEMPLATE = ../../template;
  text = builtins.readFile ./flux-install.sh;
  derivationArgs.postCheck = ''
    TZDIR=${tzdata}/share/zoneinfo bash ${./test.sh} "$target"
  '';
}
