<script lang="ts">
	let { level }: { level: number } = $props();
</script>

<!--
@component
Wi-Fi signal strength as four rising bars, like the Flux Settings network list.
-->

<span class="signal">
	{#each [5, 8, 11, 14] as height, i (i)}
		<span class="bar" class:on={i < level} style:height="{height}px"></span>
	{/each}
</span>

<style>
	.signal {
		display: inline-flex;
		flex: none;
		align-items: flex-end;
		gap: 2px;
		width: 18px;
		height: 14px;
	}
	.bar {
		width: 3px;
		border-radius: 1px;
		background: var(--fx-field);
		transition: background 200ms ease-out;
	}
	.bar.on {
		background: var(--fx-f);
	}
</style>
