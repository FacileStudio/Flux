log=/tmp/flux-install.log
keymaps="us gb fr de es it pt br be ch ca nl se no dk fi pl cz latam"
locales="en_US.UTF-8 en_GB.UTF-8 fr_FR.UTF-8 fr_BE.UTF-8 fr_CH.UTF-8 fr_CA.UTF-8 es_ES.UTF-8 de_DE.UTF-8 de_CH.UTF-8 it_IT.UTF-8 pt_PT.UTF-8 pt_BR.UTF-8 nl_NL.UTF-8"

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

usage() {
  cat <<'EOF'
usage: flux-install                      interactive install
       flux-install --disk DEV --hostname NAME --full-name NAME --user NAME
                    --password-hash HASH --timezone ZONE --keymap MAP
                    --locale LOCALE --yes [--progress json|text] [--dry-run]
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

fail() {
  step=failed
  if [ "$progress" = json ]; then
    printf '{"step":"error","message":"%s"}\n' "$(json_escape "$*")"
  fi
  printf 'flux-install: %s\n' "$*" >&2
  exit 1
}

on_exit() {
  local code=$?
  if [ "$code" -ne 0 ] && [ "$step" != failed ] && [ "$step" != start ]; then
    if [ "$progress" = json ]; then
      printf '{"step":"error","message":"%s"}\n' "$(json_escape "the $step step failed, see $log")"
    fi
    printf 'flux-install: the %s step failed, see %s\n' "$step" "$log" >&2
  fi
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
  local source
  source=$(findmnt -no SOURCE /iso 2>/dev/null || true)
  [ -n "$source" ] || return 0
  source=$(readlink -f "$source")
  lsblk -no PKNAME "$source" 2>/dev/null | head -n1 | grep . | sed 's|^|/dev/|' || printf '%s' "$source"
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
  hashed=$(printf '%s' "$password" | mkpasswd -s -m sha-512)
  unset password password2
}

validate() {
  local tzdir boot path type found=0
  [ -n "$disk" ] || fail "--disk is required"
  while read -r path type; do
    if [ "$path" = "$disk" ] && [ "$type" = disk ]; then found=1; fi
  done < <(lsblk -dnpo PATH,TYPE 2>/dev/null || true)
  [ "$found" -eq 1 ] || fail "$disk is not a whole disk"
  case "$disk" in
    /dev/zram*|/dev/loop*) fail "$disk is not a real disk" ;;
  esac
  boot=$(boot_disk)
  [ -z "$boot" ] || [ "$boot" != "$disk" ] || fail "$disk holds the Flux live system, pick another disk"
  [[ "$hostname" =~ ^[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?$ ]] || fail "invalid hostname: $hostname"
  [[ "$username" =~ ^[a-z_][a-z0-9_-]{0,31}$ ]] || fail "invalid username: $username"
  [ "$username" != root ] || fail "the username root is reserved"
  [ "${#full_name}" -le 64 ] || fail "the full name is longer than 64 characters"
  case "$full_name" in
    *[\"\\\$\|\&\`]*|*[[:cntrl:]]*) fail "the full name contains a character Flux cannot store: \" \\ \$ | & \`" ;;
  esac
  [[ "$hashed" =~ ^\$6\$(rounds=[0-9]+\$)?[./0-9A-Za-z]{1,16}\$[./0-9A-Za-z]{86}$ ]] \
    || fail "--password-hash must be a sha-512 crypt hash (\$6\$...)"
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

partition() {
  step=partition
  emit partition 2 "Preparing $disk"
  swapoff -a >>"$log" 2>&1 || true
  umount -R /mnt >>"$log" 2>&1 || true
  quiet wipefs -af "$disk"
  quiet parted -s "$disk" -- mklabel gpt \
    mkpart ESP fat32 1MiB 513MiB \
    set 1 esp on \
    mkpart nixos ext4 513MiB 100%
  quiet udevadm settle
  emit partition 10 "Partitioned $disk"
}

format() {
  local boot root
  step=format
  boot=$(part "$disk" 1)
  root=$(part "$disk" 2)
  emit format 12 "Formatting $boot and $root"
  quiet mkfs.fat -F 32 -n BOOT "$boot"
  quiet mkfs.ext4 -F -L nixos "$root"
  quiet udevadm settle
  mount "$root" /mnt
  mkdir -p /mnt/boot
  mount -o umask=077 "$boot" /mnt/boot
  emit format 20 "Formatted the disk"
}

configure() {
  local system release
  step=config
  emit config 22 "Writing the system configuration"
  case "$(uname -m)" in
    x86_64) system=x86_64-linux ;;
    aarch64) system=aarch64-linux ;;
    *) fail "unsupported architecture $(uname -m)" ;;
  esac
  release=$(nixos-version | cut -d. -f1,2)
  quiet nixos-generate-config --root /mnt
  mkdir -p /mnt/etc/flux
  cp -r "$FLUX_TEMPLATE"/. /mnt/etc/flux/
  chmod -R u+w /mnt/etc/flux
  mv /mnt/etc/nixos/hardware-configuration.nix /mnt/etc/flux/
  rm -rf /mnt/etc/nixos
  sed -i "s|system = \"x86_64-linux\";|system = \"$system\";|" /mnt/etc/flux/flake.nix
  sed -i \
    -e "s|^  hostName = .*;|  hostName = \"$hostname\";|" \
    -e "s|^  userName = .*;|  userName = \"$username\";|" \
    -e "s|^  fullName = .*;|  fullName = \"$full_name\";|" \
    -e "s|^  hashedPassword = .*;|  hashedPassword = \"$hashed\";|" \
    -e "s|^  timeZone = .*;|  timeZone = \"$timezone\";|" \
    -e "s|^  keyMap = .*;|  keyMap = \"$keymap\";|" \
    -e "s|^  locale = .*;|  locale = \"$locale\";|" \
    -e "s|^  system.stateVersion = .*;|  system.stateVersion = \"$release\";|" \
    /mnt/etc/flux/configuration.nix
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
  step=install
  emit install 30 "Installing Flux"
  if [ "$progress" = json ]; then
    nixos-install --flake /mnt/etc/flux#flux --no-root-passwd 2>&1 | track
  else
    nixos-install --flake /mnt/etc/flux#flux --no-root-passwd 2>&1 | tee -a "$log"
  fi
  step="done"
  emit "done" 100 "Flux is installed. Your system flake lives in /etc/flux."
}

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
partition
format
configure
install_system
[ "$progress" = json ] || printf '\nRemove the install media and reboot.\n'
