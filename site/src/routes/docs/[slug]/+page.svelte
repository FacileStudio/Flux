<script lang="ts">
	import { Button, Icon, icons } from '@facile/muse';
	import { NIX_PAGES, type Slug } from '$lib/docs';
	import { locale, t } from '$lib/i18n';
	import { marks } from '$lib/marks';
	import DocsNav from '$lib/components/DocsNav.svelte';
	import LandingShell from '$lib/components/LandingShell.svelte';

	let { data } = $props();

	const doc = $derived(data.page[locale()]);
	const items = $derived(data.titles[locale()]);
	const at = $derived(items.findIndex((item) => item.slug === data.slug));
	const prev = $derived(at > 0 ? items[at - 1] : null);
	const next = $derived(at < items.length - 1 ? items[at + 1] : null);
	const nix = $derived(NIX_PAGES.includes(data.slug as Slug));
</script>

<svelte:head><title>{doc.title} · {t('docs.title')}</title></svelte:head>

<LandingShell>
	<div class="mx-auto grid w-full max-w-5xl gap-10 px-4 py-12 sm:px-6 sm:py-16 lg:grid-cols-[15rem_minmax(0,1fr)]">
		<aside class="hidden lg:block">
			<div class="sticky top-24"><DocsNav {items} current={data.slug} /></div>
		</aside>

		<article class="flex min-w-0 flex-col gap-10">
			<a href="/docs" class="inline-flex min-h-11 items-center gap-2 self-start text-fc-sm text-fc-fg-muted hover:text-fc-fg lg:hidden">
				<Icon icon={icons.chevronLeft} size={16} />{t('docs.all')}
			</a>
			<div class="doc">
				<h1 class="flex items-center gap-3">
					{#if nix}<Icon icon={marks.nix} size={32} />{/if}
					<span>{doc.title}</span>
				</h1>
				{@html doc.html}
			</div>

			<div class="flex flex-col gap-3 sm:flex-row sm:justify-between">
				{#if prev}
					<Button href="/docs/{prev.slug}" variant="secondary" icon={icons.chevronLeft}>{prev.title}</Button>
				{:else}<span></span>{/if}
				{#if next}
					<Button href="/docs/{next.slug}" variant="secondary" iconRight={icons.arrow}>{next.title}</Button>
				{/if}
			</div>
		</article>
	</div>
</LandingShell>
