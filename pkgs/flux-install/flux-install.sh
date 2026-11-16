log=/tmp/flux-install.log
keymaps="us gb fr de es it pt br be ch ca nl se no dk fi pl cz latam"
locales="en_US.UTF-8 en_GB.UTF-8 fr_FR.UTF-8 fr_BE.UTF-8 fr_CH.UTF-8 fr_CA.UTF-8 es_ES.UTF-8 de_DE.UTF-8 de_CH.UTF-8 it_IT.UTF-8 pt_PT.UTF-8 pt_BR.UTF-8 nl_NL.UTF-8"
reserved_users="root bin daemon adm lp sync shutdown halt mail news uucp operator games ftp man nobody nogroup
  users wheel admin sudo audio video input render kvm disk tty dialout floppy cdrom tape kmem utmp lock
  networkmanager messagebus dbus polkituser sshd nscd avahi rtkit geoclue pipewire greeter regreet
  colord cups fwupd-refresh flatpak nm-openvpn nm-iodine usbmux tss uinput systemd-journal"
reserved_hosts="localhost localdomain"
cache=https://cache.nixos.org

disk=""
hostname=""
full_name=""
username=""
hashed=""
timezone=""
keymap=""
locale=""
yes=0
dry_run=0
progress=text
step=start
interactive=1
stage=""
mounted=0
online=1
root_uuid=""
boot_uuid=""
logging=0
tail_lines=30
nix_flags=(--extra-experimental-features "nix-command flakes")

usage() {
  cat <<'EOF'
usage: flux-install                      interactive install
       flux-install --disk DEV --hostname NAME --full-name NAME --user NAME
                    --password-hash HASH --timezone ZONE --keymap MAP
                    --locale LOCALE --yes [--progress json|text] [--dry-run]

--password-hash takes a yescrypt ($y$) or sha-512 ($6$) crypt hash.
EOF
}

