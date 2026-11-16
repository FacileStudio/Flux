{ lib, stdenv, fetchurl, autoPatchelfHook, makeWrapper, symlinkJoin, typst }:

let
  system = stdenv.hostPlatform.system;
  lock = lib.importJSON ./tools.json;
  runtimeInputs = { ardoise = [ typst ]; };

  wrap = name: bin: lib.optionalString (runtimeInputs ? ${name})
    "wrapProgram $out/bin/${bin} --prefix PATH : ${lib.makeBinPath runtimeInputs.${name}}\n";

  mkTool = name: tool: stdenv.mkDerivation {
    pname = name;
    inherit (tool) version;
    src = fetchurl tool.systems.${system};
    sourceRoot = ".";

    nativeBuildInputs = [ autoPatchelfHook makeWrapper ];
    buildInputs = [ stdenv.cc.cc.lib ];

    dontConfigure = true;
    dontBuild = true;
    dontStrip = true;

    installPhase = ''
      runHook preInstall
      ${lib.concatMapStrings (bin: "install -Dm755 ${bin} $out/bin/${bin}\n" + wrap name bin) tool.bins}
      runHook postInstall
    '';

    meta = {
      homepage = "https://github.com/${tool.repo}";
      mainProgram = lib.head tool.bins;
      platforms = lib.attrNames tool.systems;
      sourceProvenance = [ lib.sourceTypes.binaryNativeCode ];
    };
  };

  tools = lib.mapAttrs mkTool (lib.filterAttrs (_: tool: tool.systems ? ${system}) lock);
in
symlinkJoin {
  name = "facile-tools";
  paths = lib.attrValues tools;
  passthru = { inherit tools; catalog = lib.attrNames lock; };
}
