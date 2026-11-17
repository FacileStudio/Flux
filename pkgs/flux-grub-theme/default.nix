{ lib
, stdenvNoCC
, buildPackages
, librsvg
, inter
, flux-theme
, scheme ? "dark"
}:

let
  palette = "${flux-theme}/share/flux/palette-${scheme}.css";
  radius = 24;
  height = 2 * radius;
  r = toString radius;
  h = toString height;
in
stdenvNoCC.mkDerivation {
  pname = "flux-grub-theme-${scheme}";
  version = lib.fileContents ../../version;

  outputs = [ "out" "splash" ];

  dontUnpack = true;
  nativeBuildInputs = [ buildPackages.grub2 librsvg ];

  installPhase = ''
    runHook preInstall

    color() { sed -n "s/^@define-color fc_$1 \(#[0-9a-f]*\);$/\1/p" ${palette}; }
    page=$(color page)
    fg=$(color fg)
    muted=$(color fg_muted)
    accent=$(color accent)
    accentFg=$(color accent_fg)
    for c in "$page" "$fg" "$muted" "$accent" "$accentFg"; do
      [ -n "$c" ] || { echo "missing color in ${palette}" >&2; exit 1; }
    done

    mkdir -p $out $splash
    sed "s/currentColor/$fg/g" ${../../site/static/flux-mark.svg} > mark.svg
    rsvg-convert -w 80 -h 80 -o $out/mark.png mark.svg

    pill() {
      echo "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"$2\" height=\"${h}\">$3</svg>" \
        | rsvg-convert -o $out/select_$1.png /dev/stdin
    }
    pill w ${r} "<circle cx=\"${r}\" cy=\"${r}\" r=\"${r}\" fill=\"$accent\"/>"
    pill e ${r} "<circle cx=\"0\" cy=\"${r}\" r=\"${r}\" fill=\"$accent\"/>"
    pill c 8 "<rect width=\"8\" height=\"${h}\" fill=\"$accent\"/>"

    ranges=0x20-0x7e,0xa0-0x17f,0x2010-0x2027,0x2190-0x2193,0x2212-0x2212
    font=${inter}/share/fonts/truetype/Inter.ttc
    grub-mkfont -i 0 -s 16 -r $ranges -o $out/inter-regular-16.pf2 $font
    grub-mkfont -i 0 -s 20 -r $ranges -o $out/inter-regular-20.pf2 $font
    grub-mkfont -i 10 -n "Inter Medium" -s 20 -r $ranges -o $out/inter-medium-20.pf2 $font

    cat > $out/theme.txt <<EOF
    title-text: ""
    desktop-color: "$page"
    message-font: "Inter Regular 16"
    message-color: "$muted"
    message-bg-color: "$page"

    + image {
      left = 50%-40
      top = 50%-216
      width = 80
      height = 80
      file = "mark.png"
    }

    + boot_menu {
      left = 50%-320
      top = 50%-104
      width = 640
      height = 336
      item_font = "Inter Regular 20"
      selected_item_font = "Inter Medium Regular 20"
      item_color = "$fg"
      selected_item_color = "$accentFg"
      item_height = ${h}
      item_spacing = 8
      item_padding = 0
      icon_width = 0
      icon_height = 0
      item_icon_space = 0
      selected_item_pixmap_style = "select_*.png"
      scrollbar = false
    }

    + label {
      id = "__timeout__"
      left = 0
      top = 100%-112
      width = 100%
      height = 24
      align = "center"
      font = "Inter Regular 16"
      color = "$muted"
      text = "Starting in %d s"
    }

    + label {
      left = 0
      top = 100%-72
      width = 100%
      height = 24
      align = "center"
      font = "Inter Regular 16"
      color = "$muted"
      text = "↑ ↓ to choose   ·   Enter to start   ·   E to edit   ·   C for a console"
    }
    EOF

    splash() {
      cat > $1.svg <<EOF
    <svg xmlns="http://www.w3.org/2000/svg" width="$2" height="$3"><rect width="$2" height="$3" fill="$page"/><image x="$(( ($2 - $4) / 2 ))" y="$(( $3 / 3 - $4 / 2 ))" width="$4" height="$4" href="mark.svg"/></svg>
    EOF
      rsvg-convert -o $splash/$1.png $1.svg
    }
    splash efi 1920 1080 240
    splash bios 800 600 112

    runHook postInstall
  '';

  passthru = { inherit scheme; };

  meta = {
    description = "GRUB theme for Flux: the muse page, the Flux mark and Inter";
    platforms = lib.platforms.linux;
  };
}
