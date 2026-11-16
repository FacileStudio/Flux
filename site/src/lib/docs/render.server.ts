import { LOCALES, type Locale } from '@facile/muse';
import { marked } from 'marked';
import { SLUGS, type Slug } from './index';

const sources = import.meta.glob('./*/*.md', { query: '?raw', import: 'default', eager: true }) as Record<string, string>;

/** One docs page in one locale: its title, from the first `# ` line, and the rest as HTML. */
export interface DocPage {
	title: string;
	html: string;
}

function source(locale: Locale, slug: Slug): string {
	const text = sources[`./${locale}/${slug}.md`] ?? sources[`./fr/${slug}.md`];
	if (text === undefined) throw new Error(`docs: missing ${slug}.md`);
	return text;
}

function render(text: string): DocPage {
	const [first, ...rest] = text.trimStart().split('\n');
	const title = first.replace(/^#\s+/, '').trim();
	const html = marked.parse(rest.join('\n'), { async: false });
	return { title, html };
}

/** A page rendered in every locale the site ships. */
export function docPage(slug: Slug): Record<Locale, DocPage> {
	return Object.fromEntries(LOCALES.map((locale) => [locale, render(source(locale, slug))])) as Record<Locale, DocPage>;
}

/** Every page's title in every locale, in reading order, for the navigation. */
export function docTitles(): Record<Locale, { slug: Slug; title: string }[]> {
	return Object.fromEntries(
		LOCALES.map((locale) => [locale, SLUGS.map((slug) => ({ slug, title: render(source(locale, slug)).title }))])
	) as Record<Locale, { slug: Slug; title: string }[]>;
}
