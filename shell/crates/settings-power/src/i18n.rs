//! The page's strings in English and French. Other languages read English.

use std::cell::Cell;

pub struct Text {
    pub title: &'static str,
    pub description: &'static str,
    pub battery: &'static str,
    pub no_battery: &'static str,
    pub no_battery_body: &'static str,
    pub charging: &'static str,
    pub discharging: &'static str,
    pub full: &'static str,
    pub not_charging: &'static str,
    pub until_full: &'static str,
    pub left: &'static str,
    pub health: &'static str,
    pub cycles: &'static str,
    pub health_unknown: &'static str,
    pub show_percentage: &'static str,
    pub show_percentage_body: &'static str,
    pub mode: &'static str,
    pub mode_body: &'static str,
    pub mode_missing: &'static str,
    pub power_saver: &'static str,
    pub balanced: &'static str,
    pub performance: &'static str,
    pub idle: &'static str,
    pub idle_body: &'static str,
    pub screen_off: &'static str,
    pub lock: &'static str,
    pub lock_body: &'static str,
    pub suspend_battery: &'static str,
    pub suspend_ac: &'static str,
    pub lid: &'static str,
    pub lid_suspend: &'static str,
    pub lid_nothing: &'static str,
    pub never: &'static str,
    pub save_failed: &'static str,
    pub hours: &'static str,
    pub minutes: &'static str,
}

pub const EN: Text = Text {
    title: "Power",
    description: "Battery, power mode, and when the screen sleeps.",
    battery: "Battery",
    no_battery: "No battery",
    no_battery_body: "This computer runs on mains power.",
    charging: "Charging",
    discharging: "On battery",
    full: "Fully charged",
    not_charging: "Plugged in, not charging",
    until_full: "until full",
    left: "left",
    health: "Battery health",
    cycles: "charge cycles",
    health_unknown: "This battery does not report its health.",
    show_percentage: "Show the percentage in the bar",
    show_percentage_body: "Next to the battery icon at the top of the screen.",
    mode: "Power mode",
    mode_body: "Balanced suits most uses. Power saver makes the battery last longer.",
    mode_missing: "Power modes are not available on this computer.",
    power_saver: "Power saver",
    balanced: "Balanced",
    performance: "Performance",
    idle: "Screen and sleep",
    idle_body: "What happens when you leave the computer alone.",
    screen_off: "Turn the screen off after",
    lock: "Lock after",
    lock_body: "Flux always locks before the computer sleeps.",
    suspend_battery: "Sleep on battery after",
    suspend_ac: "Sleep when plugged in after",
    lid: "When the lid closes",
    lid_suspend: "Sleep",
    lid_nothing: "Do nothing",
    never: "Never",
    save_failed: "Flux could not save this setting.",
    hours: "h",
    minutes: "min",
};

pub const FR: Text = Text {
    title: "Alimentation",
    description: "Batterie, mode d'alimentation et mise en veille de l'écran.",
    battery: "Batterie",
    no_battery: "Aucune batterie",
    no_battery_body: "Cet ordinateur fonctionne sur secteur.",
    charging: "En charge",
    discharging: "Sur batterie",
    full: "Chargée",
    not_charging: "Branché, pas en charge",
    until_full: "avant la charge complète",
    left: "restantes",
    health: "État de la batterie",
    cycles: "cycles de charge",
    health_unknown: "Cette batterie n'indique pas son état.",
    show_percentage: "Afficher le pourcentage dans la barre",
    show_percentage_body: "À côté de l'icône de batterie, en haut de l'écran.",
    mode: "Mode d'alimentation",
    mode_body: "Équilibré convient à la plupart des usages. Économie d'énergie prolonge la batterie.",
    mode_missing: "Les modes d'alimentation ne sont pas disponibles sur cet ordinateur.",
    power_saver: "Économie",
    balanced: "Équilibré",
    performance: "Performance",
    idle: "Écran et veille",
    idle_body: "Ce qui se passe quand vous laissez l'ordinateur.",
    screen_off: "Éteindre l'écran après",
    lock: "Verrouiller après",
    lock_body: "Flux verrouille toujours avant la mise en veille.",
    suspend_battery: "Veille sur batterie après",
    suspend_ac: "Veille sur secteur après",
    lid: "À la fermeture du capot",
    lid_suspend: "Mettre en veille",
    lid_nothing: "Ne rien faire",
    never: "Jamais",
    save_failed: "Flux n'a pas pu enregistrer ce réglage.",
    hours: "h",
    minutes: "min",
};

thread_local! {
    static FORCED: Cell<Option<&'static Text>> = const { Cell::new(None) };
}

/// The strings for a language code such as `fr`.
pub fn text_for(code: &str) -> &'static Text {
    match code {
        "fr" => &FR,
        _ => &EN,
    }
}

/// Use `code`'s strings from now on, whatever the locale says. For previews.
pub fn force(code: &str) {
    FORCED.with(|slot| slot.set(Some(text_for(code))));
}

/// The strings for the session's locale.
pub fn text() -> &'static Text {
    if let Some(forced) = FORCED.with(Cell::get) {
        return forced;
    }
    let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()))
        .unwrap_or_default();
    text_for(locale.split(['_', '.', '@']).next().unwrap_or(""))
}
