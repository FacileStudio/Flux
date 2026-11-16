{ lib, stdenvNoCC, python3 }:

stdenvNoCC.mkDerivation {
  pname = "flux-theme";
  version = "0.1.0";

  dontUnpack = true;
  nativeBuildInputs = [ python3 ];

  installPhase = ''
    runHook preInstall
    python3 -I ${./generate.py} ${../../theme/tokens.css} $out/share
    runHook postInstall
  '';

  meta = {
    description = "muse design tokens turned into GTK, Ghostty and fuzzel themes for Flux";
    platforms = lib.platforms.linux;
  };
}
