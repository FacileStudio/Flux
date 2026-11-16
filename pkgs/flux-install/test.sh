#!/usr/bin/env bash
set -euo pipefail

bin=$1
salt=abcdefghijklmnop
hash="\$6\$$salt\$$(printf 'a%.0s' $(seq 86))"
disk=$(lsblk -dnpo PATH,TYPE 2>/dev/null | while read -r path type; do
  case "$path" in /dev/zram*|/dev/loop*) continue ;; esac
  if [ "$type" = disk ]; then printf '%s' "$path"; break; fi
done || true)
failures=0

run() {
  local args=(--disk "${disk:-/dev/nonexistent}" --hostname flux --full-name "Ada Lovelace" --user ada
    --password-hash "$hash" --timezone UTC --keymap fr --locale fr_FR.UTF-8 --yes --dry-run --progress json)
  local key value i
  while [ "$#" -gt 1 ]; do
    key=$1 value=$2
    shift 2
    for i in "${!args[@]}"; do
      [ "${args[$i]}" = "$key" ] || continue
      case "$value" in
        DROP) unset 'args[i]' 'args[i+1]' ;;
        DROPFLAG) unset 'args[i]' ;;
        *) args[i+1]=$value ;;
      esac
      break
    done
  done
  "$bin" "${args[@]}"
}

rejects() {
  local name=$1 output
  shift
  if output=$(run "$@" 2>/dev/null); then
    printf 'FAIL %s: accepted\n' "$name"
    failures=$((failures + 1))
  elif [ "$1" != --progress ] && ! printf '%s' "$output" | grep -q '^{"step":"error","message":".*"}$'; then
    printf 'FAIL %s: no error line, got: %s\n' "$name" "$output"
    failures=$((failures + 1))
  else
    printf 'ok   %s\n' "$name"
  fi
}

rejects "missing disk" --disk DROP
rejects "partition-like disk" --disk /dev/nonexistent1
rejects "uppercase user" --user Ada
rejects "user with a space" --user "ada l"
rejects "root user" --user root
rejects "hostname with a leading dash" --hostname -flux
rejects "hostname with a dot" --hostname flux.local
rejects "hostname too long" --hostname "$(printf 'a%.0s' $(seq 64))"
rejects "unknown time zone" --timezone Mars/Olympus
rejects "time zone path escape" --timezone ../../etc/passwd
rejects "unknown keymap" --keymap zz
rejects "unknown locale" --locale xx_XX.UTF-8
rejects "plain password" --password-hash hunter2
rejects "md5 hash" --password-hash "\$1\$abc\$def"
rejects "quote in the full name" --full-name 'Ada "the" Lovelace'
rejects "dollar in the full name" --full-name "Ada \${pkgs}"
rejects "missing yes" --yes DROPFLAG
rejects "bad progress mode" --progress xml

if [ -n "$disk" ]; then
  output=$(run)
  if printf '%s' "$output" | grep -qx "disk=$disk" && printf '%s' "$output" | grep -qx 'user=ada'; then
    printf 'ok   valid plan on %s\n' "$disk"
  else
    printf 'FAIL valid plan: %s\n' "$output"
    failures=$((failures + 1))
  fi
else
  printf 'skip valid plan: no whole disk visible\n'
fi

[ "$failures" -eq 0 ]
