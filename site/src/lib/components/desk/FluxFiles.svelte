<script lang="ts">
	import { Icon, icons } from '@facile/muse';
	import { t } from '$lib/i18n';
	import { glyphs } from '$lib/marks';

	let { compact = false, online = false }: { compact?: boolean; online?: boolean } = $props();

	const places = $derived([
		{ name: t('demo.files.recent'), icon: glyphs.recent },
		{ name: t('demo.files.home'), icon: glyphs.home },
		{ name: t('demo.files.documents'), icon: glyphs.document },
		{ name: t('demo.files.downloads'), icon: glyphs.download },
		{ name: t('demo.files.pictures'), icon: glyphs.image },
		{ name: 'Nuage', icon: glyphs.cloud, sel: true }
	]);
	const entries = [
		{ name: 'Code', icon: glyphs.folder, folder: true },
		{ name: 'Design', icon: glyphs.folder, folder: true },
		{ name: 'Photos', icon: glyphs.folder, folder: true },
		{ name: 'Archives', icon: glyphs.folder, folder: true },
		{ name: 'roadmap.pdf', icon: glyphs.document },
		{ name: 'budget-2027.ods', icon: glyphs.sheet },
		{ name: 'notes.md', icon: glyphs.file },
		{ name: 'logo.svg', icon: glyphs.image }
	];
</script>

<!--
@component
The Files window of the hero desktop, drawn from Nautilus as Flux ships it: places on the left, the
Nuage folder open with a path bar, folders and files in a grid, and the item count with the sync state.
-->

<div class="files" class:compact>
	<aside class="fside">
		{#each places as place (place.name)}
			<div class="place" class:sel={place.sel}>
				<Icon icon={place.icon} size={16} />
				<span>{place.name}</span>
			</div>
		{/each}
	</aside>
	<div class="fmain">
		<div class="fhead">
			<span class="nav"><Icon icon={glyphs.back} size={16} /></span>
			<span class="nav dim"><Icon icon={glyphs.forward} size={16} /></span>
			<span class="path"><Icon icon={glyphs.cloud} size={14} /><span>Nuage</span></span>
			<span class="nav"><Icon icon={icons.search} size={16} /></span>
			<span class="nav"><Icon icon={glyphs.grid} size={16} /></span>
		</div>
		<div class="grid">
			{#each entries as entry (entry.name)}
				<div class="entry" class:folder={entry.folder}>
					<Icon icon={entry.icon} size={compact ? 30 : 36} />
					<span class="ename">{entry.name}</span>
				</div>
			{/each}
		</div>
		<div class="fstatus">{t('demo.files.items')} · {online ? t('demo.synced') : t('demo.files.waiting')}</div>
	</div>
</div>

<style>
	.files {
		display: flex;
		height: 100%;
		background: var(--fx-p);
		color: var(--fx-f);
	}
	.fside {
		display: flex;
		flex: none;
		flex-direction: column;
		gap: 2px;
		width: 170px;
		padding: 12px 8px;
		background: var(--fx-c);
	}
	.files.compact .fside {
		display: none;
	}
	.place {
		display: flex;
		align-items: center;
		gap: 10px;
		height: 32px;
		padding: 0 10px;
		border-radius: 8px;
		color: var(--fx-m);
		white-space: nowrap;
	}
	.place span {
		overflow: hidden;
		text-overflow: ellipsis;
		color: var(--fx-f);
	}
	.place.sel {
		background: var(--fx-s);
		color: var(--fx-f);
		font-weight: 500;
	}
	.fmain {
		display: flex;
		flex: 1;
		flex-direction: column;
		min-width: 0;
		padding: 10px 14px 12px;
	}
	.fhead {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-bottom: 18px;
	}
	.nav {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 30px;
		height: 30px;
		border-radius: 8px;
		color: var(--fx-f);
	}
	.nav.dim {
		color: var(--fx-m);
	}
	.path {
		display: flex;
		flex: 1;
		align-items: center;
		gap: 6px;
		height: 30px;
		margin: 0 6px;
		padding: 0 12px;
		border-radius: 8px;
		background: var(--fx-s);
		font-weight: 500;
	}
	.grid {
		display: grid;
		flex: 1;
		align-content: start;
		grid-template-columns: repeat(4, 1fr);
		gap: 18px 8px;
	}
	.files.compact .grid {
		grid-template-columns: repeat(3, 1fr);
	}
	.entry {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		min-width: 0;
		color: var(--fx-m);
	}
	.entry.folder {
		color: var(--fx-f);
	}
	.ename {
		max-width: 100%;
		overflow: hidden;
		color: var(--fx-f);
		font-size: 12px;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.fstatus {
		color: var(--fx-m);
		font-size: 12px;
		text-align: center;
	}
</style>
