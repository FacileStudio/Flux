{ lib
, rustPlatform
, pkg-config
, wrapGAppsHook4
, gtk4
, gtk4-layer-shell
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
  buildInputs = [ gtk4 gtk4-layer-shell ];

  preFixup = ''
    gappsWrapperArgs+=(--suffix PATH : ${lib.makeBinPath [ mkpasswd ]})
  '';

  postInstall = ''
    cat > $out/bin/flux-wallpaper <<EOF
    #!${runtimeShell}
    exec $out/bin/sidebar wallpaper "\$@"
    EOF
    chmod +x $out/bin/flux-wallpaper
  '';

  meta = {
    description = "Flux desktop shell: statusbar, sidebar, launcher, exit menu and installer";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "statusbar";
  };
}
