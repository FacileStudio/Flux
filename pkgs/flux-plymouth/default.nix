{ lib
, stdenvNoCC
, librsvg
, flux-theme
, scheme ? "light"
}:

let
  palette = "${flux-theme}/share/flux/palette-${scheme}.css";
in
stdenvNoCC.mkDerivation {
  pname = "flux-plymouth-${scheme}";
  version = lib.fileContents ../../version;

  dontUnpack = true;
  nativeBuildInputs = [ librsvg ];

  installPhase = ''
    runHook preInstall

    color() { sed -n "s/^@define-color fc_$1 \(#[0-9a-f]*\);$/\1/p" ${palette}; }
    floats() {
      [ -n "$1" ] || { echo "missing color in ${palette}" >&2; exit 1; }
      printf '%d %d %d\n' 0x''${1:1:2} 0x''${1:3:2} 0x''${1:5:2} \
        | awk '{ printf "%.4f, %.4f, %.4f", $1 / 255, $2 / 255, $3 / 255 }'
    }
    page=$(color page)
    fg=$(color fg)
    muted=$(color fg_muted)
    field=$(color field)

    dir=$out/share/plymouth/themes/flux
    mkdir -p $dir

    sed "s/currentColor/$fg/g" ${../../site/static/flux-mark.svg} > mark.svg
    shape() {
      echo "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"$2\" height=\"$3\">$4</svg>" > $1.svg
      rsvg-convert -o $dir/$1.png $1.svg
      rsvg-convert -z 2 -o $dir/$1@2x.png $1.svg
    }
    rsvg-convert -w 112 -h 112 -o $dir/mark.png mark.svg
    rsvg-convert -w 224 -h 224 -o $dir/mark@2x.png mark.svg
    shape dot 8 8 "<circle cx=\"4\" cy=\"4\" r=\"4\" fill=\"$muted\"/>"
    shape bullet 8 8 "<circle cx=\"4\" cy=\"4\" r=\"4\" fill=\"$fg\"/>"
    shape field 288 44 "<rect width=\"288\" height=\"44\" rx=\"22\" fill=\"$field\"/>"

    substitute ${./flux.script} $dir/flux.script \
      --replace-fail @page@ "$(floats $page)" \
      --replace-fail @fg@ "$(floats $fg)" \
      --replace-fail @muted@ "$(floats $muted)"

    cat > $dir/flux.plymouth <<EOF
    [Plymouth Theme]
    Name=Flux
    Description=The Flux mark on the muse page
    ModuleName=script

    [script]
    ImageDir=$dir
    ScriptFile=$dir/flux.script
    EOF

    runHook postInstall
  '';

  passthru = { inherit scheme; };

  meta = {
    description = "Plymouth theme for Flux: the Flux mark on the muse page with a quiet spinner";
    platforms = lib.platforms.linux;
  };
}
