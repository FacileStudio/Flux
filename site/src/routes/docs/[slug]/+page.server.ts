import { error } from '@sveltejs/kit';
import { SLUGS, type Slug } from '$lib/docs';
import { docPage, docTitles } from '$lib/docs/render.server';
import type { EntryGenerator } from './$types';

export const entries: EntryGenerator = () => SLUGS.map((slug) => ({ slug }));

export function load({ params }) {
	const slug = params.slug as Slug;
	if (!SLUGS.includes(slug)) error(404, 'Not found');
	return { slug, page: docPage(slug), titles: docTitles() };
}
