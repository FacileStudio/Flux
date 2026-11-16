<script lang="ts">
	import { Icon } from '@facile/muse';
	import { Stage, type Act } from '@facile/mise-en-scene';
	import { t } from '$lib/i18n';
	import { marks } from '$lib/marks';

	let selected = $state(0);
	let booted = $state(false);

	const rows = $derived([
		{ gen: 42, when: t('demo.boot.today') },
		{ gen: 41, when: t('demo.boot.yesterday') },
		{ gen: 40, when: t('demo.boot.week') }
	]);

	function reset() {
		selected = 0;
		booted = false;
	}

	async function script(act: Act) {
		await act.wait(1400);
		selected = 1;
		await act.wait(1000);
		booted = true;
		await act.wait(2600);
		reset();
		await act.wait(600);
	}
</script>

<!--
@component
The Flux boot menu, drawn from the real one: the mark, one pill per system generation, and the keys.
The selection moves to yesterday's generation and starts it.
-->

<div class="rounded-fc-lg bg-fc-component">
	<Stage
		width={520}
		height={300}
		compact={{ below: 400, width: 340, height: 290 }}
		{script}
		{reset}
		poster={() => ((selected = 1), (booted = true))}
	>
		{#snippet children({ compact })}
			<div class="screen flux-pal" class:compact>
				<Icon icon={marks.flux} size={40} />
				<div class="menu">
					{#each rows as row, i (row.gen)}
						<div class="row" class:sel={i === selected}>
							<span class="gen">Flux {row.gen}</span>
							<span class="when">{row.when}</span>
							<span class="ok" class:show={booted && i === selected}>✓</span>
						</div>
					{/each}
				</div>
				<div class="hint">{t('demo.boot.hint')}</div>
			</div>
		{/snippet}
	</Stage>
</div>

<style>
	.screen {
		position: absolute;
		inset: 24px;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 14px;
		padding: 0 40px;
		overflow: hidden;
		border-radius: 16px;
		background: var(--fx-p);
		box-shadow: 0 18px 40px -24px rgb(0 0 0 / 0.35);
	}
	.screen.compact {
		inset: 14px;
		padding: 0 18px;
	}
	.menu {
		display: flex;
		flex-direction: column;
		gap: 4px;
		width: 100%;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 10px;
		height: 38px;
		padding: 0 16px;
		border-radius: 999px;
		font-size: 14px;
		transition:
			background 200ms ease-out,
			color 200ms ease-out;
	}
	.row.sel {
		background: var(--fx-f);
		color: var(--fx-p);
	}
	.gen {
		font-weight: 600;
	}
	.when {
		color: var(--fx-m);
		white-space: nowrap;
	}
	.row.sel .when {
		color: inherit;
		opacity: 0.7;
	}
	.ok {
		margin-left: auto;
		opacity: 0;
		transition: opacity 200ms ease-out;
	}
	.ok.show {
		opacity: 1;
	}
	.hint {
		color: var(--fx-m);
		font-size: 11px;
		white-space: nowrap;
	}
	@media (prefers-reduced-motion: reduce) {
		.row,
		.ok {
			transition: none;
		}
	}
</style>
