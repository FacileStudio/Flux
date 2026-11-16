//! The exit menu's strings in English and French.

use shell_common::locale::{self, Lang};

use crate::actions::Action;

pub struct Text {
    pub title: &'static str,
    pub subtitle: &'static str,
    pub cancel: &'static str,
    pub lock: &'static str,
    pub suspend: &'static str,
    pub logout: &'static str,
    pub restart: &'static str,
    pub poweroff: &'static str,
}

impl Text {
    /// The label of one choice.
    pub fn label(&self, action: Action) -> &'static str {
        match action {
            Action::Lock => self.lock,
            Action::Suspend => self.suspend,
            Action::Logout => self.logout,
            Action::Restart => self.restart,
            Action::PowerOff => self.poweroff,
        }
    }
}

const EN: Text = Text {
    title: "Leave or lock",
    subtitle: "Choose with the arrow keys and Enter. Escape closes.",
    cancel: "Cancel",
    lock: "Lock",
    suspend: "Suspend",
    logout: "Log out",
    restart: "Restart",
    poweroff: "Power off",
};

const FR: Text = Text {
    title: "Quitter ou verrouiller",
    subtitle: "Choisissez avec les flèches et Entrée. Échap ferme.",
    cancel: "Annuler",
    lock: "Verrouiller",
    suspend: "Veille",
    logout: "Déconnexion",
    restart: "Redémarrer",
    poweroff: "Éteindre",
};

/// The strings for the current language.
pub fn get() -> &'static Text {
    match locale::current() {
        Lang::En => &EN,
        Lang::Fr => &FR,
    }
}
