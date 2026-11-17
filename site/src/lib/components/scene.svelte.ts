/** A page of the faux Flux Settings window. */
export type SettingsPage = 'wifi' | 'bluetooth' | 'sound' | 'displays' | 'appearance';

/** Where the hero's Wi-Fi network stands. */
export type WifiState = 'idle' | 'connecting' | 'connected';

/** The wallpapers Flux ships, default first. Thumbnails live in `static/wallpapers/<id>.jpg`. */
export const WALLPAPERS = [
	'concrete-maze-courtyard',
	'shaded-hall',
	'tatami-room',
	'greenhouse',
	'alpine-road',
	'kingfisher-branch'
] as const;

/** One of the shipped wallpapers. */
export type Wallpaper = (typeof WALLPAPERS)[number];

/** The wallpaper the demo switches to, the only other one drawn full size. */
export const ALTERNATE: Wallpaper = 'tatami-room';

/** The hero desktop's state: terminal, launcher, Settings column, network, theme, wallpaper and workspace. */
export class DesktopScene {
	command = $state('');
	done = $state(false);
	hint = $state(false);
	hintKey = $state<'space' | '3' | '2'>('space');
	launcher = $state(false);
	query = $state('');
	opened = $state(false);
	page = $state<SettingsPage>('wifi');
	wifi = $state<WifiState>('idle');
	swapped = $state(false);
	wallpaper = $state<Wallpaper>(WALLPAPERS[0]);
	workspace = $state(2);

	/** Puts the desktop back on its opening frame: an empty prompt in Ghostty, Files beside it, Flux in dark mode. */
	reset(): void {
		this.command = '';
		this.done = false;
		this.hint = false;
		this.hintKey = 'space';
		this.launcher = false;
		this.query = '';
		this.opened = false;
		this.page = 'wifi';
		this.wifi = 'idle';
		this.swapped = false;
		this.wallpaper = WALLPAPERS[0];
		this.workspace = 2;
	}

	/** The frame that tells the story alone: the timer started in Ghostty, Files beside it, still offline. */
	poster(command: string): void {
		this.reset();
		this.command = command;
		this.done = true;
	}
}
