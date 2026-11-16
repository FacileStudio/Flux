/** A page of the faux Flux Settings window. */
export type SettingsPage = 'wifi' | 'bluetooth' | 'sound' | 'displays' | 'appearance';

/** Where the hero's Wi-Fi network stands. */
export type WifiState = 'idle' | 'connecting' | 'connected';

/** The hero desktop's state: the terminal, the launcher, the Settings column, the network and the theme. */
export class DesktopScene {
	command = $state('');
	done = $state(false);
	hint = $state(false);
	launcher = $state(false);
	query = $state('');
	opened = $state(false);
	page = $state<SettingsPage>('wifi');
	wifi = $state<WifiState>('idle');
	swapped = $state(false);

	/** Puts the desktop back on its opening frame: an empty prompt in Ghostty, Files beside it. */
	reset(): void {
		this.command = '';
		this.done = false;
		this.hint = false;
		this.launcher = false;
		this.query = '';
		this.opened = false;
		this.page = 'wifi';
		this.wifi = 'idle';
		this.swapped = false;
	}

	/** The frame that tells the story alone: the timer started, Settings open on Wi-Fi, the network connected. */
	poster(command: string): void {
		this.reset();
		this.command = command;
		this.done = true;
		this.opened = true;
		this.wifi = 'connected';
	}
}
