#!/usr/bin/env bash
# Run the backend test against a mock NetworkManager on a private session bus.
# From the repo root: scripts/dev.sh bash crates/settings-network/tests/mock-nm.sh
set -euo pipefail

case "${1:-}" in
  "")
    tools='with (builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem};
      buildEnv { name = "nm-mock"; paths = [ dbus (python3.withPackages (p: [ p.python-dbusmock ])) ]; }'
    exec nix --extra-experimental-features "nix-command flakes" shell --impure \
      --expr "$tools" -c bash "$0" --bus
    ;;
  --bus)
    config="$(dirname "$(readlink -f "$(command -v dbus-daemon)")")/../share/dbus-1/session.conf"
    exec dbus-run-session --config-file="$config" -- bash "$0" --inside
    ;;
esac

python3 -m dbusmock --session --template networkmanager \
  --parameters '{"WwanEnabled": true}' &
mock=$!
trap 'kill "$mock" 2>/dev/null || true' EXIT
FLUX_NM_BUS=session cargo test -p settings-network --test mock -- --ignored --nocapture
