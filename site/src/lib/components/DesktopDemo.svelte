<script lang="ts">
	import { tick } from 'svelte';
	import { Cursor, Stage, cursorAt, type Act } from '@facile/mise-en-scene';
	import { t } from '$lib/i18n';
	import FluxDesktop from './FluxDesktop.svelte';
	import { ALTERNATE, DesktopScene } from './scene.svelte';

	let { class: className = '' }: { class?: string } = $props();

	const scene = new DesktopScene();
	const command = 'sablier start';
	let cursor = $state(cursorAt());
	let host = $state<HTMLElement>();

	function reset() {
		scene.reset();
		cursor.visible = false;
	}

	function hit(name: string): HTMLElement | undefined {
		return host?.querySelector<HTMLElement>(`[data-hit="${name}"]`) ?? undefined;
	}

	async function press(act: Act, name: string, fx = 0.5, fy = 0.55, settle = 650) {
		await act.move(cursor, act.point(hit(name), fx, fy), settle);
		await act.click(cursor);
	}

	async function switchTo(act: Act, workspace: 2 | 3) {
		scene.hintKey = workspace === 3 ? '3' : '2';
		scene.hint = true;
		await act.wait(450);
		scene.workspace = workspace;
		await act.wait(350);
		scene.hint = false;
	}

	async function script(act: Act) {
		cursor = { ...act.point(hit('screen'), 0.6, 0.72), visible: false, down: false };
		await act.wait(700);
		await act.type(command, (value) => (scene.command = value), 60);
		await act.wait(350);
		scene.done = true;
		await act.wait(1000);
		scene.hint = true;
		await act.wait(700);
		scene.launcher = true;
		await act.wait(350);
		scene.hint = false;
		await act.wait(300);
		await act.type(t('demo.query'), (value) => (scene.query = value), 160);
		await act.wait(800);
		scene.launcher = false;
		scene.opened = true;
		await act.wait(650);
		await tick();
		cursor.visible = true;
		await act.wait(200);
		await press(act, 'net-maison', 0.35);
		scene.wifi = 'connecting';
		await act.wait(1100);
		scene.wifi = 'connected';
		await act.wait(1300);
		await press(act, 'nav-appearance', 0.4);
		scene.page = 'appearance';
		await tick();
		await act.wait(500);
		await press(act, 'theme-light', 0.5, 0.45);
		scene.swapped = true;
		await act.wait(1600);
		await press(act, 'theme-dark', 0.5, 0.45);
		scene.swapped = false;
		await act.wait(900);
		await press(act, `wall-${ALTERNATE}`, 0.5, 0.5, 500);
		scene.wallpaper = ALTERNATE;
		await act.wait(900);
		cursor.visible = false;
		await switchTo(act, 3);
		await act.wait(1800);
		await switchTo(act, 2);
		await act.wait(900);
		scene.reset();
		await act.wait(900);
	}
</script>

<div class="desk-demo {className}" bind:this={host}>
	<Stage
		width={1104}
		height={660}
		compact={{ below: 640, width: 358, height: 600 }}
		{script}
		{reset}
		poster={() => scene.poster(command)}
		label={t('landing.demo.label')}
	>
		{#snippet children({ compact })}
			<FluxDesktop {scene} {compact} />
			<Cursor {cursor} />
		{/snippet}
	</Stage>
</div>
