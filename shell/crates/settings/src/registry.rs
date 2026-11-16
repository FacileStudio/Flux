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

#[cfg(any(
    feature = "page-network",
    feature = "page-bluetooth",
    feature = "page-audio",
    feature = "page-display",
    feature = "page-power",
    feature = "page-system"
))]
macro_rules! external {
    ($krate:ident) => {
        Page {
            id: $krate::ID,
            icon: $krate::ICON,
            title: $krate::title,
            build: $krate::page,
        }
    };
}

/// The pages in the order the navigation lists them.
pub const PAGES: &[Page] = &[
    #[cfg(feature = "page-network")]
    external!(settings_network),
    #[cfg(feature = "page-bluetooth")]
    external!(settings_bluetooth),
    #[cfg(feature = "page-audio")]
    external!(settings_audio),
    #[cfg(feature = "page-display")]
    external!(settings_display),
    Page {
        id: appearance::ID,
        icon: appearance::ICON,
        title: appearance::title,
        build: appearance::page,
    },
    #[cfg(feature = "page-power")]
    external!(settings_power),
    #[cfg(feature = "page-system")]
    external!(settings_system),
    Page {
        id: about::ID,
        icon: about::ICON,
        title: about::title,
        build: about::page,
    },
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

/// Whether a page with this `title` and `id` answers `query`: every word of the query starts a word of either.
pub fn matches(query: &str, title: &str, id: &str) -> bool {
    let haystack = fold(&format!("{title} {id}"));
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
    fn page_ids_are_unique() {
        let mut ids: Vec<&str> = PAGES.iter().map(|page| page.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), PAGES.len());
    }
}
