{ writeShellApplication, coreutils, util-linux, parted, dosfstools, e2fsprogs, gnused, mkpasswd, nixos-install-tools, systemd }:

writeShellApplication {
  name = "flux-install";
  runtimeInputs = [ coreutils util-linux parted dosfstools e2fsprogs gnused mkpasswd nixos-install-tools systemd ];
  runtimeEnv.FLUX_TEMPLATE = ../../template;
  text = builtins.readFile ./flux-install.sh;
}
