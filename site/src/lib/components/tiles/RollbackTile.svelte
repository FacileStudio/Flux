<script lang="ts">
	import { Stage, Tile, type Act } from '@facile/mise-en-scene';
	import { t } from '$lib/i18n';

	let selected = $state(0);
	let booted = $state(false);

	const rows = $derived([
		{ gen: 42, when: t('tile.rollback.today') },
		{ gen: 41, when: t('tile.rollback.yesterday') },
		{ gen: 40, when: t('tile.rollback.week') }
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

<Tile title={t('tile.rollback.title')} body={t('tile.rollback.body')}>
	<Stage
		width={420}
		height={250}
		compact={{ below: 330, width: 320, height: 250 }}
		{script}
		{reset}
		poster={() => ((selected = 1), (booted = true))}
	>
		{#snippet children({ compact })}
			<div class="boot" style="left:{compact ? 16 : 40}px;right:{compact ? 16 : 40}px">
				<div class="title">{t('tile.rollback.menu')}</div>
				{#each rows as row, i (row.gen)}
					<div class="row" class:sel={i === selected}>
						<span class="gen">Flux {row.gen}</span>
						<span class="when">{row.when}</span>
						<span class="ok" class:show={booted && i === selected}>✓</span>
					</div>
				{/each}
			</div>
		{/snippet}
	</Stage>
</Tile>

<style>
	.boot {
		position: absolute;
		top: 28px;
		bottom: 16px;
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 14px;
		border-radius: 12px;
		background: var(--color-fc-inverse);
		color: var(--color-fc-on-inverse);
		font-family: var(--font-fc-mono);
		font-size: 12px;
		box-shadow: 0 18px 40px -16px rgb(0 0 0 / 0.35);
	}
	.title {
		margin-bottom: 4px;
		color: var(--color-fc-on-inverse-muted);
	}
	.row {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 10px;
		border-radius: 8px;
		transition: background 300ms ease;
	}
	.row.sel {
		background: color-mix(in oklab, var(--color-fc-on-inverse) 14%, var(--color-fc-inverse));
	}
	.gen {
		font-weight: 600;
	}
	.when {
		color: var(--color-fc-on-inverse-muted);
		white-space: nowrap;
	}
	.ok {
		margin-left: auto;
		opacity: 0;
		color: color-mix(in oklab, var(--color-fc-success), var(--color-fc-on-inverse) 35%);
		transition: opacity 300ms ease;
	}
	.ok.show {
		opacity: 1;
	}
	@media (prefers-reduced-motion: reduce) {
		.row,
		.ok {
			transition: none;
		}
	}
</style>
