<script lang="ts">
	import { Badge, Button, DownloadSection, Icon, LANDING_HEADING, icons, type DownloadEntry } from '@facile/muse';
	import { t } from '$lib/i18n';
	import { glyphs, marks } from '$lib/marks';
	import { CHECKSUMS, DOWNLOADS, ISO_AARCH64, RELEASES, REPO, TRY_VM } from '$lib/release';
	import Command from '$lib/components/Command.svelte';
	import DesktopDemo from '$lib/components/DesktopDemo.svelte';
	import LandingShell from '$lib/components/LandingShell.svelte';
	import BoiteTile from '$lib/components/tiles/BoiteTile.svelte';
	import KitTile from '$lib/components/tiles/KitTile.svelte';
	import InstallerTile from '$lib/components/tiles/InstallerTile.svelte';
	import LoginTile from '$lib/components/tiles/LoginTile.svelte';
	import RollbackTile from '$lib/components/tiles/RollbackTile.svelte';
	import SettingsTile from '$lib/components/tiles/SettingsTile.svelte';

	const downloads = $derived<DownloadEntry[]>(
		DOWNLOADS.map((entry) => (entry.href === ISO_AARCH64 ? { ...entry, arch: t('arch.arm') } : entry))
	);

	$effect(() => {
		document.querySelector('meta[name="description"]')?.setAttribute('content', t('landing.meta'));
	});

	const width = 'mx-auto w-full max-w-6xl px-4 sm:px-6';
	const lead = 'text-fc-md text-fc-fg-muted sm:text-fc-lg';
</script>

<svelte:head>
	<title>{t('landing.title')}</title>
	<meta name="description" content={t('landing.meta')} />
</svelte:head>

