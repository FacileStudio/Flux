//! The Sound page's strings in English and French. Other languages read English.

use crate::model::Kind;

pub struct Text {
    pub title: &'static str,
    pub output: &'static str,
    pub input: &'static str,
    pub applications: &'static str,
    pub options: &'static str,
    pub volume: &'static str,
    pub mute: &'static str,
    pub unmute: &'static str,
    pub port: &'static str,
    pub profile: &'static str,
    pub test_title: &'static str,
    pub test_body: &'static str,
    pub test: &'static str,
    pub boost_title: &'static str,
    pub boost_body: &'static str,
    pub no_output: &'static str,
    pub no_input: &'static str,
    pub connecting: &'static str,
    pub unavailable_title: &'static str,
    pub unavailable_body: &'static str,
    pub microphone: &'static str,
    pub settings: &'static str,
    pub kind_speakers: &'static str,
    pub kind_headphones: &'static str,
    pub kind_headset: &'static str,
    pub kind_hdmi: &'static str,
    pub kind_bluetooth: &'static str,
    pub kind_usb: &'static str,
    pub kind_microphone: &'static str,
    pub kind_virtual: &'static str,
    pub kind_other: &'static str,
    pub percent_suffix: &'static str,
}

pub const EN: Text = Text {
    title: "Sound",
    output: "Output",
    input: "Input",
    applications: "Applications",
    options: "Options",
    volume: "Volume",
    mute: "Mute",
    unmute: "Unmute",
    port: "Connector",
    profile: "Mode",
    test_title: "Test the output",
    test_body: "Plays a short sound on the left, then on the right.",
    test: "Test",
    boost_title: "Allow volume above 100%",
    boost_body: "Louder, but the sound can distort.",
    no_output: "No output device found.",
    no_input: "No microphone found.",
    connecting: "Connecting to the sound server",
    unavailable_title: "Sound is not available",
    unavailable_body: "Flux cannot reach the sound server. It keeps trying on its own.",
    microphone: "Microphone",
    settings: "Sound settings",
    kind_speakers: "Speakers",
    kind_headphones: "Headphones",
    kind_headset: "Headset",
    kind_hdmi: "HDMI",
    kind_bluetooth: "Bluetooth",
    kind_usb: "USB",
    kind_microphone: "Microphone",
    kind_virtual: "Virtual device",
    kind_other: "Sound card",
    percent_suffix: "%",
};

pub const FR: Text = Text {
    title: "Son",
    output: "Sortie",
    input: "Entrée",
    applications: "Applications",
    options: "Options",
    volume: "Volume",
    mute: "Couper le son",
    unmute: "Remettre le son",
    port: "Connecteur",
    profile: "Mode",
    test_title: "Tester la sortie",
    test_body: "Joue un son court à gauche, puis à droite.",
    test: "Tester",
    boost_title: "Autoriser un volume au-delà de 100\u{202f}%",
    boost_body: "Plus fort, mais le son peut saturer.",
    no_output: "Aucune sortie audio trouvée.",
    no_input: "Aucun micro trouvé.",
    connecting: "Connexion au serveur de son",
    unavailable_title: "Le son n'est pas disponible",
    unavailable_body: "Flux n'arrive pas à joindre le serveur de son. Il réessaie tout seul.",
    microphone: "Micro",
    settings: "Réglages du son",
    kind_speakers: "Haut-parleurs",
    kind_headphones: "Casque",
    kind_headset: "Casque-micro",
    kind_hdmi: "HDMI",
    kind_bluetooth: "Bluetooth",
    kind_usb: "USB",
    kind_microphone: "Micro",
    kind_virtual: "Périphérique virtuel",
    kind_other: "Carte son",
    percent_suffix: "\u{202f}%",
};

/// The table for a locale string such as `fr_FR.UTF-8`.
pub fn for_locale(locale: &str) -> &'static Text {
    if locale.to_ascii_lowercase().starts_with("fr") {
        &FR
    } else {
        &EN
    }
}

/// The table for this session, from `LC_ALL`, `LC_MESSAGES` then `LANG`.
pub fn text() -> &'static Text {
    let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|key| std::env::var(key).ok())
        .find(|value| !value.is_empty())
        .unwrap_or_default();
    for_locale(&locale)
}

impl Text {
    /// The label of a device kind.
    pub fn kind(&self, kind: Kind) -> &'static str {
        match kind {
            Kind::Speakers => self.kind_speakers,
            Kind::Headphones => self.kind_headphones,
            Kind::Headset => self.kind_headset,
            Kind::Hdmi => self.kind_hdmi,
            Kind::Bluetooth => self.kind_bluetooth,
            Kind::Usb => self.kind_usb,
            Kind::Microphone => self.kind_microphone,
            Kind::Virtual => self.kind_virtual,
            Kind::Other => self.kind_other,
        }
    }

    /// A percentage, with the spacing the language wants.
    pub fn percent(&self, value: u32) -> String {
        format!("{value}{}", self.percent_suffix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_picks_the_table() {
        assert_eq!(for_locale("fr_FR.UTF-8").title, "Son");
        assert_eq!(for_locale("en_GB.UTF-8").title, "Sound");
        assert_eq!(for_locale("").title, "Sound");
        assert_eq!(for_locale("de_DE").title, "Sound");
    }

    #[test]
    fn french_percent_uses_a_narrow_space() {
        assert_eq!(FR.percent(80), "80\u{202f}%");
        assert_eq!(EN.percent(80), "80%");
    }
}
