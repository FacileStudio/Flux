<script lang="ts">
	import { Badge, Button, DownloadSection, icons } from '@facile/muse';
	import { t } from '$lib/i18n';
	import { CHECKSUMS, DOWNLOADS, RELEASES, VERIFY, WRITE_USB } from '$lib/release';
	import Command from '$lib/components/Command.svelte';
	import LandingShell from '$lib/components/LandingShell.svelte';
</script>

<svelte:head><title>{t('downloads.title')} · Flux</title></svelte:head>

<LandingShell>
	<section
		class="mx-auto flex w-full max-w-2xl flex-col items-start gap-10 px-4 py-16 sm:px-6 sm:py-24"
		aria-labelledby="downloads-title"
	>
		<div class="flex flex-col items-start gap-4">
			<Badge tone="accent">{t('landing.badge')}</Badge>
			<h1 id="downloads-title" class="font-fc-title text-fc-3xl font-bold tracking-tight sm:text-5xl">
				{t('downloads.heading')}
			</h1>
			<p class="text-fc-md text-fc-fg-muted">{t('downloads.line')}</p>
		</div>

		<DownloadSection class="w-full" downloads={DOWNLOADS} releasesHref={RELEASES} checksumsHref={CHECKSUMS} />

		<div class="flex w-full flex-col gap-3">
			<h2 class="font-fc-title text-fc-xl font-semibold">{t('downloads.verify.title')}</h2>
			<p class="text-fc-md text-fc-fg-muted">{t('downloads.verify.line')}</p>
			<Command command={VERIFY} />
		</div>

		<div class="flex w-full flex-col gap-3">
			<h2 class="font-fc-title text-fc-xl font-semibold">{t('downloads.usb.title')}</h2>
			<p class="text-fc-md text-fc-fg-muted">{t('downloads.usb.line')}</p>
			<Command command={WRITE_USB} />
			<p class="text-fc-md text-fc-fg-muted">{t('downloads.usb.gui')}</p>
		</div>

		<Button href="/docs" variant="secondary" iconRight={icons.arrow}>{t('downloads.next')}</Button>
	</section>
</LandingShell>
