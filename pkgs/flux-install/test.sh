#!/usr/bin/env bash
set -euo pipefail

bin=$1
salt=abcdefghijklmnop
hash="\$6\$$salt\$$(printf 'a%.0s' $(seq 86))"
yescrypt="\$y\$j9T\$abcdefghijklmnopqrstuv\$$(printf 'b%.0s' $(seq 43))"
scratch=$(mktemp -d)
ln -s /dev/sda "$scratch/ata-FAKE_SSD_123"
failures=0

lsblk() {
  case "$*" in
    "-dnpo PATH,TYPE") printf '/dev/sda disk\n/dev/sdb disk\n/dev/sdc disk\n/dev/sdd disk\n/dev/sr0 rom\n/dev/loop0 loop\n' ;;
    "-nrpo NAME,TYPE,MOUNTPOINT /dev/sda") printf '/dev/sda disk \n/dev/sda1 part \n/dev/sda2 part /mnt\n' ;;
    "-nrpo NAME,TYPE,MOUNTPOINT /dev/sdb") printf '/dev/sdb disk \n/dev/sdb1 part /iso\n/dev/sdb2 part \n' ;;
    "-nrpo NAME,TYPE,MOUNTPOINT /dev/sdc") printf '/dev/sdc disk \n/dev/sdc1 part \n/dev/mapper/vg-home lvm /home\n' ;;
    "-nrpo NAME,TYPE,MOUNTPOINT /dev/sdd") printf '/dev/sdd disk \n/dev/sdd1 part [SWAP]\n' ;;
    "-no PKNAME /dev/sr0") printf '\n' ;;
    *) return 0 ;;
  esac
}

findmnt() {
  [ "$*" = "-no SOURCE /iso" ] && [ -n "${FAKE_ISO:-}" ] && printf '%s\n' "$FAKE_ISO"
  return 0
}

readlink() {
  if [ "$1" = -f ]; then printf '%s\n' "$2"; else command readlink "$@"; fi
}

# shellcheck source=/dev/null
source "$bin"

run() {
  local args=(--disk /dev/sda --hostname flux --full-name "Ada Lovelace" --user ada
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
  (main "${args[@]}")
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

accepts() {
  local name=$1 expect=$2 output
  shift 2
  if output=$(run "$@" 2>&1) && printf '%s' "$output" | grep -qx "$expect"; then
    printf 'ok   %s\n' "$name"
  else
    printf 'FAIL %s: %s\n' "$name" "$output"
    failures=$((failures + 1))
  fi
}

rejects "missing disk" --disk DROP
rejects "partition-like disk" --disk /dev/nonexistent1
rejects "loop device" --disk /dev/loop0
rejects "cd-rom" --disk /dev/sr0
rejects "live medium mounted at /iso" --disk /dev/sdb
FAKE_ISO=/dev/sr0 accepts "live medium on a cd-rom leaves sda free" "disk=/dev/sda"
export FAKE_ISO=/dev/sda
rejects "live medium found by findmnt" --disk /dev/sda
unset FAKE_ISO
rejects "disk with a mounted volume" --disk /dev/sdc
rejects "disk with an active swap" --disk /dev/sdd
accepts "by-id path resolves to the disk" "disk=/dev/sda" --disk "$scratch/ata-FAKE_SSD_123"
rejects "uppercase user" --user Ada
rejects "user with a space" --user "ada l"
rejects "root user" --user root
rejects "reserved user daemon" --user daemon
rejects "reserved user nixbld1" --user nixbld1
rejects "reserved user systemd-oom" --user systemd-oom
rejects "reserved user greeter" --user greeter
rejects "hostname with a leading dash" --hostname -flux
rejects "hostname with a trailing dash" --hostname flux-
rejects "hostname with a dot" --hostname flux.local
rejects "hostname too long" --hostname "$(printf 'a%.0s' $(seq 64))"
rejects "reserved hostname" --hostname LocalHost
rejects "unknown time zone" --timezone Mars/Olympus
rejects "time zone path escape" --timezone ../../etc/passwd
rejects "unknown keymap" --keymap zz
rejects "unknown locale" --locale xx_XX.UTF-8
rejects "plain password" --password-hash hunter2
rejects "md5 hash" --password-hash "\$1\$abc\$def"
rejects "truncated yescrypt hash" --password-hash "\$y\$j9T\$salt\$short"
rejects "quote in the full name" --full-name 'Ada "the" Lovelace'
rejects "dollar in the full name" --full-name "Ada \${pkgs}"
rejects "missing yes" --yes DROPFLAG
rejects "bad progress mode" --progress xml
accepts "valid plan" "user=ada"
accepts "yescrypt hash" "user=ada" --password-hash "$yescrypt"
accepts "hostname with an inner dash" "hostname=my-flux" --hostname my-flux

rm -rf "$scratch"
[ "$failures" -eq 0 ]
