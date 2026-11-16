<script lang="ts">
	import { Stage, type Act } from '@facile/mise-en-scene';
	import { t } from '$lib/i18n';
	import FluxDesktop from './FluxDesktop.svelte';
	import { DesktopScene } from './scene.svelte';

	let { class: className = '' }: { class?: string } = $props();

	const scene = new DesktopScene();
	const command = $derived(`sablier start ${t('demo.project')}`);

	async function script(act: Act) {
		await act.wait(700);
		await act.type(command, (value) => (scene.command = value), 55);
		await act.wait(450);
		scene.done = true;
		await act.wait(1100);
		scene.launcher = true;
		await act.wait(700);
		await act.type('sab', (value) => (scene.query = value), 140);
		await act.wait(900);
		scene.launcher = false;
		scene.opened = true;
		await act.wait(700);
		await act.count(7, (value) => (scene.seconds = 4 + value), 1000);
		await act.wait(1200);
		scene.reset();
		await act.wait(900);
	}
</script>

<div class={className}>
	<Stage
		width={960}
		height={560}
		compact={{ below: 640, width: 358, height: 520 }}
		{script}
		reset={() => scene.reset()}
		poster={() => scene.poster(command)}
		label={t('landing.demo.label')}
	>
		{#snippet children({ compact })}
			<FluxDesktop {scene} {compact} />
		{/snippet}
	</Stage>
</div>
