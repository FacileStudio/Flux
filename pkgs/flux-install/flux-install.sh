die() {
  printf 'flux-install: %s\n' "$*" >&2
  exit 1
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

[ "$(id -u)" -eq 0 ] || die "run me as root: sudo flux-install"
[ -d /sys/firmware/efi ] || die "this machine booted in legacy BIOS mode; Flux installs on UEFI only"

case "$(uname -m)" in
  x86_64) system=x86_64-linux ;;
  aarch64) system=aarch64-linux ;;
  *) die "unsupported architecture $(uname -m)" ;;
esac
release=$(nixos-version | cut -d. -f1,2)

printf 'Flux installer. UEFI only. The disk you pick is wiped completely.\n\n'
lsblk -dpno NAME,SIZE,MODEL,TYPE | grep -E 'disk$' || die "no disks found"
printf '\n'

disk=$(ask "Target disk (for example /dev/nvme0n1):")
[ -b "$disk" ] || die "$disk is not a block device"
[ "$(lsblk -dno TYPE "$disk")" = disk ] || die "$disk is not a whole disk"

hostname=$(ask "Hostname [flux]:")
hostname=${hostname:-flux}
[[ "$hostname" =~ ^[a-zA-Z0-9][a-zA-Z0-9-]{0,62}$ ]] || die "invalid hostname: $hostname"

username=$(ask "Username:")
[[ "$username" =~ ^[a-z_][a-z0-9_-]{0,31}$ ]] || die "invalid username: $username"

while true; do
  read -r -s -p "Password for $username: " password
  printf '\n'
  read -r -s -p "Repeat the password: " password2
  printf '\n'
  [ -n "$password" ] && [ "$password" = "$password2" ] && break
  printf 'Passwords are empty or do not match, try again.\n'
done
hashed=$(printf '%s' "$password" | mkpasswd -s -m sha-512)
unset password password2

printf '\nEvery partition and file on %s will be destroyed:\n' "$disk"
lsblk -po NAME,SIZE,FSTYPE,LABEL,MOUNTPOINTS "$disk"
confirm=$(ask "Type the disk name again to wipe it ($disk):")
[ "$confirm" = "$disk" ] || die "confirmation did not match, nothing was touched"

swapoff -a || true
umount -R /mnt 2>/dev/null || true
wipefs -af "$disk"
parted -s "$disk" -- mklabel gpt \
  mkpart ESP fat32 1MiB 513MiB \
  set 1 esp on \
  mkpart nixos ext4 513MiB 100%
udevadm settle

boot=$(part "$disk" 1)
root=$(part "$disk" 2)
mkfs.fat -F 32 -n BOOT "$boot"
mkfs.ext4 -F -L nixos "$root"
udevadm settle

mount "$root" /mnt
mkdir -p /mnt/boot
mount -o umask=077 "$boot" /mnt/boot

nixos-generate-config --root /mnt
mkdir -p /mnt/etc/flux
cp -r "$FLUX_TEMPLATE"/. /mnt/etc/flux/
chmod -R u+w /mnt/etc/flux
mv /mnt/etc/nixos/hardware-configuration.nix /mnt/etc/flux/
rm -rf /mnt/etc/nixos

sed -i "s|system = \"x86_64-linux\";|system = \"$system\";|" /mnt/etc/flux/flake.nix
sed -i \
  -e "s|^  hostName = .*;|  hostName = \"$hostname\";|" \
  -e "s|^  userName = .*;|  userName = \"$username\";|" \
  -e "s|^  hashedPassword = .*;|  hashedPassword = \"$hashed\";|" \
  -e "s|^  system.stateVersion = .*;|  system.stateVersion = \"$release\";|" \
  /mnt/etc/flux/configuration.nix

nixos-install --flake /mnt/etc/flux#flux --no-root-passwd

printf '\nFlux is installed. Your system flake lives in /etc/flux. Remove the install media and reboot.\n'
