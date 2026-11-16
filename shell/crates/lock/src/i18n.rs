//! The lock screen's strings in English and French. Other languages read English.

pub struct Text {
    pub hint: &'static str,
    pub placeholder: &'static str,
    pub checking: &'static str,
    pub wrong: &'static str,
    pub unavailable: &'static str,
    pub caps_lock: &'static str,
    pub charging: &'static str,
    pub preview: &'static str,
}

pub const EN: Text = Text {
    hint: "Press any key to unlock",
    placeholder: "Password",
    checking: "Checking",
    wrong: "That password is not right. Try again.",
    unavailable: "Flux cannot check passwords right now. The session stays locked.",
    caps_lock: "Caps Lock is on",
    charging: "charging",
    preview: "Preview: nothing is locked",
};

pub const FR: Text = Text {
    hint: "Appuyez sur une touche pour déverrouiller",
    placeholder: "Mot de passe",
    checking: "Vérification",
    wrong: "Ce mot de passe n'est pas le bon. Réessayez.",
    unavailable: "Flux ne peut pas vérifier les mots de passe pour l'instant. La session reste verrouillée.",
    caps_lock: "Verr. Maj est activé",
    charging: "en charge",
    preview: "Aperçu : rien n'est verrouillé",
};

/// The strings for a language code such as `fr`.
pub fn text(code: &str) -> &'static Text {
    match code {
        "fr" => &FR,
        _ => &EN,
    }
}

/// The language code from a locale variable such as `fr_FR.UTF-8`.
pub fn code_of(locale: &str) -> &str {
    locale.split(['_', '.', '@']).next().unwrap_or("")
}

/// The strings for the session's locale (`LC_ALL`, `LC_MESSAGES`, `LANG`).
pub fn current() -> &'static Text {
    let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()))
        .unwrap_or_default();
    text(code_of(&locale))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locales_pick_the_right_table() {
        assert_eq!(code_of("fr_FR.UTF-8"), "fr");
        assert_eq!(code_of("en_GB"), "en");
        assert_eq!(text("fr").placeholder, "Mot de passe");
        assert_eq!(text("de").placeholder, "Password");
    }
}
