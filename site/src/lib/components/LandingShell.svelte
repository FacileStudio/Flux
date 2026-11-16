<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Button, LandingFooter, LandingHeader } from '@facile/muse';
	import { page } from '$app/state';
	import { rememberLocale, t } from '$lib/i18n';
	import { marks } from '$lib/marks';
	import { RELEASES, REPO } from '$lib/release';

	let { children }: { children: Snippet } = $props();

	const path = $derived(page.url.pathname);
	const links = $derived([
		{ href: '/docs', label: t('nav.docs'), active: path.startsWith('/docs') },
		{ href: '/downloads', label: t('nav.downloads'), active: path === '/downloads' },
		{ href: REPO, label: t('nav.github'), external: true }
	]);
	const columns = $derived([
		{
			title: 'Flux',
			links: [
				{ href: '/docs', label: t('nav.docs') },
				{ href: '/downloads', label: t('nav.downloads') },
				{ href: RELEASES, label: t('nav.releases'), external: true }
			]
		},
		{
			title: 'Facile',
			links: [
				{ href: REPO, label: t('nav.github'), external: true },
				{ href: `${REPO}/issues`, label: t('nav.issues'), external: true },
				{ href: 'https://suite.facile.studio', label: t('nav.suite'), external: true }
			]
		}
	]);
</script>

<!--
@component
The public shell of the Flux site: the landing header and footer around every page.
-->

<div class="flex min-h-dvh flex-col overflow-x-clip bg-fc-page text-fc-fg">
	<LandingHeader appName="Flux" icon={marks.flux} {links} skipHref="#main">
		{#snippet actions()}
			<Button href="/downloads" size="sm">{t('landing.cta.download')}</Button>
		{/snippet}
	</LandingHeader>

	<main id="main" class="flex flex-1 flex-col">{@render children()}</main>

	<LandingFooter
		appName="Flux"
		icon={marks.flux}
		tagline={t('landing.line')}
		{columns}
		year={2026}
		legal={{ onLocaleChange: rememberLocale }}
	/>
</div>
