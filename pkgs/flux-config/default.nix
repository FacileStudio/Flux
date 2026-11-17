{ lib, rustPlatform }:

let
  src = ../../shell;
  manifest = lib.importTOML (src + "/Cargo.toml");
in
rustPlatform.buildRustPackage {
  pname = "flux-config";
  inherit (manifest.workspace.package) version;
  inherit src;

  cargoLock.lockFile = src + "/Cargo.lock";
  cargoBuildFlags = [ "-p" "flux-config" ];
  cargoTestFlags = [ "-p" "flux-config" ];

  postInstall = ''
    install -Dm644 ${./studio.facile.flux.config.policy} \
      $out/share/polkit-1/actions/studio.facile.flux.config.policy
    substituteInPlace $out/share/polkit-1/actions/studio.facile.flux.config.policy \
      --replace-fail @flux-config@ $out/bin/flux-config
  '';

  meta = {
    description = "Root helper behind Flux Settings: settings.json, rebuilds, updates, rollback and passwords";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "flux-config";
  };
}
