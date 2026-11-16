<script lang="ts">
	import { Stage, Tile, type Act } from '@facile/mise-en-scene';
	import { t } from '$lib/i18n';
	import FluxSettings from '../desk/FluxSettings.svelte';
	import type { SettingsPage } from '../scene.svelte';

	const PAGES: SettingsPage[] = ['wifi', 'bluetooth', 'sound', 'displays'];

	let page = $state<SettingsPage>('wifi');

	async function script(act: Act) {
		for (const next of PAGES) {
			page = next;
			await act.wait(2200);
		}
	}
</script>

<Tile title={t('surface.settings.title')} body={t('surface.settings.body')}>
	<Stage
		width={960}
		height={480}
		compact={{ below: 560, width: 358, height: 470 }}
		{script}
		reset={() => (page = 'wifi')}
		poster={() => (page = 'bluetooth')}
		label={t('surface.settings.label')}
	>
		{#snippet children({ compact })}
			<div class="screen flux-pal" class:compact>
				<div class="window">
					<FluxSettings {page} {compact} nav={PAGES} />
				</div>
			</div>
		{/snippet}
	</Stage>
</Tile>

<style>
	.screen {
		position: absolute;
		inset: 24px 24px 8px;
		overflow: hidden;
		border-radius: 16px;
		background: var(--flux-wallpaper);
	}
	.screen.compact {
		inset: 14px 14px 6px;
	}
	.window {
		position: absolute;
		inset: 28px 56px;
		overflow: hidden;
		border-radius: 12px;
		box-shadow: 0 24px 60px -20px rgb(0 0 0 / 0.55);
	}
	.compact .window {
		inset: 14px;
	}
</style>
