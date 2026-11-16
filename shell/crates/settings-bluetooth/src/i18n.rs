//! The page's strings in English and French. Other languages read English.
//! `{name}`, `{service}` and `{n}` are filled in by the view.

use crate::model::{Failure, Kind, Service, Signal};

pub struct Text {
    pub title: &'static str,
    pub description: &'static str,
    pub visible_as: &'static str,
    pub my_devices: &'static str,
    pub nearby: &'static str,
    pub searching: &'static str,
    pub no_paired: &'static str,
    pub no_nearby: &'static str,
    pub show_unnamed: &'static str,
    pub unnamed_count: &'static str,
    pub connected: &'static str,
    pub not_connected: &'static str,
    pub connecting: &'static str,
    pub disconnecting: &'static str,
    pub forget: &'static str,
    pub details: &'static str,
    pub address: &'static str,
    pub kind: &'static str,
    pub trusted: &'static str,
    pub trusted_hint: &'static str,
    pub battery: &'static str,
    pub pair: &'static str,
    pub pairing: &'static str,
    pub confirm_title: &'static str,
    pub confirm_body: &'static str,
    pub confirm_button: &'static str,
    pub display_title: &'static str,
    pub display_body: &'static str,
    pub display_pin_body: &'static str,
    pub request_pin_title: &'static str,
    pub request_pin_body: &'static str,
    pub request_passkey_title: &'static str,
    pub request_passkey_body: &'static str,
    pub authorize_title: &'static str,
    pub authorize_body: &'static str,
    pub service_title: &'static str,
    pub service_body: &'static str,
    pub allow: &'static str,
    pub deny: &'static str,
    pub cancel: &'static str,
    pub close: &'static str,
    pub try_again: &'static str,
    pub failed_title: &'static str,
    pub connect_failed: &'static str,
    pub no_service_title: &'static str,
    pub no_service_body: &'static str,
    pub no_adapter_title: &'static str,
    pub no_adapter_body: &'static str,
    pub off_title: &'static str,
    pub off_body: &'static str,
    pub soft_title: &'static str,
    pub soft_body: &'static str,
    pub hard_title: &'static str,
    pub hard_body: &'static str,
    pub turn_on: &'static str,
    pub settings_link: &'static str,
    pub kinds: [&'static str; 8],
    pub signals: [&'static str; 3],
    pub services: [&'static str; 7],
    pub failures: [&'static str; 8],
}

pub const EN: Text = Text {
    title: "Bluetooth",
    description: "Headphones, keyboards, mice and other wireless devices.",
    visible_as: "Visible to other devices as \u{201c}{name}\u{201d}",
    my_devices: "My devices",
    nearby: "Nearby devices",
    searching: "Searching\u{2026}",
    no_paired: "No paired devices yet.",
    no_nearby: "Put a device in pairing mode to see it here.",
    show_unnamed: "Show unnamed devices",
    unnamed_count: "{n} without a name",
    connected: "Connected",
    not_connected: "Not connected",
    connecting: "Connecting\u{2026}",
    disconnecting: "Disconnecting\u{2026}",
    forget: "Forget",
    details: "Details",
    address: "Address",
    kind: "Type",
    trusted: "Trusted",
    trusted_hint: "Connects without asking",
    battery: "{n}%",
    pair: "Pair",
    pairing: "Pairing\u{2026}",
    confirm_title: "Pair with {name}?",
    confirm_body: "Check that {name} shows the same code.",
    confirm_button: "It matches",
    display_title: "Type this code on {name}",
    display_body: "Then press Enter on the keyboard.",
    display_pin_body: "Type it on {name} if it asks for a PIN.",
    request_pin_title: "Enter the PIN for {name}",
    request_pin_body: "It is in the device's manual. Many devices use 0000 or 1234.",
    request_passkey_title: "Enter the code shown on {name}",
    request_passkey_body: "Six digits.",
    authorize_title: "Pair with {name}?",
    authorize_body: "{name} wants to pair with this computer.",
    service_title: "Allow {name} to connect?",
    service_body: "{name} asks to use {service}.",
    allow: "Allow",
    deny: "Deny",
    cancel: "Cancel",
    close: "Close",
    try_again: "Try again",
    failed_title: "Couldn't pair with {name}",
    connect_failed: "Couldn't connect",
    no_service_title: "The Bluetooth service is not running",
    no_service_body: "Flux cannot reach BlueZ. Restarting the computer usually fixes this.",
    no_adapter_title: "No Bluetooth adapter",
    no_adapter_body: "This computer has no Bluetooth, or it is disabled in the firmware settings. A USB adapter works too.",
    off_title: "Bluetooth is off",
    off_body: "Turn it on to use headphones, keyboards and other wireless devices.",
    soft_title: "Bluetooth is blocked",
    soft_body: "Airplane mode or another app turned the radio off. Turning Bluetooth on unblocks it.",
    hard_title: "Bluetooth is switched off on the hardware",
    hard_body: "Use the wireless switch or key on this computer to turn it back on.",
    turn_on: "Turn on",
    settings_link: "Bluetooth settings",
    kinds: [
        "Headphones",
        "Speaker",
        "Keyboard",
        "Mouse",
        "Phone",
        "Game controller",
        "Computer",
        "Device",
    ],
    signals: ["Strong signal", "Good signal", "Weak signal"],
    services: [
        "audio",
        "calls",
        "input",
        "media controls",
        "file transfer",
        "network sharing",
        "a service",
    ],
    failures: [
        "The device declined.",
        "Pairing was canceled.",
        "The device did not answer in time. Keep it close and in pairing mode.",
        "The codes did not match.",
        "The device is out of range or not in pairing mode.",
        "Bluetooth is blocked.",
        "Another pairing is in progress. Try again in a moment.",
        "Something went wrong. Try again.",
    ],
};

pub const FR: Text = Text {
    title: "Bluetooth",
    description: "Casques, claviers, souris et autres appareils sans fil.",
    visible_as: "Visible des autres appareils sous le nom \u{ab}\u{a0}{name}\u{a0}\u{bb}",
    my_devices: "Mes appareils",
    nearby: "Appareils \u{e0} proximit\u{e9}",
    searching: "Recherche\u{2026}",
    no_paired: "Aucun appareil associ\u{e9} pour l\u{2019}instant.",
    no_nearby: "Mettez un appareil en mode association pour le voir ici.",
    show_unnamed: "Afficher les appareils sans nom",
    unnamed_count: "{n} sans nom",
    connected: "Connect\u{e9}",
    not_connected: "Non connect\u{e9}",
    connecting: "Connexion\u{2026}",
    disconnecting: "D\u{e9}connexion\u{2026}",
    forget: "Oublier",
    details: "D\u{e9}tails",
    address: "Adresse",
    kind: "Type",
    trusted: "Approuv\u{e9}",
    trusted_hint: "Se connecte sans demander",
    battery: "{n}\u{202f}%",
    pair: "Associer",
    pairing: "Association\u{2026}",
    confirm_title: "Associer {name}\u{a0}?",
    confirm_body: "V\u{e9}rifiez que {name} affiche le m\u{ea}me code.",
    confirm_button: "Le code correspond",
    display_title: "Saisissez ce code sur {name}",
    display_body: "Puis appuyez sur Entr\u{e9}e sur le clavier.",
    display_pin_body: "Saisissez-le sur {name} s\u{2019}il demande un code PIN.",
    request_pin_title: "Saisissez le code PIN de {name}",
    request_pin_body: "Il figure dans la notice de l\u{2019}appareil. Beaucoup utilisent 0000 ou 1234.",
    request_passkey_title: "Saisissez le code affich\u{e9} sur {name}",
    request_passkey_body: "Six chiffres.",
    authorize_title: "Associer {name}\u{a0}?",
    authorize_body: "{name} veut s\u{2019}associer \u{e0} cet ordinateur.",
    service_title: "Autoriser {name} \u{e0} se connecter\u{a0}?",
    service_body: "{name} demande \u{e0} utiliser {service}.",
    allow: "Autoriser",
    deny: "Refuser",
    cancel: "Annuler",
    close: "Fermer",
    try_again: "R\u{e9}essayer",
    failed_title: "Impossible d\u{2019}associer {name}",
    connect_failed: "Connexion impossible",
    no_service_title: "Le service Bluetooth ne fonctionne pas",
    no_service_body: "Flux ne parvient pas \u{e0} joindre BlueZ. Red\u{e9}marrer l\u{2019}ordinateur r\u{e8}gle g\u{e9}n\u{e9}ralement le probl\u{e8}me.",
    no_adapter_title: "Aucun adaptateur Bluetooth",
    no_adapter_body: "Cet ordinateur n\u{2019}a pas de Bluetooth, ou il est d\u{e9}sactiv\u{e9} dans le micrologiciel. Un adaptateur USB fonctionne aussi.",
    off_title: "Le Bluetooth est d\u{e9}sactiv\u{e9}",
    off_body: "Activez-le pour utiliser un casque, un clavier ou d\u{2019}autres appareils sans fil.",
    soft_title: "Le Bluetooth est bloqu\u{e9}",
    soft_body: "Le mode avion ou une autre application a coup\u{e9} la radio. Activer le Bluetooth la d\u{e9}bloque.",
    hard_title: "Le Bluetooth est coup\u{e9} mat\u{e9}riellement",
    hard_body: "Utilisez l\u{2019}interrupteur ou la touche sans fil de cet ordinateur pour le r\u{e9}activer.",
    turn_on: "Activer",
    settings_link: "Param\u{e8}tres Bluetooth",
    kinds: [
        "Casque",
        "Enceinte",
        "Clavier",
        "Souris",
        "T\u{e9}l\u{e9}phone",
        "Manette",
        "Ordinateur",
        "Appareil",
    ],
    signals: ["Signal fort", "Bon signal", "Signal faible"],
    services: [
        "l\u{2019}audio",
        "les appels",
        "la saisie",
        "les commandes multim\u{e9}dia",
        "le transfert de fichiers",
        "le partage r\u{e9}seau",
        "un service",
    ],
    failures: [
        "L\u{2019}appareil a refus\u{e9}.",
        "L\u{2019}association a \u{e9}t\u{e9} annul\u{e9}e.",
        "L\u{2019}appareil n\u{2019}a pas r\u{e9}pondu \u{e0} temps. Gardez-le pr\u{e8}s de vous et en mode association.",
        "Les codes ne correspondent pas.",
        "L\u{2019}appareil est hors de port\u{e9}e ou n\u{2019}est pas en mode association.",
        "Le Bluetooth est bloqu\u{e9}.",
        "Une autre association est en cours. R\u{e9}essayez dans un instant.",
        "Un probl\u{e8}me est survenu. R\u{e9}essayez.",
    ],
};

/// The language code of the session: the first of `LC_ALL`, `LC_MESSAGES`,
/// `LANGUAGE` and `LANG` that is set.
fn session_language() -> String {
    ["LC_ALL", "LC_MESSAGES", "LANGUAGE", "LANG"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .find(|value| !value.is_empty())
        .unwrap_or_default()
}

/// The strings for a language code such as `fr_FR.UTF-8`.
pub fn for_language(code: &str) -> &'static Text {
    if code.starts_with("fr") {
        &FR
    } else {
        &EN
    }
}

/// The strings for the session's language.
pub fn text() -> &'static Text {
    for_language(&session_language())
}

impl Text {
    pub fn kind(&self, kind: Kind) -> &'static str {
        self.kinds[kind as usize]
    }

    pub fn signal(&self, signal: Signal) -> &'static str {
        self.signals[signal as usize]
    }

    pub fn service(&self, service: Service) -> &'static str {
        self.services[service as usize]
    }

