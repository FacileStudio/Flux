final: prev:

let
  entries = builtins.readDir ./.;
  dirs = builtins.filter (n: entries.${n} == "directory") (builtins.attrNames entries);
in
builtins.listToAttrs (map (name: { inherit name; value = final.callPackage ./${name} { }; }) dirs)
