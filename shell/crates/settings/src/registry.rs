//! Every page the app can show, in navigation order, and how search picks them.
//!
//! A page crate exposes `ID`, `ICON`, `title()` and `page()`. Pages owned by
//! other crates are compiled in behind a `page-<name>` feature, so this crate
//! builds before they exist.

use crate::{about, appearance};

/// One entry of the navigation: what to call it and how to build it.
pub struct Page {
    pub id: &'static str,
    pub icon: &'static str,
    pub title: fn() -> String,
    pub build: fn() -> gtk4::Widget,
}

macro_rules! entry {
    ($($module:ident)::+) => {
        Page {
            id: $($module)::+::ID,
            icon: $($module)::+::ICON,
            title: $($module)::+::title,
            build: $($module)::+::page,
        }
    };
}

/// The pages in the order the navigation lists them: connections, devices, look, power, the machine, then About.
pub const PAGES: &[Page] = &[
    #[cfg(feature = "page-network")]
    entry!(settings_network),
    #[cfg(feature = "page-bluetooth")]
    entry!(settings_bluetooth),
    #[cfg(feature = "page-audio")]
    entry!(settings_audio),
    #[cfg(feature = "page-display")]
    entry!(settings_display::display),
    #[cfg(feature = "page-display")]
    entry!(settings_display::input),
    entry!(appearance),
    #[cfg(feature = "page-power")]
    entry!(settings_power),
    #[cfg(feature = "page-system")]
    entry!(settings_system::system),
    #[cfg(feature = "page-system")]
    entry!(settings_system::users),
    #[cfg(feature = "page-system")]
    entry!(settings_system::updates),
    entry!(about),
];

/// Lower-case `text` and drop the accents French puts on Latin letters, so "reglages" finds "Réglages".
pub fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|letter| match letter {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ÿ' => 'y',
            other => other,
        })
        .collect()
}

/// Extra words that lead to the page called `id`, for controls people look for by name, in English and French.
pub fn keywords(id: &str) -> &'static str {
    match id {
        "input" => "scroll direction wheel touchpad défilement sens molette pavé",
        "system" => "software apps gaming nvidia language logiciels jeux langue",
        _ => "",
    }
}

/// Whether a page with this `title` and `id` answers `query`: every word of the query starts a word of the title, the id or the page's [`keywords`].
pub fn matches(query: &str, title: &str, id: &str) -> bool {
    let haystack = fold(&format!("{title} {id} {}", keywords(id)));
    let words: Vec<&str> = haystack
        .split(|letter: char| !letter.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    fold(query)
        .split_whitespace()
        .all(|needle| words.iter().any(|word| word.starts_with(needle)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_ignores_case_and_accents() {
        assert!(matches("apparence", "Apparence", "appearance"));
        assert!(matches("A PROPOS", "À propos", "about"));
        assert!(matches("about", "À propos", "about"));
        assert!(matches("  ", "Network", "network"));
    }

    #[test]
    fn search_matches_word_starts_only() {
        assert!(matches("pro", "À propos", "about"));
        assert!(!matches("opos", "À propos", "about"));
        assert!(matches("wi fi", "Wi-Fi", "network"));
        assert!(!matches("wallpaper", "About", "about"));
    }

    #[test]
    fn search_finds_scroll_direction() {
        for query in ["scroll", "direction", "défilement", "defilement", "sens", "molette", "wheel", "scroll dir"] {
            assert!(matches(query, "Keyboard and mouse", "input"), "{query}");
            assert!(!matches(query, "About", "about"), "{query}");
        }
    }

    #[test]
    fn page_ids_are_unique() {
        let mut ids: Vec<&str> = PAGES.iter().map(|page| page.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), PAGES.len());
    }
}
