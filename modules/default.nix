{ lib, ... }:

let
  entries = builtins.readDir ./.;
  isModule = n: entries.${n} == "regular" && lib.hasSuffix ".nix" n && n != "default.nix";
in
{
  imports = map (n: ./${n}) (builtins.filter isModule (builtins.attrNames entries));

  options.flux.profile = lib.mkOption {
    type = lib.types.enum [ "desktop" "headless" ];
    default = "desktop";
    description = "desktop ships niri and the graphical stack; headless is the shell and tools only, for Boite and servers.";
  };

  config = {
    nix.settings.experimental-features = [ "nix-command" "flakes" ];
    networking.hostName = lib.mkDefault "flux";
    system.stateVersion = lib.mkDefault "25.11";
  };
}
