#!/usr/bin/env bash
set -euo pipefail

root="$(git -C "$(dirname "$0")/.." rev-parse --show-toplevel)"
name="$(basename "$root")"
exec docker run --rm \
  -v flux-dev:/nix \
  -v flux-cargo:/cargo \
  -v "$root":/w \
  -w /w/shell \
  -e CARGO_HOME=/cargo \
  -e CARGO_TARGET_DIR="/cargo/target-$name" \
  -e CARGO_INCREMENTAL=0 \
  -e CARGO_PROFILE_DEV_DEBUG=0 \
  -e CARGO_PROFILE_TEST_DEBUG=0 \
  nixos/nix nix --extra-experimental-features "nix-command flakes" develop path:/w -c "$@"
