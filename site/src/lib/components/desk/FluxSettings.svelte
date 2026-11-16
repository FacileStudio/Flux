<script lang="ts">
	import { Icon, icons } from '@facile/muse';
	import { Bar } from '@facile/mise-en-scene';
	import { t } from '$lib/i18n';
	import { glyphs } from '$lib/marks';
	import type { SettingsPage, WifiState } from '../scene.svelte';
	import Signal from './Signal.svelte';

	let {
		page,
		wifi = 'connected',
		compact = false,
		siteDark = false,
		deskDark = false,
		nav = ['wifi', 'bluetooth', 'sound', 'displays', 'appearance']
	}: {
		page: SettingsPage;
		wifi?: WifiState;
		compact?: boolean;
		siteDark?: boolean;
		deskDark?: boolean;
		nav?: SettingsPage[];
	} = $props();

	const NAV_ICONS: Record<SettingsPage, string> = {
		wifi: glyphs.wifi,
		bluetooth: glyphs.bluetooth,
		sound: glyphs.volume,
		displays: glyphs.display,
		appearance: glyphs.palette
	};

	const networks = $derived([
		{ name: 'Maison', level: 4, lock: true, sub: wifi === 'connected' ? t('demo.wifi.connected') : wifi === 'connecting' ? t('demo.wifi.connecting') : 'WPA2' },
		{ name: 'Bureau', level: 3, lock: true, sub: 'WPA3' },
		{ name: 'Café du Port', level: 3, lock: false, sub: t('demo.wifi.open') },
		{ name: 'Freebox-7A21C4', level: 2, lock: true, sub: 'WPA2' }
	]);

	const devices = $derived([
		{ name: 'MX Keys', icon: glyphs.keyboard, sub: t('demo.wifi.connected') },
		{ name: 'WH-1000XM4', icon: glyphs.headphones, sub: t('demo.wifi.connected') },
		{ name: 'Pixel 8', icon: glyphs.phone, sub: t('demo.bt.off') }
	]);

	const outputs = $derived([
		{ name: t('demo.sound.speakers'), icon: glyphs.volume, sub: t('demo.sound.builtin') },
		{ name: 'HDMI / DisplayPort', icon: glyphs.display, sub: t('demo.sound.builtin') },
		{ name: 'WH-1000XM4', icon: glyphs.bluetooth, sub: 'Bluetooth' }
	]);

	const themes = $derived([
		{ id: 'light', label: t('demo.theme.light'), swap: siteDark, on: !deskDark },
		{ id: 'dark', label: t('demo.theme.dark'), swap: !siteDark, on: deskDark }
	]);
</script>

<!--
@component
The Flux Settings window drawn from the real app: a sidebar with search and pages, and one page at a
time. Compact swaps the sidebar for a row of page icons. Elements a script points at carry `data-hit`.
-->

