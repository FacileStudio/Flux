<script lang="ts">
	import { CopyButton } from '@facile/muse';

	let { command, class: className = '' }: { command: string; class?: string } = $props();
</script>

<!--
@component
A shell command on the inverse surface, with a copy button. Multi-line commands keep their lines.
-->

<div class="flex min-w-0 items-start gap-2 rounded-fc-lg bg-fc-inverse py-2 pr-2 pl-4 text-fc-on-inverse {className}">
	<pre class="min-w-0 flex-1 overflow-x-auto py-2 font-fc-mono text-fc-sm leading-relaxed whitespace-pre"><code>{command}</code></pre>
	<CopyButton text={command} class="shrink-0 text-fc-on-inverse-muted hover:text-fc-on-inverse" />
</div>
