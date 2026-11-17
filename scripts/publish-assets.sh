#!/usr/bin/env bash
set -euo pipefail

tag="${1:?usage: publish-assets.sh <tag> [assets-dir]}"
root="${2:-/etc/dokploy/flux-assets}"
repo="FacileStudio/Flux"
base_url="${FLUX_BASE_URL:-https://flux.facile.studio/releases}"
needed_kib=$((20 * 1024 * 1024))
root=$(realpath -m -- "$root")
parent=$(dirname "$root")
target="$root/$tag"
staging="$parent/.flux-assets-staging"
partial="$staging/$tag"

die() {
  echo "publish-assets: $*" >&2
  exit 1
}

need_space() {
  local dir=$1 free
  free=$(df --output=avail -k "$dir" | tail -1)
  [ "$free" -ge "$needed_kib" ] || die "less than 20 GiB free under $dir; not downloading"
}

move_stable() {
  local commit
  commit=$(gh api "repos/$repo/commits/$tag" -q .sha)
  if ! gh api "repos/$repo/git/ref/heads/stable" > /dev/null 2>&1; then
    gh api -X POST "repos/$repo/git/refs" -f ref=refs/heads/stable -f sha="$commit" > /dev/null
    echo "created stable at $tag"
    return
  fi
  case "$(gh api "repos/$repo/compare/stable...$commit" -q .status)" in
    identical) echo "stable is already at $tag" ;;
    ahead)
      gh api -X PATCH "repos/$repo/git/refs/heads/stable" -f sha="$commit" -F force=false > /dev/null
      echo "stable moved to $tag: installed machines now update to it"
      ;;
    behind) echo "stable is already past $tag; left where it is" ;;
    *) die "stable and $tag have diverged; move stable by hand" ;;
  esac
}

case "$tag" in
  v[0-9]*) ;;
  *) die "$tag is not a v<version> tag" ;;
esac
[[ "$tag" =~ ^v[0-9A-Za-z.+-]+$ ]] || die "$tag contains characters a release tag cannot have"
[ -d "$root" ] || die "$root does not exist"

mkdir -p "$staging"
exec 9>"$staging/.lock"
flock -n 9 || die "another publish-assets.sh run holds $staging/.lock"

sums=$(mktemp -d)
trap 'rm -rf "$sums" "$partial" "$staging/tmp"' EXIT
gh release download "$tag" -R "$repo" -p 'SHA256SUMS*' -D "$sums"
[ -f "$sums/SHA256SUMS" ] || die "release $tag has no SHA256SUMS"

if [ -e "$target" ]; then
  (cd "$target" && sha256sum --quiet --check "$sums/SHA256SUMS") \
    || die "$target already exists and does not match the release SHA256SUMS; refusing to overwrite a published release"
  echo "$target is already published and matches SHA256SUMS; checking the public URLs"
else
  run=$(gh run list -R "$repo" --workflow release.yml --branch "$tag" --status success --limit 1 --json databaseId -q '.[0].databaseId')
  [ -n "$run" ] || die "no successful release run for $tag"

  export TMPDIR="$staging/tmp"
  mkdir -p "$TMPDIR"
  need_space "$root"
  need_space "$TMPDIR"

  rm -rf "$partial"
  mkdir -p "$partial"
  gh run download "$run" -R "$repo" -D "$partial/artifacts"
  find "$partial/artifacts" -type f \( -name '*.iso' -o -name '*.qcow2' \) -exec mv {} "$partial/" \;
  rm -rf "$partial/artifacts"
  cp "$sums"/SHA256SUMS* "$partial/"

  (cd "$partial" && sha256sum --check SHA256SUMS)
  chmod 0755 "$partial"
  chmod 0644 "$partial"/*
  mv -T "$partial" "$target"
  echo "published $tag:"
  ls -la "$target"
fi

failed=0
for file in "$target"/*; do
  name=$(basename "$file")
  url="$base_url/$tag/$name"
  size=$(stat -c %s "$file")
  read -r code length < <(curl -sS -o /dev/null -I -w '%{http_code} %header{content-length}\n' --retry 3 --max-time 60 "$url" || echo "000 0")
  if [ "$code" = 200 ] && [ "${length:-}" = "$size" ]; then
    echo "ok   $url ($size bytes)"
  elif [ "$code" = 200 ] && [ -z "${length:-}" ]; then
    echo "ok   $url (no Content-Length to compare)"
  else
    echo "FAIL $url answered $code with length ${length:-none}, expected 200 with $size" >&2
    failed=1
  fi
done
[ "$failed" -eq 0 ] || die "some public URLs do not serve $tag; the release stays a draft"

gh release edit "$tag" -R "$repo" --draft=false
echo "release $tag is public: https://github.com/$repo/releases/tag/$tag"
move_stable
