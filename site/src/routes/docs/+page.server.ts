import { docTitles } from '$lib/docs/render.server';

export function load() {
	return { titles: docTitles() };
}
