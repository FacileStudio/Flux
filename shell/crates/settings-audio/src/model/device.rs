//! Turning what the server calls a device into what a person calls it.

use super::{percent, Availability, Card, Device, Direction, Kind, Label, Port, Profile};

impl Device {
    fn property(&self, key: &str) -> Option<&str> {
        self.properties
            .get(key)
            .map(String::as_str)
            .filter(|value| !value.is_empty())
    }

    fn active(&self) -> Option<&Port> {
        let name = self.active_port.as_deref()?;
        self.ports.iter().find(|port| port.name == name)
    }

    /// What kind of device this is, from its name, bus, form factor and active port.
    pub fn kind(&self, direction: Direction) -> Kind {
        let name = self.name.to_ascii_lowercase();
        let bus = self.property("device.bus").unwrap_or_default();
        if name.starts_with("bluez_") || bus == "bluetooth" {
            return Kind::Bluetooth;
        }
        if self.is_virtual() {
            return Kind::Virtual;
        }
        let port = self
            .active()
            .map(|port| port.name.to_ascii_lowercase())
            .unwrap_or_default();
        let form = self.property("device.form_factor").unwrap_or_default();
        let hdmi =
            direction == Direction::Output && (port.contains("hdmi") || name.contains("hdmi"));
        let usb = bus == "usb" || name.contains(".usb-");
        let speaker = port.contains("speaker") || form == "speaker" || form == "internal";
        [
            (hdmi, Kind::Hdmi),
            (port.contains("headset") || form == "headset", Kind::Headset),
            (
                port.contains("headphone") || form == "headphone",
                Kind::Headphones,
            ),
            (usb, Kind::Usb),
            (direction == Direction::Input, Kind::Microphone),
            (speaker, Kind::Speakers),
        ]
        .into_iter()
        .find_map(|(matches, kind)| matches.then_some(kind))
        .unwrap_or(Kind::Other)
    }

    fn is_virtual(&self) -> bool {
        self.property("device.class") == Some("abstract")
            || self.property("factory.name") == Some("support.null-audio-sink")
            || self.name.starts_with("null")
            || self.name.starts_with("auto_null")
    }

    /// A name a person recognises: "Speakers" on the built-in card, "WH-1000XM4" for a headset,
    /// "Scarlett Solo USB" for an interface, never "alsa_output.pci-0000_00_1f.3.analog-stereo".
    pub fn label(&self, direction: Direction) -> Label {
        let kind = self.kind(direction);
        let card = clean_description(&self.description);
        let card = (!card.is_empty()).then_some(card);
        let from_port = matches!(
            kind,
            Kind::Speakers | Kind::Headphones | Kind::Headset | Kind::Hdmi | Kind::Microphone
        ) && !matches!(self.property("device.bus"), Some("usb"));
        let port = self
            .active()
            .map(|port| port.description.trim().to_string())
            .filter(|text| !text.is_empty() && !is_generic_port(text));
        let fallback = || card.clone().unwrap_or_else(|| self.name.clone());
        match (from_port, port) {
            (true, Some(port)) => Label {
                title: port,
                detail: card,
                kind,
            },
            _ => Label {
                title: fallback(),
                detail: None,
                kind,
            },
        }
    }

    /// The ports worth offering: those not known to be unplugged, plus the active one.
    pub fn port_choices(&self) -> Vec<&Port> {
        self.ports
            .iter()
            .filter(|port| {
                port.available != Availability::No
                    || self.active_port.as_deref() == Some(&port.name)
            })
            .collect()
    }

    /// The volume as a whole percentage.
    pub fn percent(&self) -> u32 {
        percent(self.volume)
    }
}

fn is_generic_port(description: &str) -> bool {
    [
        "Analog Output",
        "Analog Input",
        "Digital Output",
        "Line Out",
        "Analogique",
    ]
    .iter()
    .any(|generic| description.starts_with(generic))
}

const NOISE: [&str; 12] = [
    " Analog Stereo",
    " Analog Mono",
    " Analog Surround 2.1",
    " Analog Surround 4.0",
    " Analog Surround 5.1",
    " Analog Surround 7.1",
    " Digital Stereo (HDMI)",
    " Digital Stereo (IEC958)",
    " Digital Surround 5.1 (HDMI)",
    " Multichannel",
    " Stereo",
    " Mono",
];

/// Drop the channel layout ALSA appends to a card name: "Built-in Audio Analog Stereo" reads
/// "Built-in Audio".
pub fn clean_description(description: &str) -> String {
    let mut text = description.trim();
    if let Some(stripped) = text.strip_prefix("Monitor of ") {
        text = stripped;
    }
    let mut text = text.to_string();
    for noise in NOISE {
        if let Some(stripped) = text.strip_suffix(noise) {
            text = stripped.trim_end().to_string();
            break;
        }
    }
    text
}

impl Card {
    /// The profiles worth offering, best first: available ones, without "off" unless active.
    pub fn profile_choices(&self) -> Vec<&Profile> {
        let active = self.active_profile.as_deref();
        let mut list: Vec<&Profile> = self
            .profiles
            .iter()
            .filter(|profile| {
                Some(profile.name.as_str()) == active || profile.available && profile.name != "off"
            })
            .collect();
        list.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.description.cmp(&b.description))
        });
        list
    }
}
