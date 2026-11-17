<script lang="ts">
	import { Icon, icons } from '@facile/muse';
	import { Caret, grow } from '@facile/mise-en-scene';
	import { t } from '$lib/i18n';
	import { glyphs, marks } from '$lib/marks';
	import FluxFiles from './desk/FluxFiles.svelte';
	import FluxSettings from './desk/FluxSettings.svelte';
	import { ALTERNATE, type DesktopScene } from './scene.svelte';

	let { scene, compact }: { scene: DesktopScene; compact: boolean } = $props();

	const gap = $derived(compact ? 12 : 14);
	const view = $derived(compact ? 358 : 1104);
	const half = $derived(compact ? view - 2 * gap : (view - 3 * gap) / 2);
	const full = $derived(view - 2 * gap);
	const offset = $derived(scene.opened ? half + gap : 0);

	const settingsName = $derived(t('demo.app.settings'));
	const apps = $derived([
		{ name: t('demo.app.files'), icon: glyphs.files, line: t('demo.app.files.line') },
		{ name: settingsName, icon: glyphs.settings, line: t('demo.app.settings.line') },
		{ name: 'Ghostty', icon: marks.terminal, line: t('demo.app.terminal') },
		{ name: 'Nuage', icon: marks.nuage, line: t('demo.app.nuage') },
		{ name: 'Sablier', icon: marks.sablier, line: t('demo.app.sablier') }
	]);
	const hits = $derived(apps.filter((app) => app.name.toLowerCase().includes(scene.query.toLowerCase())));
	const title = $derived(scene.workspace !== 2 ? '' : scene.opened ? settingsName : 'ghostty');
	const hintKey = $derived(scene.hintKey === 'space' ? t('demo.space') : scene.hintKey);
</script>

