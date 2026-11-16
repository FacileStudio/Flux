import type { Catalogs, Message, Row } from '@facile/muse';
import { demo } from './demo';
import { page } from './page';
import { pages } from './pages';

const rows = { ...page, ...demo, ...pages };

export type SiteKey = keyof typeof rows;
type SiteCatalog = Record<SiteKey, Message>;

function column(index: number): SiteCatalog {
	const entries = Object.entries(rows as Record<string, Row>).map(([key, row]) => [key, row[index]]);
	return Object.fromEntries(entries) as SiteCatalog;
}

/** The site's catalogs: English is the typed base, and every row carries all four locales. */
export const catalogs: Catalogs<SiteCatalog> = {
	en: column(0),
	fr: column(1),
	es: column(2),
	de: column(3)
};
