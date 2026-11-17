//! The bar's strings in English and French.

use shell_common::locale::{self, Lang};

pub struct Text {
    pub clock_format: &'static str,
    pub clock_tooltip: &'static str,
    pub session_tooltip: &'static str,
    pub apps_tooltip: &'static str,
    pub close_tooltip: &'static str,
    pub layout_tooltip: &'static str,
    pub volume: &'static str,
    pub muted: &'static str,
    pub mute: &'static str,
    pub unmute: &'static str,
    pub brightness: &'static str,
    pub battery: &'static str,
    pub charging: &'static str,
    pub discharging: &'static str,
    pub full: &'static str,
    pub plugged: &'static str,
    pub unknown: &'static str,
    pub to_full: &'static str,
    pub to_empty: &'static str,
    pub battery_low: &'static str,
    pub battery_low_body: &'static str,
    pub network: &'static str,
    pub online: &'static str,
    pub connecting: &'static str,
    pub disconnecting: &'static str,
    pub offline: &'static str,
    pub limited: &'static str,
    pub interface: &'static str,
    pub unavailable: &'static str,
}

const EN: Text = Text {
    clock_format: "%a %-d %b  %H:%M",
    clock_tooltip: "Notifications and quick settings",
    session_tooltip: "Lock, log out or power off",
    apps_tooltip: "Open an app (Super+Space)",
    close_tooltip: "Close this window (Super+Q)",
    layout_tooltip: "Keyboard layout",
    volume: "Volume",
    muted: "Muted",
    mute: "Mute",
    unmute: "Unmute",
    brightness: "Brightness",
    battery: "Battery",
    charging: "Charging",
    discharging: "On battery",
    full: "Full",
    plugged: "Plugged in, not charging",
    unknown: "Unknown",
    to_full: "Full in",
    to_empty: "Empty in",
    battery_low: "Battery low",
    battery_low_body: "{n}% left. Plug in the charger.",
    network: "Network",
    online: "Online",
    connecting: "Connecting",
    disconnecting: "Disconnecting",
    offline: "Offline",
    limited: "No internet",
    interface: "Interface",
    unavailable: "Not available",
};

const FR: Text = Text {
    clock_format: "%a %-d %b  %H:%M",
    clock_tooltip: "Notifications et réglages rapides",
    session_tooltip: "Verrouiller, se déconnecter ou éteindre",
    apps_tooltip: "Ouvrir une application (Super+Espace)",
    close_tooltip: "Fermer cette fenêtre (Super+Q)",
    layout_tooltip: "Disposition du clavier",
    volume: "Volume",
    muted: "Muet",
    mute: "Couper le son",
    unmute: "Rétablir le son",
    brightness: "Luminosité",
    battery: "Batterie",
    charging: "En charge",
    discharging: "Sur batterie",
    full: "Chargée",
    plugged: "Branchée, sans charge",
    unknown: "Inconnu",
    to_full: "Chargée dans",
    to_empty: "Vide dans",
    battery_low: "Batterie faible",
    battery_low_body: "Il reste {n}\u{202f}%. Branchez le chargeur.",
    network: "Réseau",
    online: "Connecté",
    connecting: "Connexion",
    disconnecting: "Déconnexion",
    offline: "Hors ligne",
    limited: "Sans internet",
    interface: "Interface",
    unavailable: "Indisponible",
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
