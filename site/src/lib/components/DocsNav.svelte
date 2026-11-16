<script lang="ts">
	import { t } from '$lib/i18n';

	let { items, current }: { items: { slug: string; title: string }[]; current?: string } = $props();
</script>

<!--
@component
The docs page list: one link per page, the current one filled.
-->

<nav aria-label={t('docs.nav')}>
	<ol class="flex flex-col gap-1">
		{#each items as item, i (item.slug)}
			<li>
				<a
					href="/docs/{item.slug}"
					aria-current={item.slug === current ? 'page' : undefined}
					class="flex min-h-11 items-center gap-3 rounded-fc-md px-3 text-fc-sm text-fc-fg-muted hover:bg-fc-surface hover:text-fc-fg aria-[current=page]:bg-fc-surface aria-[current=page]:font-medium aria-[current=page]:text-fc-fg"
				>
					<span class="w-5 shrink-0 text-right font-fc-mono text-fc-xs tabular-nums">{i + 1}</span>
					<span>{item.title}</span>
				</a>
			</li>
		{/each}
	</ol>
</nav>