{#snippet ctas()}
	<div class="flex w-full flex-col items-stretch gap-3 sm:w-auto sm:flex-row sm:items-center">
		<Button href="/downloads" size="lg" icon={icons.download}>{t('landing.cta.download')}</Button>
		<Button href={REPO} size="lg" variant="secondary" icon={icons.github} target="_blank" rel="noopener noreferrer">
			{t('landing.cta.github')}
		</Button>
	</div>
{/snippet}

<LandingShell>
	<div class="flex flex-col gap-24 pb-24 md:gap-32 md:pb-32">
		<section class="{width} flex flex-col items-start pt-12 sm:pt-20" aria-labelledby="hero-title">
			<h1
				id="hero-title"
				class="max-w-4xl font-fc-title text-fc-3xl leading-[1.08] font-bold tracking-tight text-balance sm:text-5xl lg:text-6xl"
			>
				{t('landing.slogan')}
			</h1>
			<p class="mt-5 max-w-2xl text-pretty {lead}">{t('landing.subtitle')}</p>
			<div class="mt-8 w-full sm:w-auto">{@render ctas()}</div>
			<DesktopDemo class="mt-12 w-full sm:mt-16" />
		</section>

		<section class="{width} flex flex-col gap-10" aria-labelledby="surfaces-title">
			<div class="flex max-w-2xl flex-col gap-3">
				<h2 id="surfaces-title" class="{LANDING_HEADING} text-balance">{t('surfaces.title')}</h2>
				<p class={lead}>{t('surfaces.line')}</p>
			</div>
			<div class="grid gap-4 md:grid-cols-2 md:gap-6">
				<div class="grid md:col-span-2"><SettingsTile /></div>
				<div class="grid"><LoginTile /></div>
				<div class="grid"><InstallerTile /></div>
			</div>
		</section>

		<section class="{width} grid items-center gap-10 md:grid-cols-2 md:gap-16" aria-labelledby="rollback-title">
			<div class="flex flex-col items-start gap-4">
				<h2 id="rollback-title" class="{LANDING_HEADING} text-balance">{t('rollback.title')}</h2>
				<p class={lead}>{t('rollback.line')}</p>
				<span class="mt-2 inline-flex items-center gap-2 text-fc-sm font-medium text-fc-fg-muted">
					<Icon icon={marks.nix} size={18} class="text-fc-fg" />
					{t('rollback.nix')}
				</span>
			</div>
			<RollbackTile />
		</section>

		<section class="{width} grid items-center gap-10 md:grid-cols-2 md:gap-16" aria-labelledby="boite-title">
			<div class="flex flex-col items-start gap-4 md:order-2">
				<span class="inline-flex size-12 items-center justify-center rounded-fc-lg bg-fc-surface">
					<Icon icon={marks.boite} size={28} />
				</span>
				<h2 id="boite-title" class="{LANDING_HEADING} text-balance">{t('boite.title')}</h2>
				<p class={lead}>{t('boite.line')}</p>
				<Button
					href="https://boite.facile.studio"
					variant="ghost"
					iconRight={icons.external}
					target="_blank"
					rel="noopener noreferrer"
					class="-ml-3"
				>
					{t('boite.link')}
				</Button>
			</div>
			<div class="md:order-1"><BoiteTile /></div>
		</section>

		<section class="{width} grid items-center gap-10 md:grid-cols-2 md:gap-16" aria-labelledby="kit-title">
			<div class="flex flex-col items-start gap-4">
				<span class="inline-flex size-12 items-center justify-center rounded-fc-lg bg-fc-surface">
					<Icon icon={marks.kit} size={28} />
				</span>
				<Badge tone="accent">{t('kit.builtin')}</Badge>
				<h2 id="kit-title" class="{LANDING_HEADING} text-balance">{t('kit.title')}</h2>
				<p class={lead}>{t('kit.line')}</p>
				<Button
					href="https://github.com/FacileStudio/kit"
					variant="ghost"
					iconRight={icons.external}
					target="_blank"
					rel="noopener noreferrer"
					class="-ml-3"
				>
					{t('kit.link')}
				</Button>
			</div>
			<KitTile />
		</section>

		<section class={width} aria-labelledby="mobile-title">
			<div class="flex max-w-2xl flex-col items-start gap-4">
				<span class="inline-flex size-12 items-center justify-center rounded-fc-lg bg-fc-surface">
					<Icon icon={glyphs.phone} size={26} />
				</span>
				<Badge tone="accent">{t('mobile.badge')}</Badge>
				<h2 id="mobile-title" class="{LANDING_HEADING} text-balance">{t('mobile.title')}</h2>
				<p class={lead}>{t('mobile.line')}</p>
			</div>
		</section>

		<section class="{width} grid items-center gap-8 md:grid-cols-2 md:gap-16" aria-labelledby="dev-title">
			<div class="flex flex-col items-start gap-4">
				<span class="inline-flex size-12 items-center justify-center rounded-fc-lg bg-fc-surface">
					<Icon icon={marks.nix} size={26} />
				</span>
				<h2 id="dev-title" class={LANDING_HEADING}>{t('dev.title')}</h2>
				<p class={lead}>{t('dev.line')}</p>
			</div>
			<div class="flex min-w-0 flex-col gap-4">
				<Command command={TRY_VM} />
				<Button href="/docs/vm" variant="ghost" iconRight={icons.arrow} class="-ml-3 self-start">{t('dev.docs')}</Button>
			</div>
		</section>

		<div class={width}>
			<DownloadSection
				mode="device"
				allHref="/downloads"
				{downloads}
				releasesHref={RELEASES}
				checksumsHref={CHECKSUMS}
				headingLevel={2}
			/>
		</div>

		<section class="{width} flex flex-col items-center gap-5 text-center" aria-labelledby="end-title">
			<Badge tone="accent">{t('landing.badge')}</Badge>
			<h2 id="end-title" class="{LANDING_HEADING} text-balance">{t('end.title')}</h2>
			<p class="max-w-2xl text-balance {lead}">{t('end.line')}</p>
			<div class="mt-2 flex w-full justify-center">{@render ctas()}</div>
		</section>
	</div>
</LandingShell>
