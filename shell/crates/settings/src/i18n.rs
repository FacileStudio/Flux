//! The app's strings in English and French. Other languages read English.

use std::sync::OnceLock;

pub struct Text {
    pub window_title: &'static str,
    pub search: &'static str,
    pub no_results: &'static str,
    pub appearance_title: &'static str,
    pub appearance_description: &'static str,
    pub theme: &'static str,
    pub light: &'static str,
    pub dark: &'static str,
    pub wallpaper: &'static str,
    pub add_wallpaper: &'static str,
    pub add_wallpaper_title: &'static str,
    pub images: &'static str,
    pub wallpaper_none: &'static str,
    pub wallpaper_none_body: &'static str,
    pub wallpaper_failed: &'static str,
    pub loading: &'static str,
    pub interface: &'static str,
    pub text_size: &'static str,
    pub text_size_body: &'static str,
    pub cursor_size: &'static str,
    pub cursor_sizes: [&'static str; 4],
    pub reduce_motion: &'static str,
    pub reduce_motion_body: &'static str,
    pub interface_missing: &'static str,
    pub interface_missing_body: &'static str,
    pub about_title: &'static str,
    pub about_description: &'static str,
    pub system: &'static str,
    pub hardware: &'static str,
    pub links: &'static str,
    pub version: &'static str,
    pub hostname: &'static str,
    pub processor: &'static str,
    pub memory: &'static str,
    pub disk: &'static str,
    pub graphics: &'static str,
    pub kernel: &'static str,
    pub unknown: &'static str,
    pub copy_details: &'static str,
    pub copied: &'static str,
    pub website: &'static str,
    pub website_body: &'static str,
    pub docs: &'static str,
    pub docs_body: &'static str,
    pub report_bug: &'static str,
    pub report_bug_body: &'static str,
    pub open: &'static str,
}

pub const EN: Text = Text {
    window_title: "Settings",
    search: "Search settings",
    no_results: "No page matches",
    appearance_title: "Appearance",
    appearance_description: "Theme, wallpaper, text size and motion.",
    theme: "Theme",
    light: "Light",
    dark: "Dark",
    wallpaper: "Wallpaper",
    add_wallpaper: "Add from file",
    add_wallpaper_title: "Add a wallpaper",
    images: "Images",
    wallpaper_none: "No wallpapers yet",
    wallpaper_none_body: "Add an image, or put some in ~/Pictures/Wallpapers.",
    wallpaper_failed: "The wallpaper could not be changed: {error}",
    loading: "Loading…",
    interface: "Text and motion",
    text_size: "Text size",
    text_size_body: "Scales text in every app.",
    cursor_size: "Cursor size",
    cursor_sizes: ["Default", "Medium", "Large", "Larger"],
    reduce_motion: "Reduce motion",
    reduce_motion_body: "Turns animations off across the desktop.",
    interface_missing: "Not available",
    interface_missing_body: "This system has no desktop interface settings to change.",
    about_title: "About",
    about_description: "This computer and the Flux release it runs.",
    system: "System",
    hardware: "Hardware",
    links: "Links",
    version: "Version",
    hostname: "Device name",
    processor: "Processor",
    memory: "Memory",
    disk: "Disk",
    graphics: "Graphics",
    kernel: "Kernel",
    unknown: "Unknown",
    copy_details: "Copy details",
    copied: "Copied",
    website: "Website",
    website_body: "flux.facile.studio",
    docs: "Documentation",
    docs_body: "Guides for everyday use.",
    report_bug: "Report a bug",
    report_bug_body: "Opens an issue on GitHub.",
    open: "Open",
};

pub const FR: Text = Text {
    window_title: "Réglages",
    search: "Rechercher un réglage",
    no_results: "Aucune page ne correspond",
    appearance_title: "Apparence",
    appearance_description: "Thème, fond d'écran, taille du texte et animations.",
    theme: "Thème",
    light: "Clair",
    dark: "Sombre",
    wallpaper: "Fond d'écran",
    add_wallpaper: "Ajouter un fichier",
    add_wallpaper_title: "Ajouter un fond d'écran",
    images: "Images",
    wallpaper_none: "Aucun fond d'écran",
    wallpaper_none_body: "Ajoutez une image, ou placez-en dans ~/Pictures/Wallpapers.",
    wallpaper_failed: "Le fond d'écran n'a pas pu changer : {error}",
    loading: "Chargement…",
    interface: "Texte et animations",
    text_size: "Taille du texte",
    text_size_body: "Agrandit le texte dans toutes les applications.",
    cursor_size: "Taille du curseur",
    cursor_sizes: ["Par défaut", "Moyen", "Grand", "Très grand"],
    reduce_motion: "Réduire les animations",
    reduce_motion_body: "Désactive les animations sur tout le bureau.",
    interface_missing: "Indisponible",
    interface_missing_body: "Ce système n'a pas de réglages d'interface à modifier.",
    about_title: "À propos",
    about_description: "Cet ordinateur et la version de Flux qu'il utilise.",
    system: "Système",
    hardware: "Matériel",
    links: "Liens",
    version: "Version",
    hostname: "Nom de l'appareil",
    processor: "Processeur",
    memory: "Mémoire",
    disk: "Disque",
    graphics: "Graphismes",
    kernel: "Noyau",
    unknown: "Inconnu",
    copy_details: "Copier les détails",
    copied: "Copié",
    website: "Site web",
    website_body: "flux.facile.studio",
    docs: "Documentation",
    docs_body: "Les guides du quotidien.",
    report_bug: "Signaler un bug",
    report_bug_body: "Ouvre un ticket sur GitHub.",
    open: "Ouvrir",
};

static LANGUAGE: OnceLock<String> = OnceLock::new();

/// Fix the language for the rest of the run, ahead of the environment. Later calls do nothing.
pub fn force(code: &str) {
    let _ = LANGUAGE.set(code.to_string());
}

/// The two-letter code of the first locale variable set, the order glibc reads them in.
pub fn from_env(lookup: impl Fn(&str) -> Option<String>) -> String {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|name| lookup(name))
        .find(|value| !value.is_empty())
        .map(|value| {
            value
                .chars()
                .take(2)
                .collect::<String>()
                .to_ascii_lowercase()
        })
        .unwrap_or_else(|| "en".to_string())
}

/// The strings for the language this run uses.
pub fn text() -> &'static Text {
    let code = LANGUAGE.get_or_init(|| from_env(|name| std::env::var(name).ok()));
    match code.as_str() {
        "fr" => &FR,
        _ => &EN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_set_locale_variable_wins() {
        let env = |name: &str| match name {
            "LC_ALL" => Some(String::new()),
            "LC_MESSAGES" => Some("fr_FR.UTF-8".to_string()),
            "LANG" => Some("en_US.UTF-8".to_string()),
            _ => None,
        };
        assert_eq!(from_env(env), "fr");
        assert_eq!(from_env(|_| None), "en");
    }
}
