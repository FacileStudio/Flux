<script lang="ts">
	import { Icon, icons } from '@facile/muse';
	import { Bar, Caret } from '@facile/mise-en-scene';
	import { t } from '$lib/i18n';
	import { marks } from '$lib/marks';
	import { clock, type DesktopScene } from './scene.svelte';

	let { scene, compact }: { scene: DesktopScene; compact: boolean } = $props();

	const gap = $derived(compact ? 10 : 14);
	const view = $derived(compact ? 346 : 948);
	const term = $derived(compact ? 326 : 520);
	const app = $derived(compact ? 326 : 400);
	const offset = $derived(scene.opened ? (compact ? gap + term : Math.max(0, gap + term + gap + app + gap - view)) : 0);
	const results = $derived(
		[
			{ name: 'Sablier', icon: marks.sablier, line: t('demo.app.sablier') },
			{ name: 'Nuage', icon: marks.nuage, line: t('demo.app.nuage') }
		].filter((item) => item.name.toLowerCase().startsWith(scene.query.toLowerCase()))
	);
</script>

<div class="desk">
	<div class="bar" style="left:{gap}px;right:{gap}px">
		<span class="ws">
			<Icon icon={marks.flux} size={16} />
			<span class="dot on"></span><span class="dot"></span><span class="dot"></span>
		</span>
		<span class="time">09:41</span>
		<span class="tray"><Bar w={compact ? 28 : 44} h={6} /><Bar w={18} h={6} /></span>
	</div>

	<div class="cols">
		<div class="strip" style="gap:{gap}px;padding-left:{gap}px;transform:translateX({-offset}px)">
			<div class="col term" style="width:{term}px">
				<div class="tline"><span class="dim">~</span> <span class="ok">❯</span> nuage status</div>
				<div class="tline"><span class="ok">✓</span> <span class="dim">~/Nuage · {t('demo.synced')}</span></div>
				<div class="tline">
					<span class="dim">~</span> <span class="ok">❯</span>
					{scene.command}{#if !scene.done}<Caret />{/if}
				</div>
				{#if scene.done}
					<div class="tline"><span class="ok">✓</span> <span class="dim">{t('demo.started')} · {t('demo.project')}</span></div>
					<div class="tline"><span class="dim">~</span> <span class="ok">❯</span> <Caret /></div>
				{/if}
			</div>

			<div class="col win new" class:open={scene.opened} style="width:{scene.opened ? app : 0}px">
				<div class="inner" style="width:{app}px">
					<div class="head"><Icon icon={marks.sablier} size={18} /><span>Sablier</span></div>
					<div class="timer">
						<span class="chip"><span class="live"></span>{t('demo.running')}</span>
						<span class="digits">{clock(scene.seconds)}</span>
						<span class="proj">{t('demo.project')}</span>
					</div>
					<div class="label">{t('demo.today')}</div>
					{#each [62, 48, 70] as w, i (i)}
						<div class="row"><Bar w="{w}%" h={7} /><Bar w={compact ? 34 : 52} h={7} /></div>
					{/each}
				</div>
			</div>

			<div class="col win" style="width:{app}px">
				<div class="inner" style="width:{app}px">
					<div class="head"><Icon icon={marks.nuage} size={18} /><span>Nuage</span></div>
					{#each [58, 44, 66, 50, 38] as w, i (i)}
						<div class="file"><span class="thumb"></span><Bar w="{w}%" h={7} /></div>
					{/each}
				</div>
			</div>
		</div>
	</div>

	<div class="launcher" class:show={scene.launcher} style="width:{compact ? 300 : 420}px">
		<div class="search">
			<Icon icon={icons.search} size={16} />
			{#if scene.query}<span>{scene.query}</span><Caret />{:else}<span class="ph">{t('demo.search')}</span>{/if}
		</div>
		{#each results as item, i (item.name)}
			<div class="hit" class:sel={i === 0 && scene.query}>
				<Icon icon={item.icon} size={22} />
				<span class="hit-name">{item.name}</span>
				<span class="hit-line">{item.line}</span>
			</div>
		{/each}
	</div>
</div>

<style>
	.desk {
		--d-win: var(--color-fc-page);
		--d-fill: var(--color-fc-surface);
		position: absolute;
		inset: 6px;
		overflow: hidden;
		border-radius: 18px;
		background: var(--flux-wallpaper);
		box-shadow: 0 30px 70px -24px rgb(0 0 0 / 0.3);
		font-family: var(--font-fc-body);
		color: var(--color-fc-fg);
	}
	@media (prefers-color-scheme: dark) {
		:global(:root:not(.light)) .desk {
			--d-win: var(--color-fc-component);
			--d-fill: var(--color-fc-surface);
		}
	}
	:global(:root.dark) .desk {
		--d-win: var(--color-fc-component);
		--d-fill: var(--color-fc-surface);
	}
	.bar {
		position: absolute;
		top: 10px;
		height: 30px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 12px;
		border-radius: 999px;
		background: color-mix(in oklab, var(--d-win) 88%, transparent);
		box-shadow: 0 6px 18px -10px rgb(0 0 0 / 0.3);
		font-size: 12px;
	}
	.ws,
	.tray {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.dot {
		width: 6px;
		height: 6px;
		border-radius: 999px;
		background: var(--color-fc-fg-muted);
		opacity: 0.4;
	}
	.dot.on {
		width: 16px;
		opacity: 1;
		background: var(--color-fc-fg);
	}
	.time {
		position: absolute;
		left: 50%;
		translate: -50% 0;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}
	.cols {
		position: absolute;
		top: 52px;
		bottom: 14px;
		left: 0;
		right: 0;
		overflow: hidden;
	}
	.strip {
		display: flex;
		height: 100%;
		transition: transform 900ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.col {
		flex: none;
		height: 100%;
		border-radius: 14px;
		box-shadow: 0 18px 40px -18px rgb(0 0 0 / 0.35);
	}
	.term {
		padding: 14px 16px;
		background: var(--color-fc-inverse);
		color: var(--color-fc-on-inverse);
		font-family: var(--font-fc-mono);
		font-size: 12.5px;
		line-height: 22px;
	}
	.tline {
		white-space: nowrap;
		overflow: hidden;
	}
	.dim {
		color: var(--color-fc-on-inverse-muted);
	}
	.ok {
		color: color-mix(in oklab, var(--color-fc-success), var(--color-fc-on-inverse) 35%);
	}
	.win {
		overflow: hidden;
		background: var(--d-win);
	}
	.new {
		opacity: 0;
		transition:
			width 900ms cubic-bezier(0.22, 1, 0.36, 1),
			opacity 500ms ease;
	}
	.new.open {
		opacity: 1;
	}
	.inner {
		display: flex;
		flex-direction: column;
		gap: 12px;
		height: 100%;
		padding: 16px 18px;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 8px;
		font-family: var(--font-fc-title);
		font-size: 14px;
		font-weight: 600;
	}
	.timer {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 6px;
		padding: 16px;
		border-radius: 12px;
		background: var(--d-fill);
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 2px 10px;
		border-radius: 999px;
		background: color-mix(in oklab, var(--color-fc-success) 14%, transparent);
		color: var(--color-fc-success);
		font-size: 11px;
		font-weight: 500;
	}
	.live {
		width: 6px;
		height: 6px;
		border-radius: 999px;
		background: currentColor;
	}
	.digits {
		font-family: var(--font-fc-mono);
		font-size: 34px;
		line-height: 1.1;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}
	.proj {
		font-size: 12px;
		color: var(--color-fc-fg-muted);
	}
	.label {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-fc-fg-muted);
	}
	.row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 10px 12px;
		border-radius: 10px;
		background: var(--d-fill);
	}
	.file {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 4px;
	}
	.thumb {
		flex: none;
		width: 26px;
		height: 26px;
		border-radius: 7px;
		background: var(--d-fill);
	}
	.launcher {
		position: absolute;
		top: 96px;
		left: 50%;
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 8px;
		border-radius: 16px;
		background: var(--d-win);
		box-shadow: 0 30px 70px -20px rgb(0 0 0 / 0.45);
		opacity: 0;
		transform: translate(-50%, 8px) scale(0.97);
		transition:
			opacity 220ms ease,
			transform 260ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.launcher.show {
		opacity: 1;
		transform: translate(-50%, 0) scale(1);
	}
	.search {
		display: flex;
		align-items: center;
		gap: 8px;
		height: 40px;
		padding: 0 12px;
		border-radius: 10px;
		background: var(--d-fill);
		font-size: 14px;
	}
	.ph {
		color: var(--color-fc-fg-muted);
	}
	.hit {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 10px;
		border-radius: 10px;
		font-size: 13px;
	}
	.hit.sel {
		background: var(--d-fill);
	}
	.hit-name {
		font-weight: 600;
	}
	.hit-line {
		margin-left: auto;
		color: var(--color-fc-fg-muted);
		font-size: 12px;
		white-space: nowrap;
	}
	@media (prefers-reduced-motion: reduce) {
		.strip,
		.new,
		.launcher {
			transition: none;
		}
	}
</style>
