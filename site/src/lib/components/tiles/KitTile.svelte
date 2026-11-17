<script lang="ts">
	import { Caret, Stage, Terminal, type Act } from '@facile/mise-en-scene';
	import { t } from '$lib/i18n';

	type Line = { kind: 'cmd' | 'ok' | 'prompt'; text: string };

	const story = $derived<Line[]>([
		{ kind: 'cmd', text: 'kit add sablier firefox ripgrep' },
		{ kind: 'ok', text: 'sablier · Facile' },
		{ kind: 'ok', text: 'firefox · Flathub' },
		{ kind: 'ok', text: 'ripgrep · Nix' },
		{ kind: 'cmd', text: 'kit sync' },
		{ kind: 'ok', text: t('demo.kit.synced') }
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

<!--
@component
kit adds three apps from three sources, then rebuilds a fresh machine from its list.
-->

<div class="rounded-fc-lg bg-fc-component">
	<Stage width={520} height={300} compact={{ below: 400, width: 340, height: 290 }} {script} {reset} poster={() => (lines = [...story])}>
		{#snippet children({ compact })}
			<Terminal style="inset:{compact ? 14 : 24}px">
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
</div>

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
