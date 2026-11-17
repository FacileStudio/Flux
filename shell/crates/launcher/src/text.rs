//! The launcher's strings in English and French.

use shell_common::locale::{self, Lang};

pub struct Text {
    pub search: &'static str,
    pub empty: &'static str,
    pub failed: &'static str,
}

const EN: Text = Text {
    search: "Search apps",
    empty: "No app matches",
    failed: "{name} could not start.",
};

const FR: Text = Text {
    search: "Rechercher une application",
    empty: "Aucune application ne correspond",
    failed: "Impossible de lancer {name}.",
};

/// The strings for the current language.
pub fn get() -> &'static Text {
    match locale::current() {
        Lang::En => &EN,
        Lang::Fr => &FR,
    }
}
