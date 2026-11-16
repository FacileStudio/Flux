<script lang="ts">
	import { Icon } from '@facile/muse';
	import { Caret, Stage, Terminal, Tile, type Act } from '@facile/mise-en-scene';
	import { t } from '$lib/i18n';
	import { marks } from '$lib/marks';

	type Line = { kind: 'cmd' | 'ok' | 'prompt'; text: string };

	const story = $derived<Line[]>([
		{ kind: 'cmd', text: 'boite new api' },
		{ kind: 'ok', text: `api · flux · headless · ${t('tile.boite.ready')}` },
		{ kind: 'cmd', text: 'boite ssh api' },
		{ kind: 'prompt', text: 'flux@api ~ ❯' }
	]);

	let lines = $state<Line[]>([]);

	function reset() {
		lines = [];
	}

	async function script(act: Act) {
		await act.wait(600);
		for (const line of story) {
			if (line.kind !== 'cmd') lines.push(line);
			else {
				lines.push({ kind: 'cmd', text: '' });
				const at = lines.length - 1;
				await act.type(line.text, (value) => (lines[at].text = value), 60);
			}
			await act.wait(line.kind === 'cmd' ? 500 : 900);
		}
		await act.wait(2200);
		reset();
		await act.wait(400);
	}
</script>

<Tile title={t('tile.boite.title')} body={t('tile.boite.body')}>
	{#snippet badge()}
		<a
			href="https://boite.facile.studio"
			class="inline-flex size-11 shrink-0 items-center justify-center rounded-fc-md text-fc-fg-muted hover:bg-fc-surface hover:text-fc-fg"
			aria-label={t('tile.boite.link')}
		>
			<Icon icon={marks.boite} size={22} />
		</a>
	{/snippet}
	<Stage width={420} height={250} compact={{ below: 330, width: 320, height: 250 }} {script} {reset} poster={() => (lines = [...story])}>
		{#snippet children({ compact })}
			<Terminal style="top:28px;left:{compact ? 16 : 40}px;right:{compact ? 16 : 40}px;bottom:16px">
				{#each lines as line, i (i)}
					<div class="line">
						{#if line.kind === 'cmd'}
							<span class="dim">$</span>
							{line.text}{#if i === lines.length - 1}<Caret />{/if}
						{:else if line.kind === 'ok'}
							<span class="ok">✓</span> <span class="dim">{line.text}</span>
						{:else}
							<span class="ok">{line.text}</span> <Caret />
						{/if}
					</div>
				{/each}
				{#if lines.length === 0 || lines.at(-1)?.kind === 'ok'}
					<div class="line"><span class="dim">$</span> <Caret /></div>
				{/if}
			</Terminal>
		{/snippet}
	</Stage>
</Tile>

<style>
	.line {
		white-space: nowrap;
		overflow: hidden;
	}
	.dim {
		color: var(--color-fc-on-inverse-muted);
	}
	.ok {
		color: color-mix(in oklab, var(--color-fc-success), var(--color-fc-on-inverse) 35%);
	}
</style>