json_escape() {
  local s=$1
  s=${s//\\/\\\\}
  s=${s//\"/\\\"}
  s=${s//$'\n'/ }
  s=${s//$'\t'/ }
  s=${s//$'\r'/ }
  printf '%s' "$s"
}

emit() {
  if [ "$progress" = json ]; then
    printf '{"step":"%s","percent":%d,"message":"%s"}\n' "$1" "$2" "$(json_escape "$3")"
  else
    printf '[%3d%%] %s\n' "$2" "$3"
  fi
}

json_text() {
  local s=$1
  s=${s//\\/\\\\}
  s=${s//\"/\\\"}
  s=${s//$'\t'/ }
  s=${s//$'\r'/}
  s=${s//$'\n'/\\n}
  printf '%s' "$s"
}

log_tail() {
  [ "$logging" -eq 1 ] && [ -s "$log" ] || return 0
  tail -n "$tail_lines" "$log" | tr -d '\000-\010\013\014\016-\037\177'
}

to_console() {
  local dev
  for dev in /dev/ttyS0 /dev/console; do
    if [ -c "$dev" ] && [ -w "$dev" ]; then
      printf '%s\n' "$1" | timeout 2 tee "$dev" >/dev/null 2>&1 || true
      return 0
    fi
  done
}

report_error() {
  local message=$1 tail
  tail=$(log_tail)
  if [ "$progress" = json ]; then
    if [ -n "$tail" ]; then
      printf '{"step":"error","message":"%s","log_tail":"%s"}\n' "$(json_escape "$message")" "$(json_text "$tail")"
    else
      printf '{"step":"error","message":"%s"}\n' "$(json_escape "$message")"
    fi
  fi
  printf 'flux-install: %s\n' "$message" >&2
  [ -z "$tail" ] || to_console "flux-install failed: $message"$'\n'"$tail"
}

fail() {
  step=failed
  report_error "$*"
  exit 1
}

cleanup() {
  if [ "$mounted" -eq 1 ]; then
    sync
    umount -R /mnt >>"$log" 2>&1 || umount -lR /mnt >>"$log" 2>&1 || true
  fi
  [ -z "$stage" ] || rm -rf "$stage"
}

on_exit() {
  local code=$?
  if [ "$code" -ne 0 ] && [ "$step" != failed ] && [ "$step" != start ]; then
    report_error "the $step step failed, see $log"
  fi
  cleanup
}

ask() {
  local reply
  read -r -p "$1 " reply
  printf '%s' "$reply"
}

part() {
  case "$1" in
    *[0-9]) printf '%sp%s' "$1" "$2" ;;
    *) printf '%s%s' "$1" "$2" ;;
  esac
}

in_list() {
  local item
  for item in $2; do
    [ "$item" = "$1" ] && return 0
  done
  return 1
}

zoneinfo() {
  local dir
  for dir in "${TZDIR:-}" /etc/zoneinfo /usr/share/zoneinfo; do
    [ -n "$dir" ] && [ -d "$dir" ] && { printf '%s' "$dir"; return 0; }
  done
  return 1
}

boot_disk() {
  local source parent
  source=$(findmnt -no SOURCE /iso 2>/dev/null || true)
  [ -n "$source" ] || return 0
  source=$(readlink -f "$source")
  parent=$(lsblk -no PKNAME "$source" 2>/dev/null | head -n1 || true)
  if [ -n "$parent" ]; then printf '/dev/%s' "$parent"; else printf '%s' "$source"; fi
}

children() {
  lsblk -nrpo NAME,TYPE,MOUNTPOINT "$1" 2>/dev/null || true
}

reserved_user() {
  case "$1" in
    nixbld*|systemd-*) return 0 ;;
  esac
  in_list "$1" "$reserved_users"
}

need_value() {
  [ "$#" -ge 2 ] || fail "$1 needs a value"
}

parse() {
  local previous="" arg
  for arg in "$@"; do
    if [ "$previous" = --progress ] && [ "$arg" = json ]; then progress=json; fi
    previous=$arg
  done
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --disk) need_value "$@"; disk=$2; shift ;;
      --hostname) need_value "$@"; hostname=$2; shift ;;
      --full-name) need_value "$@"; full_name=$2; shift ;;
      --user) need_value "$@"; username=$2; shift ;;
      --password-hash) need_value "$@"; hashed=$2; shift ;;
      --timezone) need_value "$@"; timezone=$2; shift ;;
      --keymap) need_value "$@"; keymap=$2; shift ;;
      --locale) need_value "$@"; locale=$2; shift ;;
      --progress) need_value "$@"; progress=$2; shift ;;
      --yes) yes=1 ;;
      --dry-run) dry_run=1 ;;
      -h|--help) usage; exit 0 ;;
      *) fail "unknown argument: $1" ;;
    esac
    interactive=0
    shift
  done
  case "$progress" in
    json|text) ;;
    *) progress=text; fail "--progress takes json or text" ;;
  esac
}

prompt() {
  local password password2
  printf 'Flux installer. UEFI only. The disk you pick is wiped completely.\n\n'
  lsblk -dpno NAME,SIZE,MODEL,TYPE | grep -E 'disk$' || fail "no disks found"
  printf '\n'
  disk=$(ask "Target disk (for example /dev/nvme0n1):")
  hostname=$(ask "Hostname [flux]:")
  hostname=${hostname:-flux}
  full_name=$(ask "Full name:")
  username=$(ask "Username:")
  timezone=$(ask "Time zone [UTC]:")
  timezone=${timezone:-UTC}
  keymap=$(ask "Keyboard layout ($keymaps) [us]:")
  keymap=${keymap:-us}
  locale=$(ask "Locale [en_US.UTF-8]:")
  locale=${locale:-en_US.UTF-8}
  while true; do
    read -r -s -p "Password for ${username}: " password
    printf '\n'
    read -r -s -p "Repeat the password: " password2
    printf '\n'
    [ -n "$password" ] && [ "$password" = "$password2" ] && break
    printf 'Passwords are empty or do not match, try again.\n'
  done
  hashed=$(printf '%s' "$password" | mkpasswd -s -m yescrypt)
  unset password password2
}

