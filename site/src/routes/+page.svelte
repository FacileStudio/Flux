<script lang="ts">
	import {
		Badge,
		Button,
		DownloadSection,
		FeatureSplit,
		Icon,
		LANDING_HEADING,
		WorksWith,
		icons,
		type FeatureItem,
		type WorksWithApp
	} from '@facile/muse';
	import { t } from '$lib/i18n';
	import { marks } from '$lib/marks';
	import { CHECKSUMS, DOWNLOADS, RELEASES, REPO, TRY_VM } from '$lib/release';
	import Command from '$lib/components/Command.svelte';
	import DesktopDemo from '$lib/components/DesktopDemo.svelte';
	import LandingShell from '$lib/components/LandingShell.svelte';
	import BoîteTile from '$lib/components/tiles/BoîteTile.svelte';
	import RollbackTile from '$lib/components/tiles/RollbackTile.svelte';

	const FEATURES = [
		['columns', marks.columns],
		['shell', marks.palette],
		['config', marks.nix],
		['terminal', marks.terminal],
		['arch', marks.cpu]
	] as const;

	const features = $derived<FeatureItem[]>(
		FEATURES.map(([key, icon]) => ({
			icon,
			title: t(`feature.${key}.title`),
			text: t(`feature.${key}.text`)
		}))
	);

	const apps = $derived<WorksWithApp[]>([
		{ name: 'Sablier', icon: marks.sablier, line: t('with.sablier'), href: 'https://sablier.facile.studio' },
		{ name: 'Nuage', icon: marks.nuage, line: t('with.nuage'), href: 'https://nuage.facile.studio' },
		{ name: 'Casier', icon: marks.casier, line: t('with.casier'), href: 'https://casier.facile.studio' },
		{ name: 'Bulle', icon: marks.bulle, line: t('with.bulle'), href: 'https://bulle.facile.studio' },
		{ name: 'Boîte', icon: marks.boite, line: t('with.boite'), href: 'https://boite.facile.studio' }
	]);

	$effect(() => {
		document.querySelector('meta[name="description"]')?.setAttribute('content', t('landing.meta'));
	});

	const width = 'mx-auto w-full max-w-5xl px-4 sm:px-6';
</script>

<svelte:head>
	<title>{t('landing.title')}</title>
	<meta name="description" content={t('landing.meta')} />
</svelte:head>

{#snippet ctas()}
	<div class="flex w-full flex-col items-stretch justify-center gap-3 sm:flex-row sm:items-center">
		<Button href="/downloads" size="lg" icon={icons.download}>{t('landing.cta.download')}</Button>
		<Button href={REPO} size="lg" variant="secondary" icon={icons.github} target="_blank" rel="noopener noreferrer">
			{t('landing.cta.github')}
		</Button>
	</div>
{/snippet}

<LandingShell>
	<div class="flex flex-col gap-20 pb-24 md:gap-28">
		<section class="{width} flex flex-col items-center pt-14 text-center sm:pt-20">
			<Badge tone="accent">{t('landing.badge')}</Badge>
			<h1 class="mt-6 max-w-3xl font-fc-title text-fc-3xl leading-tight font-bold tracking-tight text-balance sm:text-5xl">
				{t('landing.slogan')}
			</h1>
			<p class="mt-5 max-w-2xl text-fc-lg text-balance text-fc-fg-muted">{t('landing.subtitle')}</p>
			<div class="mt-8 w-full">{@render ctas()}</div>
			<DesktopDemo class="mt-12 w-full sm:mt-16" />
		</section>

		<section aria-labelledby="tiles-title" class="{width} scroll-mt-24">
			<h2 id="tiles-title" class="mb-10 text-center {LANDING_HEADING}">{t('tiles.title')}</h2>
			<div class="grid gap-4 md:grid-cols-2 md:gap-6">
				<div class="grid"><RollbackTile /></div>
				<div class="grid"><BoîteTile /></div>
			</div>
		</section>

		<FeatureSplit class={width} title={t('features.title')} description={t('features.line')} {features} />

		<WorksWith class={width} title={t('with.title')} description={t('with.line')} {apps} />

		<section aria-labelledby="dev-title" class="{width} grid gap-8 md:grid-cols-2 md:items-center">
			<div class="flex flex-col items-start gap-4">
				<span class="inline-flex size-12 items-center justify-center rounded-fc-lg bg-fc-surface">
					<Icon icon={marks.nix} size={28} />
				</span>
				<h2 id="dev-title" class={LANDING_HEADING}>{t('dev.title')}</h2>
				<p class="text-fc-md text-fc-fg-muted">{t('dev.line')}</p>
			</div>
			<div class="flex min-w-0 flex-col gap-4">
				<Command command={TRY_VM} />
				<Button href="/docs/vm" variant="ghost" iconRight={icons.arrow} class="self-start">{t('dev.docs')}</Button>
			</div>
		</section>

		<div class={width}>
			<DownloadSection
				mode="device"
				allHref="/downloads"
				downloads={DOWNLOADS}
				releasesHref={RELEASES}
				checksumsHref={CHECKSUMS}
				headingLevel={3}
			/>
		</div>

		<section class="{width} flex flex-col items-center gap-5 text-center">
			<Badge tone="accent">{t('landing.badge')}</Badge>
			<h2 class="{LANDING_HEADING} text-balance">{t('end.title')}</h2>
			<p class="max-w-2xl text-fc-md text-balance text-fc-fg-muted">{t('end.line')}</p>
			<div class="mt-2 w-full">{@render ctas()}</div>
		</section>
	</div>
</LandingShell>
