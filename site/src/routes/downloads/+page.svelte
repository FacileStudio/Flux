<script lang="ts">
	import { onMount } from 'svelte';
	import { Badge, Button, Icon, icons } from '@facile/muse';
	import { t } from '$lib/i18n';
	import { marks } from '$lib/marks';
	import {
		CHECKSUMS,
		INIT_FLAKE,
		ISO_AARCH64,
		ISO_X86_64,
		RELEASES,
		RELEASE_NOTES,
		TRY_VM,
		VERIFY,
		VERSION,
		WRITE_USB
	} from '$lib/release';
	import Command from '$lib/components/Command.svelte';
	import LandingShell from '$lib/components/LandingShell.svelte';

	type Arch = 'x86' | 'arm';

	const isos = [
		{ id: 'x86', icon: icons.monitor, href: ISO_X86_64, arch: 'x86_64' },
		{ id: 'arm', icon: marks.cpu, href: ISO_AARCH64, arch: 'aarch64' }
	] as const;

	let device = $state<Arch>('x86');

	onMount(() => {
		detectArch().then((found) => (device = found));
	});

	async function detectArch(): Promise<Arch> {
		const data = (navigator as Navigator & { userAgentData?: { getHighEntropyValues(h: string[]): Promise<{ architecture?: string }> } }).userAgentData;
		if (data) {
			const { architecture } = await data.getHighEntropyValues(['architecture']).catch(() => ({ architecture: undefined }));
			if (architecture) return architecture.startsWith('arm') ? 'arm' : 'x86';
		}
		return /aarch64|arm64/i.test(navigator.userAgent) ? 'arm' : 'x86';
	}

	const steps = [
		{ key: 'verify', command: VERIFY },
		{ key: 'usb', command: WRITE_USB },
		{ key: 'boot', command: null }
	] as const;

	const others = [
		{ key: 'vm', icon: marks.nix, command: TRY_VM },
		{ key: 'nixos', icon: marks.nix, command: INIT_FLAKE },
		{ key: 'boite', icon: marks.boite, command: null }
	] as const;

	const width = 'mx-auto w-full max-w-4xl px-4 sm:px-6';
</script>

<svelte:head><title>{t('downloads.title')} · Flux</title></svelte:head>

<LandingShell>
	<div class="flex flex-col gap-20 pt-16 pb-24 sm:pt-24 md:gap-24">
		<section class="{width} flex flex-col gap-10" aria-labelledby="downloads-title">
			<div class="flex flex-col items-start gap-4">
				<Badge tone="accent">{t('landing.badge')}</Badge>
				<h1 id="downloads-title" class="font-fc-title text-fc-3xl leading-tight font-bold tracking-tight sm:text-5xl">
					{t('downloads.heading')}
				</h1>
				<p class="max-w-2xl text-fc-lg text-balance text-fc-fg-muted">{t('downloads.line')}</p>
				<p class="flex flex-wrap items-center gap-x-4 gap-y-1 text-fc-sm text-fc-fg-muted">
					<span>{t('downloads.version')} <span class="font-fc-mono text-fc-fg">{VERSION}</span></span>
					<a class="underline underline-offset-4 hover:text-fc-fg" href={RELEASE_NOTES}>{t('downloads.notes')}</a>
					<a class="underline underline-offset-4 hover:text-fc-fg" href={RELEASES}>{t('downloads.all')}</a>
				</p>
			</div>

			<div class="grid gap-4 sm:grid-cols-2">
				{#each isos as iso (iso.id)}
					{@const mine = device === iso.id}
					<article class="flex flex-col gap-6 rounded-fc-lg bg-fc-surface p-6">
						<div class="flex items-start justify-between gap-3">
							<span class="flex size-12 items-center justify-center rounded-fc-md bg-fc-bg">
								<Icon icon={iso.icon} size={24} />
							</span>
							{#if mine}<Badge tone="accent">{t('downloads.recommended')}</Badge>{/if}
						</div>
						<div class="flex flex-1 flex-col gap-1">
							<h2 class="font-fc-title text-fc-xl font-semibold">
								{t(`downloads.${iso.id}.name`)} <span class="font-fc-mono text-fc-md font-normal text-fc-fg-muted">{iso.arch}</span>
							</h2>
							<p class="text-fc-md text-fc-fg-muted">{t(`downloads.${iso.id}.line`)}</p>
						</div>
						<div class="flex flex-col gap-3">
							<Button href={iso.href} icon={icons.download} class="w-full justify-center">
								{t('downloads.button')}
							</Button>
							<p class="flex items-center justify-between gap-3 text-fc-xs text-fc-fg-muted">
								<span class="truncate font-fc-mono">flux-{VERSION}-{iso.arch}-linux.iso</span>
								<a class="shrink-0 underline underline-offset-4 hover:text-fc-fg" href={CHECKSUMS}>{t('downloads.checksum')}</a>
							</p>
						</div>
					</article>
				{/each}
			</div>
		</section>

		<section class="{width} flex flex-col gap-8" aria-labelledby="steps-title">
			<h2 id="steps-title" class="font-fc-title text-fc-2xl font-semibold tracking-tight">{t('downloads.steps')}</h2>
			<ol class="flex flex-col gap-8">
				{#each steps as step, index (step.key)}
					<li class="grid grid-cols-[2.5rem_minmax(0,1fr)] gap-x-4 gap-y-3">
						<span class="flex size-10 items-center justify-center rounded-fc-full bg-fc-surface font-fc-mono text-fc-md font-semibold">
							{index + 1}
						</span>
						<div class="flex min-w-0 flex-col gap-2 pt-1.5">
							<h3 class="text-fc-lg font-semibold">{t(`downloads.${step.key}.title`)}</h3>
							<p class="text-fc-md text-fc-fg-muted">{t(`downloads.${step.key}.line`)}</p>
						</div>
						{#if step.command}
							<Command command={step.command} class="col-span-2 sm:col-span-1 sm:col-start-2" />
						{:else}
							<div class="col-span-2 sm:col-span-1 sm:col-start-2">
								<Button href="/docs/install" variant="secondary" iconRight={icons.arrow}>{t('downloads.next')}</Button>
							</div>
						{/if}
					</li>
				{/each}
			</ol>
		</section>

		<section class="{width} flex flex-col gap-8" aria-labelledby="other-title">
			<h2 id="other-title" class="font-fc-title text-fc-2xl font-semibold tracking-tight">{t('downloads.other')}</h2>
			<div class="flex flex-col gap-4">
				{#each others as other (other.key)}
					<article class="grid items-center gap-4 rounded-fc-lg bg-fc-surface p-6 md:grid-cols-[minmax(0,1fr)_minmax(0,1.5fr)] md:gap-8">
						<div class="flex min-w-0 items-start gap-4">
							<span class="flex size-12 shrink-0 items-center justify-center rounded-fc-md bg-fc-bg"><Icon icon={other.icon} size={24} /></span>
							<div class="flex min-w-0 flex-col gap-1">
								<h3 class="text-fc-lg font-semibold">{t(`downloads.${other.key}.title`)}</h3>
								<p class="text-fc-md text-fc-fg-muted">{t(`downloads.${other.key}.line`)}</p>
							</div>
						</div>
						{#if other.command}
							<Command command={other.command} />
						{:else}
							<div class="md:justify-self-end">
								<Button href="https://boite.facile.studio" variant="secondary" iconRight={icons.external}>boite.facile.studio</Button>
							</div>
						{/if}
					</article>
				{/each}
			</div>
		</section>
	</div>
</LandingShell>