validate_disk() {
  local boot path type found=0 name mountpoint
  [ -n "$disk" ] || fail "--disk is required"
  disk=$(realpath -m -- "$disk")
  while read -r path type; do
    if [ "$path" = "$disk" ] && [ "$type" = disk ]; then found=1; fi
  done < <(lsblk -dnpo PATH,TYPE 2>/dev/null || true)
  [ "$found" -eq 1 ] || fail "$disk is not a whole disk"
  case "$disk" in
    /dev/zram*|/dev/loop*) fail "$disk is not a real disk" ;;
  esac
  boot=$(boot_disk)
  [ -z "$boot" ] || [ "$boot" != "$disk" ] || fail "$disk holds the Flux live system, pick another disk"
  while read -r name type mountpoint; do
    case "$mountpoint" in
      "") ;;
      /iso|/iso/*) fail "$disk holds the Flux live system, pick another disk" ;;
      /mnt|/mnt/*) ;;
      *) fail "$name ($type) on $disk is in use at $mountpoint; unmount it or pick another disk" ;;
    esac
  done < <(children "$disk")
}

validate() {
  local tzdir
  validate_disk
  [[ "$hostname" =~ ^[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?$ ]] || fail "invalid hostname: $hostname"
  ! in_list "${hostname,,}" "$reserved_hosts" || fail "the hostname $hostname is reserved"
  [[ "$username" =~ ^[a-z_][a-z0-9_-]{0,31}$ ]] || fail "invalid username: $username"
  ! reserved_user "$username" || fail "the username $username is reserved for the system"
  [ "${#full_name}" -le 64 ] || fail "the full name is longer than 64 characters"
  case "$full_name" in
    *[\"\\\$\|\&\`]*|*[[:cntrl:]]*) fail "the full name contains a character Flux cannot store: \" \\ \$ | & \`" ;;
  esac
  [[ "$hashed" =~ ^\$6\$(rounds=[0-9]+\$)?[./0-9A-Za-z]{1,16}\$[./0-9A-Za-z]{86}$ ]] \
    || [[ "$hashed" =~ ^\$y\$[./0-9A-Za-z]+\$[./0-9A-Za-z]{1,86}\$[./0-9A-Za-z]{43}$ ]] \
    || fail "--password-hash must be a yescrypt (\$y\$...) or sha-512 (\$6\$...) crypt hash"
  tzdir=$(zoneinfo) || fail "no zoneinfo directory found"
  [[ "$timezone" =~ ^[A-Za-z0-9_+-]+(/[A-Za-z0-9_+-]+)*$ ]] && [ -f "$tzdir/$timezone" ] \
    || fail "unknown time zone: $timezone"
  in_list "$keymap" "$keymaps" || fail "unknown keyboard layout: $keymap (one of: $keymaps)"
  in_list "$locale" "$locales" || fail "unknown locale: $locale (one of: $locales)"
  [ "$interactive" -eq 1 ] || [ "$yes" -eq 1 ] || fail "--yes is required: it confirms that $disk is erased"
}

plan() {
  printf 'disk=%s\nhostname=%s\nfull-name=%s\nuser=%s\ntimezone=%s\nkeymap=%s\nlocale=%s\n' \
    "$disk" "$hostname" "$full_name" "$username" "$timezone" "$keymap" "$locale"
}

confirm() {
  local answer
  printf '\nEvery partition and file on %s will be destroyed:\n' "$disk"
  lsblk -po NAME,SIZE,FSTYPE,LABEL,MOUNTPOINTS "$disk"
  answer=$(ask "Type the disk name again to wipe it ($disk):")
  [ "$answer" = "$disk" ] || fail "confirmation did not match, nothing was touched"
}

quiet() {
  if [ "$progress" = json ]; then
    "$@" >>"$log" 2>&1
  else
    "$@" 2>&1 | tee -a "$log"
  fi
}

hardware() {
  printf '%s\n' "${1%\}*}"
  cat <<EOF
  fileSystems."/" = {
    device = "/dev/disk/by-uuid/$root_uuid";
    fsType = "ext4";
  };

  fileSystems."/boot" = {
    device = "/dev/disk/by-uuid/$boot_uuid";
    fsType = "vfat";
    options = [ "fmask=0077" "dmask=0077" ];
  };
}
EOF
}

configure() {
  local system release detected
  step=config
  emit config 2 "Writing the system configuration"
  case "$(uname -m)" in
    x86_64) system=x86_64-linux ;;
    aarch64) system=aarch64-linux ;;
    *) fail "unsupported architecture $(uname -m)" ;;
  esac
  release=$(nixos-version | cut -d. -f1,2)
  root_uuid=$(cat /proc/sys/kernel/random/uuid)
  boot_uuid=$(od -An -N4 -tx4 /dev/urandom | tr -d ' \n' | tr a-f A-F)
  boot_uuid="${boot_uuid:0:4}-${boot_uuid:4:4}"
  stage=$(mktemp -d /tmp/flux-stage.XXXXXX)
  cp -r "$FLUX_TEMPLATE"/. "$stage"/
  chmod -R u+w "$stage"
  if [ -n "${FLUX_LOCK:-}" ]; then
    cp "$FLUX_LOCK" "$stage/flake.lock"
    chmod 0644 "$stage/flake.lock"
  fi
  detected=$(nixos-generate-config --show-hardware-config --no-filesystems 2>>"$log")
  [[ "$detected" == *"}"* ]] || fail "nixos-generate-config printed no hardware configuration"
  hardware "$detected" >"$stage/hardware-configuration.nix"
  sed -i "s|system = \"x86_64-linux\";|system = \"$system\";|" "$stage/flake.nix"
  sed -i \
    -e "s|^  hostName = .*;|  hostName = \"$hostname\";|" \
    -e "s|^  userName = .*;|  userName = \"$username\";|" \
    -e "s|^  fullName = .*;|  fullName = \"$full_name\";|" \
    -e "s|^  timeZone = .*;|  timeZone = \"$timezone\";|" \
    -e "s|^  keyMap = .*;|  keyMap = \"$keymap\";|" \
    -e "s|^  locale = .*;|  locale = \"$locale\";|" \
    -e "s|^  system.stateVersion = .*;|  system.stateVersion = \"$release\";|" \
    "$stage/configuration.nix"
  emit config 4 "Configuration written"
}

drvs_to_build() {
  nix "${nix_flags[@]}" build --dry-run "path:$stage#nixosConfigurations.flux.config.system.build.toplevel" 2>&1 \
    | tee -a "$log" | grep -E '^[[:space:]]+/nix/store/[^[:space:]]+\.drv$' || true
}

preflight() {
  local drvs sources
  step=config
  emit config 5 "Checking the system before touching $disk"
  if ! curl -fsS --max-time 10 -o /dev/null "$cache/nix-cache-info" >>"$log" 2>&1; then
    online=0
    nix_flags+=(--offline --option substituters "")
  fi
  nix "${nix_flags[@]}" eval --raw "path:$stage#nixosConfigurations.flux.config.system.build.toplevel.drvPath" \
    >>"$log" 2>&1 || fail "the Flux system does not evaluate, nothing was touched; see $log"
  printf '\n' >>"$log"
  if [ "$online" -eq 0 ]; then
    mapfile -t drvs < <(drvs_to_build)
    sources=0
    if [ "${#drvs[@]}" -gt 0 ]; then
      sources=$(nix "${nix_flags[@]}" derivation show "${drvs[@]}" 2>>"$log" \
        | jq '[.. | objects | select(has("outputs")) | .outputs[] | objects | select(has("hash"))] | length')
    fi
    [ "$sources" -eq 0 ] \
      || fail "this machine needs $sources downloads the installer does not carry; connect to the internet and try again, nothing was touched"
    emit config 8 "Offline install: ${#drvs[@]} small builds, nothing to download"
  else
    emit config 8 "System checked"
  fi
}

release_holders() {
  local name type mountpoint
  umount -R /mnt >>"$log" 2>&1 || true
  while read -r name type mountpoint; do
    case "$type" in
      lvm|crypt|dm) dmsetup remove --retry "$name" >>"$log" 2>&1 || true ;;
      raid*) mdadm --stop "$name" >>"$log" 2>&1 || true ;;
    esac
  done < <(children "$disk" | tac)
  while read -r name type mountpoint; do
    case "$type" in
      disk|part) ;;
      *) fail "$name ($type) on $disk is still active and could not be stopped" ;;
    esac
  done < <(children "$disk")
  while read -r name type mountpoint; do
    [ "$type" != part ] || quiet wipefs -af "$name"
  done < <(children "$disk" | tac)
}

partition() {
  step=partition
  emit partition 10 "Preparing $disk"
  release_holders
  quiet wipefs -af "$disk"
  quiet parted -s "$disk" -- mklabel gpt \
    mkpart ESP fat32 1MiB 1025MiB \
    set 1 esp on \
    mkpart nixos ext4 1025MiB 100%
  quiet udevadm settle
  emit partition 14 "Partitioned $disk"
}

format() {
  local boot root
  step=format
  boot=$(part "$disk" 1)
  root=$(part "$disk" 2)
  emit format 16 "Formatting $boot and $root"
  quiet mkfs.fat -F 32 -n BOOT -i "${boot_uuid//-/}" "$boot"
  quiet mkfs.ext4 -F -L nixos -U "$root_uuid" "$root"
  quiet udevadm settle
  mounted=1
  mount "$root" /mnt
  mkdir -p /mnt/boot
  mount -o umask=077 "$boot" /mnt/boot
  emit format 20 "Formatted the disk"
}

place() {
  step=config
  emit config 22 "Copying the system configuration to /etc/flux"
  mkdir -p /mnt/etc/flux
  cp -r "$stage"/. /mnt/etc/flux/
  emit config 26 "Copying the Flux and nixpkgs sources to $disk"
  nix "${nix_flags[@]}" flake archive --to /mnt "path:/mnt/etc/flux" >>"$log" 2>&1 \
    || fail "could not copy the Flux sources to $disk; see $log"
  emit config 30 "Configuration written to /etc/flux"
}

track() {
  local line copied=0 built=0 shown=30 percent
  while IFS= read -r line; do
    printf '%s\n' "$line" >>"$log"
    case "$line" in
      *"installing the boot loader"*)
        step=bootloader
        [ "$shown" -ge 90 ] || { shown=90; emit bootloader 90 "Installing the boot loader"; }
        continue ;;
      "copying path"*) copied=$((copied + 1)) ;;
      "building '"*) built=$((built + 1)) ;;
      *) continue ;;
    esac
    [ "$step" = install ] || continue
    percent=$((30 + 58 * (copied + built) / (copied + built + 300)))
    if [ "$percent" -gt "$shown" ]; then
      shown=$percent
      emit install "$percent" "Installing Flux ($copied paths copied, $built built)"
    fi
  done
}

install_system() {
  local args=(--flake /mnt/etc/flux#flux --no-root-passwd)
  [ "$online" -eq 1 ] || args+=(--option substituters "auto?trusted=1")
  step=install
  emit install 30 "Installing Flux"
  if [ "$progress" = json ]; then
    nixos-install "${args[@]}" 2>&1 | track
  else
    nixos-install "${args[@]}" 2>&1 | tee -a "$log"
  fi
}

set_password() {
  step=bootloader
  emit bootloader 96 "Setting the password of $username"
  printf '%s:%s\n' "$username" "$hashed" \
    | nixos-enter --root /mnt -c '/nix/var/nix/profiles/system/sw/bin/chpasswd -e' >>"$log" 2>&1 \
    || fail "could not set the password of $username; see $log"
}

main() {
  trap on_exit EXIT
  parse "$@"
  [ "$interactive" -eq 0 ] || prompt
  validate
  if [ "$dry_run" -eq 1 ]; then
    plan
    exit 0
  fi
  [ "$(id -u)" -eq 0 ] || fail "run me as root: sudo flux-install"
  [ -d /sys/firmware/efi ] || fail "this machine booted in legacy BIOS mode; Flux installs on UEFI only"
  [ "$interactive" -eq 0 ] || confirm
  : >"$log"
  logging=1
  configure
  preflight
  partition
  format
  place
  install_system
  set_password
  step="done"
  emit "done" 100 "Flux is installed. Your system flake lives in /etc/flux."
  [ "$progress" = json ] || printf '\nRemove the install media and reboot.\n'
}

if [ "${BASH_SOURCE[0]}" = "$0" ]; then
  main "$@"
fi
