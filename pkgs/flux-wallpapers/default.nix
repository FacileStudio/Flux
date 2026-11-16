{ lib, stdenvNoCC }:

stdenvNoCC.mkDerivation {
  pname = "flux-wallpapers";
  version = "0.1.0";
  src = ./images;
  dontUnpack = true;
  installPhase = ''
    runHook preInstall
    mkdir -p $out/share/backgrounds/flux
    cp $src/*.jpg $out/share/backgrounds/flux/
    ln -s sunny-grass-hills-alt.jpg $out/share/backgrounds/flux/default.jpg
    runHook postInstall
  '';
  meta = {
    description = "Flux default wallpapers";
    platforms = lib.platforms.linux;
  };
}