    pub fn failure(&self, failure: Failure) -> &'static str {
        self.failures[failure as usize]
    }
}

/// `pattern` with `{name}` replaced.
pub fn named(pattern: &str, name: &str) -> String {
    pattern.replace("{name}", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn french_is_picked_by_prefix() {
        assert_eq!(for_language("fr_FR.UTF-8").title, FR.title);
        assert_eq!(for_language("fr").my_devices, "Mes appareils");
        assert_eq!(for_language("de_DE.UTF-8").my_devices, "My devices");
        assert_eq!(for_language("").my_devices, "My devices");
    }

    #[test]
    fn enum_tables_line_up() {
        assert_eq!(EN.kind(Kind::Controller), "Game controller");
        assert_eq!(FR.kind(Kind::Other), "Appareil");
        assert_eq!(EN.signal(Signal::Weak), "Weak signal");
        assert_eq!(FR.service(Service::Other), "un service");
        assert_eq!(EN.failure(Failure::WrongCode), "The codes did not match.");
        assert_eq!(FR.failure(Failure::Other), "Un probl\u{e8}me est survenu. R\u{e9}essayez.");
    }

    #[test]
    fn every_pattern_keeps_its_placeholder() {
        for text in [&EN, &FR] {
            assert!(text.battery.contains("{n}"));
            assert!(text.unnamed_count.contains("{n}"));
            assert!(text.service_body.contains("{service}"));
            for pattern in [text.visible_as, text.confirm_title, text.failed_title, text.display_title] {
                assert!(pattern.contains("{name}"));
            }
        }
    }
}
