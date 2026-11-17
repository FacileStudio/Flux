{ lib, stdenvNoCC, adwaita-icon-theme, hicolor-icon-theme }:

stdenvNoCC.mkDerivation {
  pname = "flux-icons";
  version = lib.fileContents ../../version;
  src = ./icons;
  dontUnpack = true;
  dontDropIconThemeCache = true;

  propagatedBuildInputs = [ adwaita-icon-theme hicolor-icon-theme ];

  installPhase = ''
    runHook preInstall
    theme=$out/share/icons/Flux
    install -Dm644 -t $theme/symbolic/actions $src/*.svg
    cat > $theme/index.theme <<EOF
    [Icon Theme]
    Name=Flux
    Comment=The Solar icon set by 480 Design (CC BY 4.0) for Flux
    Inherits=Adwaita,hicolor
    Directories=symbolic/actions

    [symbolic/actions]
    Context=Actions
    Size=16
    MinSize=8
    MaxSize=512
    Type=Scalable
    EOF
    runHook postInstall
  '';

  meta = {
    description = "Flux icon theme: Solar bold-duotone glyphs over Adwaita";
    license = lib.licenses.cc-by-40;
    platforms = lib.platforms.linux;
  };
}
