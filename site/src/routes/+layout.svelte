<script lang="ts">
	import '../app.css';
	import '$lib/marks';
	import { provideLocale, type Locale } from '@facile/muse';
	import { bindSite, visitorLocale } from '$lib/i18n';

	let { children } = $props();

	let visitor = $state<Locale | null>(null);
	const i18n = provideLocale(() => visitor ?? 'fr');
	bindSite(() => i18n.locale);

	$effect(() => {
		visitor = visitorLocale();
	});
</script>

{@render children()}
