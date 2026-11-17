#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/../../../.." && pwd)"
exec nix --extra-experimental-features "nix-command flakes" shell --impure --expr "
  let pkgs = (builtins.getFlake \"path:$root\").inputs.nixpkgs.legacyPackages.\${builtins.currentSystem};
  in { dbus = pkgs.dbus; python = pkgs.python3.withPackages (p: [ p.python-dbusmock ]); }
" dbus python -c cargo test -p settings-bluetooth --test bluez_mock -- --nocapture
