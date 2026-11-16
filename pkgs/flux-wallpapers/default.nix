{ lib, stdenvNoCC }:

stdenvNoCC.mkDerivation {
  pname = "flux-wallpapers";
  version = "0.1.0";
  src = ./images;
  dontUnpack = true;
  installPhase = ''
    runHook preInstall
    mkdir -p $out/share/backgrounds/flux
    cp $src/*.jpg $src/CREDITS.md $out/share/backgrounds/flux/
    ln -s monstera.jpg $out/share/backgrounds/flux/default.jpg
    runHook postInstall
  '';
  meta = {
    description = "Flux default wallpapers";
    platforms = lib.platforms.linux;
  };
}
