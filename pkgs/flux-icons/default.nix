{ lib, stdenvNoCC, adwaita-icon-theme, hicolor-icon-theme }:

stdenvNoCC.mkDerivation {
  pname = "flux-icons";
  version = lib.fileContents ../../version;
  src = ./.;
  dontUnpack = true;
  dontDropIconThemeCache = true;

  propagatedBuildInputs = [ adwaita-icon-theme hicolor-icon-theme ];

  installPhase = ''
    runHook preInstall
    places() {
      cat <<EOF

    [scalable/places]
    Context=Places
    Size=64
    MinSize=16
    MaxSize=512
    Type=Scalable
    EOF
    }
    theme=$out/share/icons/Flux
    install -Dm644 -t $theme/symbolic/actions $src/icons/*.svg
    install -Dm644 -t $theme/scalable/places $src/places/*.svg
    {
      printf '[Icon Theme]\nName=Flux\nComment=The Solar icon set by 480 Design (CC BY 4.0) and Material Design Icons chevrons (Apache 2.0) for Flux\n'
      printf 'Inherits=Adwaita,hicolor\nDirectories=scalable/places,symbolic/actions\n'
      places
      printf '\n[symbolic/actions]\nContext=Actions\nSize=16\nMinSize=8\nMaxSize=512\nType=Scalable\n'
    } > $theme/index.theme
    dark=$out/share/icons/Flux-dark
    install -Dm644 -t $dark/scalable/places $src/places-dark/*.svg
    {
      printf '[Icon Theme]\nName=Flux-dark\nComment=Flux icons with folders drawn for dark mode\n'
      printf 'Inherits=Flux,Adwaita,hicolor\nDirectories=scalable/places\n'
      places
    } > $dark/index.theme
    runHook postInstall
  '';

  meta = {
    description = "Flux icon theme: Solar bold-duotone glyphs and Material Design Icons chevrons over Adwaita";
    license = [ lib.licenses.cc-by-40 lib.licenses.asl20 ];
    platforms = lib.platforms.linux;
  };
}
