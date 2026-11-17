import type { DownloadEntry } from '@facile/muse';

/** The Flux release the site offers. Bump it when a new ISO is published. */
export const VERSION = '0.3.1';

/** The Flux repository. */
export const REPO = 'https://github.com/FacileStudio/Flux';

/** Every release, with its notes and assets. */
export const RELEASES = `${REPO}/releases`;

const base = `https://flux.facile.studio/releases/v${VERSION}`;

/** The notes of the offered release. */
export const RELEASE_NOTES = `${RELEASES}/tag/v${VERSION}`;

/** The live installer ISO for Intel and AMD machines. */
export const ISO_X86_64 = `${base}/flux-${VERSION}-x86_64-linux.iso`;

/** The live installer ISO for ARM64 UEFI virtual machines and servers. */
export const ISO_AARCH64 = `${base}/flux-${VERSION}-aarch64-linux.iso`;

/** The SHA-256 sums of both ISOs. */
export const CHECKSUMS = `${base}/SHA256SUMS`;

/** The two ISOs as muse download entries, for the landing's device-only download block. */
export const DOWNLOADS: DownloadEntry[] = [
	{ platform: 'linux', href: ISO_X86_64, label: 'ISO x86_64', arch: 'Intel / AMD', version: VERSION, checksumHref: CHECKSUMS },
	{ platform: 'linux', href: ISO_AARCH64, label: 'ISO aarch64', arch: 'ARM64 UEFI (VMs and servers)', version: VERSION, checksumHref: CHECKSUMS }
];

/** The command that checks a downloaded ISO against the published sums. */
export const VERIFY = 'sha256sum --check --ignore-missing SHA256SUMS';

/** The command that writes the x86_64 ISO to a USB stick. `/dev/sdX` is the stick, never a system disk. */
export const WRITE_USB = `sudo dd if=flux-${VERSION}-x86_64-linux.iso of=/dev/sdX bs=4M status=progress oflag=sync`;

/** The command that boots Flux in a local VM through Nix. */
export const TRY_VM = 'nix run github:FacileStudio/Flux#vm';

/** The command that starts a machine flake importing Flux. */
export const INIT_FLAKE = 'nix flake init -t github:FacileStudio/Flux';
