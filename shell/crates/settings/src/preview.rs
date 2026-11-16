//! Design-check mode: `FLUX_SETTINGS_PREVIEW=<lang>:<page>[:dark]` opens a page
//! filled with sample data and touches no system service or file.

use std::sync::OnceLock;

/// What a preview run shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preview {
    pub language: String,
    pub page: String,
    pub dark: bool,
}

static PREVIEW: OnceLock<Option<Preview>> = OnceLock::new();

/// Read `lang:page[:dark]`; a value without a colon is a page in English.
pub fn parse(value: &str) -> Option<Preview> {
    let mut parts = value.split(':').map(str::trim);
    let first = parts.next().filter(|part| !part.is_empty())?;
    let (language, page) = match parts.next() {
        Some(page) if !page.is_empty() => (first, page),
        _ => ("en", first),
    };
    Some(Preview {
        language: language.to_string(),
        page: page.to_string(),
        dark: parts.next() == Some("dark"),
    })
}

/// The preview this run is in, from the environment, or `None` for a real run.
pub fn current() -> Option<&'static Preview> {
    PREVIEW
        .get_or_init(|| {
            std::env::var("FLUX_SETTINGS_PREVIEW")
                .ok()
                .as_deref()
                .and_then(parse)
        })
        .as_ref()
}

/// Whether this run shows sample data.
pub fn active() -> bool {
    current().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_values_parse() {
        let full = parse("fr:about:dark").expect("full value");
        assert_eq!(
            (full.language.as_str(), full.page.as_str(), full.dark),
            ("fr", "about", true)
        );
        let bare = parse("appearance").expect("bare page");
        assert_eq!(
            (bare.language.as_str(), bare.page.as_str(), bare.dark),
            ("en", "appearance", false)
        );
        assert_eq!(parse(""), None);
    }
}
