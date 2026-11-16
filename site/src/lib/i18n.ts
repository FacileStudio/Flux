import { LOCALE_COOKIE, createTranslator, matchLocale, resolveLocale, type Locale } from '@facile/muse';
import { catalogs } from './messages';

let active: () => Locale = () => 'fr';

/** Points the site's translator at the locale the root layout provides. */
export function bindSite(source: () => Locale): void {
	active = source;
}

/** The locale the site is drawn in. */
export function locale(): Locale {
	return resolveLocale(active());
}

/** The visitor's locale: the one picked in the footer, then the first browser language the site speaks, else none. */
export function visitorLocale(): Locale | null {
	const prefix = `${LOCALE_COOKIE}=`;
	const cookie = document.cookie.split('; ').find((part) => part.startsWith(prefix));
	const tags = [cookie ? decodeURIComponent(cookie.slice(prefix.length)) : null, ...navigator.languages];
	for (const tag of tags) {
		const found = matchLocale(tag);
		if (found) return found;
	}
	return null;
}

/** Persists the locale picked in the footer for the next visit. */
export function rememberLocale(next: Locale): void {
	document.cookie = `${LOCALE_COOKIE}=${next}; path=/; max-age=31536000; samesite=lax`;
}

/** Translates a site message in the current locale. */
export const t = createTranslator(catalogs, () => active());
