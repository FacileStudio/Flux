<script lang="ts">
	import { Badge } from '@facile/muse';
	import { locale, t } from '$lib/i18n';
	import DocsNav from '$lib/components/DocsNav.svelte';
	import LandingShell from '$lib/components/LandingShell.svelte';

	let { data } = $props();
</script>

<svelte:head><title>{t('docs.title')} · Flux</title></svelte:head>

<LandingShell>
	<section class="mx-auto flex w-full max-w-2xl flex-col gap-10 px-4 py-16 sm:px-6 sm:py-24" aria-labelledby="docs-title">
		<div class="flex flex-col items-start gap-4">
			<Badge tone="accent">{t('landing.badge')}</Badge>
			<h1 id="docs-title" class="font-fc-title text-fc-3xl font-bold tracking-tight sm:text-5xl">{t('docs.heading')}</h1>
			<p class="text-fc-md text-fc-fg-muted">{t('docs.line')}</p>
		</div>
		<DocsNav items={data.titles[locale()]} />
	</section>
</LandingShell>
