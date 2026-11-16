#!/usr/bin/env bash
set -euo pipefail

tag="${1:?usage: publish-assets.sh <tag> [assets-dir]}"
root="${2:-/etc/dokploy/flux-assets}"
repo="FacileStudio/Flux"
target="$root/$tag"
partial="$root/.$tag.partial"

if [ -e "$target" ]; then
  echo "$target already exists; refusing to overwrite a published release" >&2
  exit 1
fi

run=$(gh run list -R "$repo" --workflow release.yml --branch "$tag" --status success --limit 1 --json databaseId -q '.[0].databaseId')
if [ -z "$run" ]; then
  echo "no successful release run for $tag" >&2
  exit 1
fi

needed=$((8 * 1024 * 1024))
free=$(df --output=avail -k "$root" | tail -1)
if [ "$free" -lt "$needed" ]; then
  echo "less than 8 GiB free under $root; not downloading" >&2
  exit 1
fi

rm -rf "$partial"
mkdir -p "$partial"
trap 'rm -rf "$partial"' EXIT

gh run download "$run" -R "$repo" -D "$partial/artifacts"
find "$partial/artifacts" -type f \( -name '*.iso' -o -name '*.qcow2' \) -exec mv {} "$partial/" \;
rm -rf "$partial/artifacts"
gh release download "$tag" -R "$repo" -p SHA256SUMS -D "$partial"

(cd "$partial" && sha256sum --check SHA256SUMS)
chmod 0644 "$partial"/*
mv "$partial" "$target"
trap - EXIT
echo "published $tag:"
ls -la "$target"
