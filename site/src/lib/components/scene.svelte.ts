/** The hero desktop's state: what the terminal shows, the launcher, and whether Sablier has opened. */
export class DesktopScene {
	command = $state('');
	done = $state(false);
	launcher = $state(false);
	query = $state('');
	opened = $state(false);
	seconds = $state(4);

	/** Puts the desktop back on its opening frame: an empty prompt, Nuage peeking on the right. */
	reset(): void {
		this.command = '';
		this.done = false;
		this.launcher = false;
		this.query = '';
		this.opened = false;
		this.seconds = 4;
	}

	/** The frame that tells the story alone: the timer started, Sablier open as its own column. */
	poster(command: string): void {
		this.command = command;
		this.done = true;
		this.launcher = false;
		this.query = '';
		this.opened = true;
		this.seconds = 12;
	}
}

/** Seconds as the `hh:mm:ss` a running Sablier timer shows. */
export function clock(seconds: number): string {
	const pad = (value: number) => String(value).padStart(2, '0');
	return `${pad(Math.floor(seconds / 3600))}:${pad(Math.floor((seconds % 3600) / 60))}:${pad(seconds % 60)}`;
}
