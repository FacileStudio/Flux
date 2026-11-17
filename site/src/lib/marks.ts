import { registerIcons } from '@facile/muse';
import solar from './solar.json';

/** The marks of Flux (Solar filters), of Nix (NixOS artwork, CC BY 4.0) and of the suite apps (Solar, CC BY 4.0), plus a few feature glyphs. */
export const marks = {
	flux: 'solar:filters-bold-duotone',
	nix: 'flux:nix',
	sablier: 'solar:hourglass-bold-duotone',
	nuage: 'solar:cloud-bold-duotone',
	boite: 'solar:box-minimalistic-bold-duotone',
	kit: 'solar:checklist-minimalistic-bold-duotone',
	terminal: 'solar:command-bold-duotone',
	cpu: 'solar:cpu-bolt-bold-duotone'
} as const;

/** The line glyphs drawn inside the faux Flux screens (Solar, CC BY 4.0). */
export const glyphs = {
	wifi: 'solar:wi-fi-linear',
	wifiOff: 'solar:wi-fi-cross-linear',
	bluetooth: 'solar:bluetooth-linear',
	volume: 'solar:volume-loud-linear',
	battery: 'solar:battery-full-minimalistic-linear',
	display: 'solar:display-linear',
	palette: 'solar:palette-linear',
	power: 'solar:power-linear',
	files: 'solar:folder-with-files-linear',
	settings: 'solar:settings-linear',
	lock: 'solar:lock-keyhole-minimalistic-linear',
	restart: 'solar:restart-linear',
	eye: 'solar:eye-linear',
	usb: 'solar:usb-linear',
	disk: 'solar:ssd-round-linear',
	keyboard: 'solar:keyboard-linear',
	headphones: 'solar:headphones-round-linear',
	phone: 'solar:smartphone-linear',
	folder: 'solar:folder-bold-duotone',
	document: 'solar:document-text-linear',
	file: 'solar:file-linear',
	sheet: 'solar:chart-linear',
	image: 'solar:gallery-linear',
	home: 'solar:home-2-linear',
	download: 'solar:download-minimalistic-linear',
	recent: 'solar:clock-circle-linear',
	cloud: 'solar:cloud-linear',
	code: 'solar:code-linear',
	back: 'solar:alt-arrow-left-linear',
	forward: 'solar:alt-arrow-right-linear',
	grid: 'solar:widget-linear',
	list: 'solar:list-linear'
} as const;

registerIcons({
	...solar,
	[marks.nix]: {
		body: '<path fill="currentColor" d="M9.88 16.18L21.57 16.18L20.25 18.52L17.11 18.51L18.67 21.22L18 22.38L16.64 22.38L14.42 18.52L11.23 18.51ZM9.43 8.09L3.59 18.21L2.22 15.9L3.8 13.19L0.67 13.18L0 12.02L0.68 10.84L5.14 10.85L6.74 8.09ZM16.69 11.74L10.84 1.62L13.53 1.59L15.09 4.31L16.66 1.61L17.99 1.61L18.68 2.79L16.44 6.64L18.03 9.4ZM7.32 12.26L13.17 22.38L10.48 22.41L8.92 19.69L7.35 22.39L6.01 22.39L5.33 21.21L7.57 17.36L5.98 14.6ZM14.1 7.8L2.41 7.8L3.73 5.46L6.87 5.47L5.31 2.75L5.98 1.6L7.34 1.6L9.56 5.46L12.75 5.47ZM14.56 15.91L20.41 5.79L21.77 8.1L20.2 10.81L23.33 10.82L23.99 11.98L23.31 13.16L18.86 13.15L17.26 15.91Z"/>',
		width: 24,
		height: 24
	}
});