<div class="settings" class:compact>
	{#if !compact}
		<aside class="side">
			<div class="search"><Icon icon={icons.search} size={14} /><span>{t('demo.settings.search')}</span></div>
			{#each nav as item (item)}
				<div class="nav" class:sel={item === page} data-hit="nav-{item}">
					<Icon icon={NAV_ICONS[item]} size={16} />
					<span>{t(`demo.nav.${item}`)}</span>
				</div>
			{/each}
		</aside>
	{/if}

	<div class="main">
		{#if compact}
			<div class="chips">
				{#each nav as item (item)}
					<span class="chip" class:sel={item === page} data-hit="nav-{item}"><Icon icon={NAV_ICONS[item]} size={16} /></span>
				{/each}
			</div>
		{/if}
		<div class="title">{t(`demo.nav.${page}`)}</div>

		{#if page === 'wifi'}
			<div class="card row">
				<div class="text">
					<span class="name">{t('demo.nav.wifi')}</span>
					<span class="sub">{wifi === 'connected' ? 'Maison' : t('demo.wifi.off')}</span>
				</div>
				<span class="toggle on"></span>
			</div>
			<div class="label">{t('demo.wifi.networks')}</div>
			<div class="card list">
				{#each networks as net, i (net.name)}
					<div class="item" data-hit={i === 0 ? 'net-maison' : undefined}>
						<Signal level={net.level} />
						<div class="text">
							<span class="name">{net.name}</span>
							<span class="sub" class:live={i === 0 && wifi === 'connected'}>{net.sub}</span>
						</div>
						{#if i === 0 && wifi === 'connecting'}<span class="spin"></span>{/if}
						{#if net.lock}<Icon icon={glyphs.lock} size={14} class="end" />{/if}
					</div>
				{/each}
			</div>
			{#if !compact}
				<div class="card row">
					<div class="text">
						<span class="name">{t('demo.wired')}</span>
						<span class="sub">{t('demo.wired.sub')}</span>
					</div>
				</div>
				<div class="card row">
					<div class="text">
						<span class="name">VPN</span>
						<span class="sub">Facile Studio</span>
					</div>
					<span class="toggle"></span>
				</div>
			{/if}
		{:else if page === 'bluetooth'}
			<div class="card row">
				<div class="text">
					<span class="name">{t('demo.nav.bluetooth')}</span>
					<span class="sub">flux-laptop</span>
				</div>
				<span class="toggle on"></span>
			</div>
			<div class="label">{t('demo.bt.devices')}</div>
			<div class="card list">
				{#each devices as device (device.name)}
					<div class="item">
						<Icon icon={device.icon} size={18} />
						<div class="text">
							<span class="name">{device.name}</span>
							<span class="sub">{device.sub}</span>
						</div>
						<span class="end"><Bar w={compact ? 22 : 34} h={6} /></span>
					</div>
				{/each}
			</div>
		{:else if page === 'sound'}
			<div class="label first">{t('demo.sound.output')}</div>
			<div class="card list">
				{#each outputs as out, i (out.name)}
					<div class="item" class:pick={i === 0}>
						<Icon icon={out.icon} size={18} />
						<div class="text">
							<span class="name">{out.name}</span>
							<span class="sub">{out.sub}</span>
						</div>
						{#if i === 0}<span class="end">✓</span>{/if}
					</div>
				{/each}
				<div class="item">
					<Icon icon={glyphs.volume} size={16} />
					<span class="track"><span class="fill" style:width="70%"></span><span class="knob" style:left="70%"></span></span>
					<span class="sub">70%</span>
				</div>
			</div>
		{:else if page === 'displays'}
			<div class="label first">{t('demo.displays.arrangement')}</div>
			<div class="card screens">
				<span class="screen" style:width="38%" style:aspect-ratio="16 / 10">eDP-1</span>
				<span class="screen" style:width="46%" style:aspect-ratio="16 / 9">DP-1</span>
			</div>
			<div class="card list">
				<div class="item">
					<span class="name grow">{t('demo.displays.resolution')}</span>
					<span class="select">2560 × 1600</span>
				</div>
				<div class="item">
					<span class="name grow">{t('demo.displays.scale')}</span>
					<span class="select">150 %</span>
				</div>
			</div>
		{:else}
			<div class="label first">{t('demo.theme')}</div>
			<div class="card themes">
				{#each themes as theme (theme.id)}
					<div class="theme" data-hit="theme-{theme.id}">
						<span class="preview flux-pal" class:swap={theme.swap} class:on={theme.on}>
							<span class="p-side"><i style:width="60%"></i><i></i><i></i></span>
							<span class="p-main"><i style:width="50%"></i><span class="p-card"><i></i><i></i></span></span>
						</span>
						<span class="sub">{theme.label}</span>
					</div>
				{/each}
			</div>
			<div class="label">{t('demo.wallpaper')}</div>
			<div class="card walls">
				<span class="wall on"></span>
				<span class="wall"></span>
				<span class="wall"></span>
			</div>
			{#if !compact}
				<div class="label">{t('demo.motion')}</div>
				<div class="card list">
					<div class="item">
						<span class="name grow">{t('demo.motion.size')}</span>
						<span class="track short"><span class="fill" style:width="30%"></span><span class="knob" style:left="30%"></span></span>
					</div>
					<div class="item">
						<span class="name grow">{t('demo.motion.reduce')}</span>
						<span class="toggle"></span>
					</div>
				</div>
			{/if}
		{/if}
	</div>
</div>

<style>
	.settings {
		display: flex;
		height: 100%;
		background: var(--fx-p);
		color: var(--fx-f);
		font-size: 13.5px;
		line-height: 1.3;
	}
	.side {
		display: flex;
		flex: none;
		flex-direction: column;
		gap: 2px;
		width: 196px;
		padding: 12px 10px;
		background: var(--fx-c);
	}
	.search {
		display: flex;
		align-items: center;
		gap: 8px;
		height: 32px;
		margin-bottom: 8px;
		padding: 0 10px;
		border-radius: 8px;
		background: var(--fx-s);
		color: var(--fx-m);
		white-space: nowrap;
		overflow: hidden;
	}
	.nav {
		display: flex;
		align-items: center;
		gap: 10px;
		height: 34px;
		padding: 0 10px;
		border-radius: 8px;
		white-space: nowrap;
		transition:
			background 180ms ease-out,
			color 180ms ease-out;
	}
	.nav.sel,
	.chip.sel,
	.item.pick {
		background: var(--fx-f);
		color: var(--fx-p);
	}
	.main {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 10px;
		min-width: 0;
		padding: 22px 26px;
		overflow: hidden;
	}
	.compact .main {
		padding: 14px 14px;
	}
	.chips {
		display: flex;
		gap: 6px;
		margin-bottom: 4px;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 32px;
		border-radius: 8px;
		background: var(--fx-c);
		transition:
			background 180ms ease-out,
			color 180ms ease-out;
	}
	.title {
		margin-bottom: 4px;
		font-family: var(--font-fc-title);
		font-size: 24px;
		font-weight: 600;
		letter-spacing: -0.01em;
	}
	.compact .title {
		font-size: 18px;
	}
	.label {
		margin-top: 6px;
		font-weight: 600;
	}
	.label.first {
		margin-top: 0;
	}
	.card {
		border-radius: 12px;
		background: var(--fx-c);
	}
	.row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 12px 16px;
	}
	.list {
		display: flex;
		flex-direction: column;
		padding: 4px;
	}
	.item {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 46px;
		padding: 6px 12px;
		border-radius: 8px;
	}
	.text {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 0;
	}
	.name {
		font-weight: 500;
		white-space: nowrap;
	}
	.grow {
		flex: 1;
	}
	.sub {
		color: var(--fx-m);
		font-size: 11.5px;
		white-space: nowrap;
	}
	.item.pick .sub {
		color: inherit;
		opacity: 0.7;
	}
	.sub.live {
		color: var(--fx-f);
		font-weight: 500;
	}
	.item :global(.end),
	.end {
		margin-left: auto;
		color: var(--fx-m);
	}
	.item.pick .end {
		color: inherit;
	}
	.toggle {
		position: relative;
		flex: none;
		width: 40px;
		height: 22px;
		border-radius: 999px;
		background: var(--fx-field);
	}
	.toggle::after {
		content: '';
		position: absolute;
		top: 3px;
		left: 3px;
		width: 16px;
		height: 16px;
		border-radius: 999px;
		background: var(--fx-p);
	}
	.toggle.on {
		background: var(--fx-f);
	}
	.toggle.on::after {
		left: 21px;
	}
	.spin {
		width: 14px;
		height: 14px;
		margin-left: auto;
		border-radius: 999px;
		background: conic-gradient(var(--fx-f) 0 25%, var(--fx-field) 0);
		mask: radial-gradient(circle, transparent 4px, black 5px);
		animation: spin 800ms linear infinite;
	}
	.spin + :global(.end) {
		margin-left: 0;
	}
	@keyframes spin {
		to {
			rotate: 1turn;
		}
	}
	.track {
		position: relative;
		flex: 1;
		min-width: 0;
		height: 4px;
		border-radius: 999px;
		background: var(--fx-field);
	}
	.track.short {
		flex: none;
		width: 140px;
	}
	.fill {
		position: absolute;
		inset: 0 auto 0 0;
		border-radius: 999px;
		background: var(--fx-f);
	}
	.knob {
		position: absolute;
		top: 50%;
		width: 14px;
		height: 14px;
		border-radius: 999px;
		background: var(--fx-f);
		translate: -50% -50%;
	}
	.screens {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 4px;
		padding: 20px 16px;
	}
	.screen {
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 6px;
		background: var(--fx-field);
		font-size: 11px;
		font-weight: 500;
	}
	.select {
		padding: 5px 10px;
		border-radius: 8px;
		background: var(--fx-s);
		font-size: 12px;
		white-space: nowrap;
	}
	.themes {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 14px;
		padding: 14px;
	}
	.theme {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.preview {
		display: flex;
		aspect-ratio: 16 / 10;
		overflow: hidden;
		border-radius: 10px;
		background: var(--fx-p);
		transition: box-shadow 180ms ease-out;
	}
	.preview.on {
		box-shadow:
			0 0 0 2px var(--fx-p),
			0 0 0 4px var(--fx-f);
	}
	.p-side {
		display: flex;
		flex-direction: column;
		gap: 5px;
		width: 30%;
		padding: 10px 8px;
		background: var(--fx-c);
	}
	.p-main {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 8px;
		padding: 10px;
	}
	.p-card {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 6px;
		padding: 8px;
		border-radius: 6px;
		background: var(--fx-c);
	}
	.preview i {
		display: block;
		width: 80%;
		height: 4px;
		border-radius: 2px;
		background: var(--fx-field);
	}
	.preview .p-side i:first-child,
	.preview .p-main > i {
		height: 6px;
		background: var(--fx-f);
	}
	.walls {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 10px;
		padding: 14px;
	}
	.wall {
		aspect-ratio: 16 / 10;
		border-radius: 8px;
		background: var(--fx-field);
	}
	.wall.on {
		background: var(--flux-wallpaper);
		box-shadow:
			0 0 0 2px var(--fx-c),
			0 0 0 4px var(--fx-f);
	}
	@media (prefers-reduced-motion: reduce) {
		.spin {
			animation: none;
		}
	}
</style>