<div class="desk flux-pal" class:swap={scene.swapped} class:compact data-hit="screen">
	<div class="wall base"></div>
	<div class="wall alt" class:show={scene.wallpaper === ALTERNATE} style:background-image="url('/wallpapers/{ALTERNATE}-large.jpg')"></div>

	<div class="bar" style:inset="{gap}px {gap}px auto">
		<div class="left">
			{#each compact ? [1, 2, 3] : [1, 2, 3, 4] as ws (ws)}
				<span class="ws" class:on={ws === scene.workspace}>{ws}</span>
			{/each}
			{#if !compact && title}<span class="wtitle">{title}</span>{/if}
		</div>
		<span class="clock">{#if !compact}<span>{t('demo.date')}</span>{/if}<span>09:41</span></span>
		<div class="right">
			<span class="pill">
				<Icon icon={scene.wifi === 'connected' ? glyphs.wifi : glyphs.wifiOff} size={14} />
				{#if !compact && scene.wifi === 'connected'}<span>Maison</span>{/if}
			</span>
			{#if !compact}<span class="pill"><Icon icon={glyphs.volume} size={14} /><span>60%</span></span>{/if}
			<span class="pill"><Icon icon={glyphs.battery} size={14} />{#if !compact}<span>82%</span>{/if}</span>
			<span class="pill icon"><Icon icon={glyphs.power} size={14} /></span>
		</div>
	</div>

	<div class="cols" class:away={scene.workspace !== 2} style:top="{2 * gap + (compact ? 30 : 32)}px">
		<div class="strip" style:padding-left="{gap}px" style:transform="translateX({-offset}px)">
			<div class="col term" style:width="{half}px" style:margin-right="{gap}px">
				<div class="tline"><span class="mute">~</span> <span class="ok">❯</span> ls</div>
				<div class="tline mute">Code&nbsp;&nbsp;Documents&nbsp;&nbsp;Nuage&nbsp;&nbsp;Photos</div>
				<div class="tline"><span class="mute">~</span> <span class="ok">❯</span> nuage status</div>
				<div class="tline"><span class="mute">○ ~/Nuage · {t('demo.offline')}</span></div>
				<div class="tline">
					<span class="mute">~</span> <span class="ok">❯</span>
					{scene.command}{#if !scene.done}<Caret />{/if}
				</div>
				{#if scene.done}
					<div class="tline"><span class="ok">✓</span> <span class="mute">{t('demo.started')} · 09:41</span></div>
					<div class="tline"><span class="mute">~</span> <span class="ok">❯</span> <Caret /></div>
				{/if}
			</div>

			<div
				class="col grower"
				class:open={scene.opened}
				style:width="{scene.opened ? full : 0}px"
				style:margin-right="{scene.opened ? gap : 0}px"
			>
				<div class="inner" style:width="{full}px">
					<FluxSettings page={scene.page} wifi={scene.wifi} wallpaper={scene.wallpaper} {compact} deskDark={!scene.swapped} />
				</div>
			</div>

			<div class="col" style:width="{half}px">
				<FluxFiles {compact} online={scene.wifi === 'connected'} />
			</div>
		</div>
	</div>

	<div class="hint" class:show={scene.hint}>
		<span class="key">Super</span><span class="plus">+</span><span class="key">{hintKey}</span>
	</div>

	<div class="launcher" class:show={scene.launcher}>
		<div class="search">
			<Icon icon={icons.search} size={16} />
			{#if scene.query}<span>{scene.query}</span><Caret />{:else}<span class="ph">{t('demo.search')}</span>{/if}
		</div>
		<div class="list">
			{#each hits as app, i (app.icon)}
				<div class="hit-wrap" transition:grow={{ duration: 200 }}>
					<div class="hit" class:sel={i === 0}>
						<Icon icon={app.icon} size={20} />
						<span class="hit-name">{app.name}</span>
						{#if !compact}<span class="hit-line">{app.line}</span>{/if}
					</div>
				</div>
			{/each}
		</div>
	</div>
</div>

<style>
	.desk {
		position: absolute;
		inset: 0;
		overflow: hidden;
		border-radius: 18px;
		background: var(--fx-p);
		font-family: var(--font-fc-body);
		font-size: 13px;
	}
	.wall {
		position: absolute;
		inset: 0;
		background-position: center;
		background-size: cover;
	}
	.wall.base {
		background: var(--flux-wallpaper);
	}
	.wall.alt {
		opacity: 0;
		transition: opacity 400ms ease-out;
	}
	.wall.alt.show {
		opacity: 1;
	}
	.bar {
		position: absolute;
		z-index: 1;
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: 32px;
		padding: 0 4px;
		border-radius: 10px;
		background: color-mix(in oklab, var(--fx-p) 94%, transparent);
		font-size: 12px;
	}
	.compact .bar {
		height: 30px;
	}
	.left,
	.right {
		display: flex;
		align-items: center;
		gap: 4px;
		min-width: 0;
	}
	.ws {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 22px;
		border-radius: 6px;
		color: var(--fx-m);
		font-variant-numeric: tabular-nums;
	}
	.ws.on {
		background: var(--fx-f);
		color: var(--fx-p);
		font-weight: 600;
	}
	.wtitle {
		margin-left: 10px;
		color: var(--fx-m);
		white-space: nowrap;
	}
	.clock {
		position: absolute;
		left: 50%;
		display: flex;
		gap: 8px;
		translate: -50% 0;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}
	.pill {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		height: 24px;
		padding: 0 8px;
		border-radius: 6px;
		background: var(--fx-s);
		white-space: nowrap;
	}
	.compact .pill {
		padding: 0 6px;
	}
	.cols {
		position: absolute;
		bottom: 14px;
		left: 0;
		right: 0;
		overflow: hidden;
		transition:
			transform 360ms cubic-bezier(0.22, 1, 0.36, 1),
			opacity 240ms ease-out;
	}
	.cols.away {
		opacity: 0;
		transform: translateY(-40px);
	}
	.compact .cols {
		bottom: 12px;
	}
	.strip {
		display: flex;
		height: 100%;
		transition: transform 320ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.col {
		flex: none;
		height: 100%;
		overflow: hidden;
		border-radius: 12px;
		box-shadow: 0 18px 40px -18px rgb(0 0 0 / 0.45);
	}
	.term {
		padding: 16px 18px;
		background: var(--fx-p);
		color: var(--fx-f);
		font-family: var(--font-fc-mono);
		font-size: 13px;
		line-height: 24px;
	}
	.tline {
		white-space: nowrap;
		overflow: hidden;
	}
	.mute {
		color: var(--fx-m);
	}
	.ok {
		color: color-mix(in oklab, var(--color-fc-success), var(--fx-f) 35%);
	}
	.grower {
		opacity: 0;
		transition:
			width 320ms cubic-bezier(0.22, 1, 0.36, 1),
			margin-right 320ms cubic-bezier(0.22, 1, 0.36, 1),
			opacity 200ms ease-out;
	}
	.grower.open {
		opacity: 1;
	}
	.inner {
		height: 100%;
	}
	.hint {
		position: absolute;
		z-index: 2;
		bottom: 26px;
		left: 50%;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px;
		border-radius: 12px;
		background: var(--fx-p);
		box-shadow: 0 16px 40px -16px rgb(0 0 0 / 0.5);
		opacity: 0;
		translate: -50% 8px;
		transition:
			opacity 180ms ease-out,
			translate 180ms ease-out;
	}
	.hint.show {
		opacity: 1;
		translate: -50% 0;
	}
	.key {
		padding: 4px 10px;
		border-radius: 7px;
		background: var(--fx-s);
		font-weight: 600;
		font-size: 12px;
	}
	.plus {
		color: var(--fx-m);
	}
	.launcher {
		position: absolute;
		z-index: 2;
		top: 132px;
		left: 50%;
		width: 460px;
		padding: 6px;
		border-radius: 14px;
		background: var(--fx-p);
		box-shadow: 0 30px 70px -20px rgb(0 0 0 / 0.55);
		opacity: 0;
		transform: translate(-50%, 6px) scale(0.98);
		transition:
			opacity 180ms ease-out,
			transform 200ms ease-out;
	}
	.compact .launcher {
		top: 96px;
		width: 318px;
	}
	.launcher.show {
		opacity: 1;
		transform: translate(-50%, 0) scale(1);
	}
	.search {
		display: flex;
		align-items: center;
		gap: 8px;
		height: 42px;
		margin-bottom: 4px;
		padding: 0 12px;
		border-radius: 10px;
		background: var(--fx-s);
		font-size: 14px;
	}
	.ph {
		color: var(--fx-m);
	}
	.hit-wrap {
		padding-top: 2px;
	}
	.hit {
		display: flex;
		align-items: center;
		gap: 12px;
		height: 38px;
		padding: 0 12px;
		border-radius: 9px;
		font-size: 13.5px;
	}
	.hit.sel {
		background: var(--fx-f);
		color: var(--fx-p);
	}
	.hit-name {
		font-weight: 500;
		white-space: nowrap;
	}
	.hit-line {
		margin-left: auto;
		color: var(--fx-m);
		font-size: 12px;
		white-space: nowrap;
	}
	.hit.sel .hit-line {
		color: inherit;
		opacity: 0.7;
	}
	@media (prefers-reduced-motion: reduce) {
		.strip,
		.grower,
		.launcher,
		.hint,
		.cols,
		.wall.alt {
			transition: none;
		}
	}
</style>
