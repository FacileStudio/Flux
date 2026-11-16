{ lib
, rustPlatform
, pkg-config
, wrapGAppsHook4
, gtk4
, gtk4-layer-shell
, librsvg
, gsettings-desktop-schemas
, runtimeShell
, mkpasswd
}:

let
  src = ../../shell;
  manifest = lib.importTOML (src + "/Cargo.toml");
in
rustPlatform.buildRustPackage {
  pname = "flux-shell";
  inherit (manifest.workspace.package) version;
  inherit src;

  cargoLock.lockFile = src + "/Cargo.lock";

  nativeBuildInputs = [ pkg-config wrapGAppsHook4 ];
  buildInputs = [ gtk4 gtk4-layer-shell librsvg gsettings-desktop-schemas ];

  preFixup = ''
    gappsWrapperArgs+=(--suffix PATH : ${lib.makeBinPath [ mkpasswd ]})
  '';

  postInstall = ''
    cat > $out/bin/flux-wallpaper <<EOF
    #!${runtimeShell}
    exec $out/bin/sidebar wallpaper "\$@"
    EOF
    chmod +x $out/bin/flux-wallpaper

    shopt -s nullglob
    for entry in crates/*/data/*.desktop; do
      install -Dm644 -t $out/share/applications "$entry"
    done
    for icon in crates/*/data/icons/*.svg; do
      install -Dm644 -t $out/share/icons/hicolor/scalable/apps "$icon"
    done
    shopt -u nullglob
  '';

  meta = {
    description = "Flux desktop shell: statusbar, sidebar, launcher, exit menu, installer and settings";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "statusbar";
  };
}
