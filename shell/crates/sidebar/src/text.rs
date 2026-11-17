//! The sidebar's strings in English and French.

use shell_common::locale::{self, Lang};

pub struct Text {
    pub notifications: &'static str,
    pub empty: &'static str,
    pub clear_all: &'static str,
    pub dismiss: &'static str,
    pub just_now: &'static str,
    pub light_mode: &'static str,
    pub dark_mode: &'static str,
    pub wallpaper: &'static str,
    pub add_from_file: &'static str,
    pub add_title: &'static str,
    pub images: &'static str,
}

const EN: Text = Text {
    notifications: "Notifications",
    empty: "No notifications",
    clear_all: "Clear all",
    dismiss: "Dismiss",
    just_now: "just now",
    light_mode: "Light mode",
    dark_mode: "Dark mode",
    wallpaper: "Wallpaper",
    add_from_file: "Add from file",
    add_title: "Add a wallpaper",
    images: "Images",
};

const FR: Text = Text {
    notifications: "Notifications",
    empty: "Aucune notification",
    clear_all: "Tout effacer",
    dismiss: "Fermer",
    just_now: "à l'instant",
    light_mode: "Mode clair",
    dark_mode: "Mode sombre",
    wallpaper: "Fond d'écran",
    add_from_file: "Ajouter un fichier",
    add_title: "Ajouter un fond d'écran",
    images: "Images",
};

/// The strings for the current language.
pub fn get() -> &'static Text {
    match locale::current() {
        Lang::En => &EN,
        Lang::Fr => &FR,
    }
}

/// The English strings, for checks that must not depend on the locale.
#[cfg(test)]
pub fn english() -> &'static Text {
    &EN
}
