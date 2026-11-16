//! What the sound server reports, and every decision about it that needs no GTK and no server:
//! friendly device names, which ports and profiles to offer, volume math, app grouping.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

mod apps;
mod device;
mod volume;

pub use apps::{group_streams, AppGroup};
pub use device::clean_description;
pub use volume::{
    amplitude, max_percent, meter, percent, raw, smooth, step, test_tone, volume_icon,
};

/// The raw volume PulseAudio calls 100% (`PA_VOLUME_NORM`).
pub const NORM: u32 = 0x10000;
/// The highest slider position when "Allow volume above 100%" is on.
pub const BOOST_PERCENT: u32 = 150;
/// The `application.name` this crate gives its own streams, so they never list as an app.
pub const OWN_APP: &str = "Flux Settings";

/// Which way sound flows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Output,
    Input,
}

/// Whether a port has something plugged into it, as the server knows it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Availability {
    #[default]
    Unknown,
    No,
    Yes,
}

/// A connector on a sink or source: speakers, headphone jack, HDMI.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Port {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub available: Availability,
}

/// A sink (output) or source (input) as the server describes it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    pub index: u32,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
    #[serde(default)]
    pub ports: Vec<Port>,
    #[serde(default)]
    pub active_port: Option<String>,
    pub volume: u32,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub card: Option<u32>,
    #[serde(default)]
    pub monitor: Option<String>,
    #[serde(default)]
    pub is_monitor: bool,
}

/// One way a card can run: analog duplex, HDMI output, A2DP, headset.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub priority: u32,
    #[serde(default = "yes")]
    pub available: bool,
}

fn yes() -> bool {
    true
}

/// A sound card and its profiles.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub index: u32,
    pub name: String,
    #[serde(default)]
    pub profiles: Vec<Profile>,
    #[serde(default)]
    pub active_profile: Option<String>,
}

/// A playing stream (a sink input).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stream {
    pub index: u32,
    pub sink: u32,
    pub volume: u32,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

/// Everything the page shows, as one value the backend sends after each change.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    #[serde(default)]
    pub default_sink: Option<String>,
    #[serde(default)]
    pub default_source: Option<String>,
    #[serde(default)]
    pub sinks: Vec<Device>,
    #[serde(default)]
    pub sources: Vec<Device>,
    #[serde(default)]
    pub streams: Vec<Stream>,
    #[serde(default)]
    pub cards: Vec<Card>,
}

/// What kind of thing a device is, which picks its icon and its label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Speakers,
    Headphones,
    Headset,
    Hdmi,
    Bluetooth,
    Usb,
    Microphone,
    Virtual,
    Other,
}

impl Kind {
    /// A symbolic icon name every GTK icon theme ships.
    pub fn icon(self) -> &'static str {
        match self {
            Kind::Speakers => "audio-speakers-symbolic",
            Kind::Headphones => "audio-headphones-symbolic",
            Kind::Headset => "audio-headset-symbolic",
            Kind::Hdmi => "video-display-symbolic",
            Kind::Bluetooth => "bluetooth-active-symbolic",
            Kind::Usb => "audio-card-symbolic",
            Kind::Microphone => "audio-input-microphone-symbolic",
            Kind::Virtual => "applications-engineering-symbolic",
            Kind::Other => "audio-card-symbolic",
        }
    }
}

/// How a device reads in a list: a short title and, when the title came from the port, the
/// card it belongs to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Label {
    pub title: String,
    pub detail: Option<String>,
    pub kind: Kind,
}

impl Snapshot {
    /// The devices of `direction` a person can pick: never a sink's monitor.
    pub fn devices(&self, direction: Direction) -> Vec<&Device> {
        let list = match direction {
            Direction::Output => &self.sinks,
            Direction::Input => &self.sources,
        };
        list.iter().filter(|device| !device.is_monitor).collect()
    }

    /// The current default device of `direction`, or the first one when the default is gone.
    pub fn default_device(&self, direction: Direction) -> Option<&Device> {
        let name = match direction {
            Direction::Output => self.default_sink.as_deref(),
            Direction::Input => self.default_source.as_deref(),
        };
        let devices = self.devices(direction);
        devices
            .iter()
            .find(|device| Some(device.name.as_str()) == name)
            .or_else(|| devices.first())
            .copied()
    }

    /// The card a device sits on.
    pub fn card_of(&self, device: &Device) -> Option<&Card> {
        let index = device.card?;
        self.cards.iter().find(|card| card.index == index)
    }
}

/// Settings this page keeps itself, in `~/.config/flux/audio.json`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Options {
    #[serde(default)]
    pub allow_boost: bool,
}

impl Options {
    /// Read the file's text; anything unreadable is the defaults.
    pub fn parse(text: &str) -> Options {
        serde_json::from_str(text).unwrap_or_default()
    }

    /// The file's text.
    pub fn to_json(self) -> String {
        serde_json::to_string_pretty(&self).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests;
