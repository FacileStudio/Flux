//! What the page knows about the network, as plain data.
//!
//! Nothing here touches D-Bus or GTK. [`value`] reads NetworkManager property
//! maps, [`parse`] turns them into the structs below, and [`wifi`] holds the
//! rules the list follows: one row per SSID, signal to bars, security flags to
//! a kind, password checks and failure reasons.

pub mod parse;
pub mod value;
pub mod wifi;

pub use value::{Props, Settings};

/// How a Wi-Fi network is protected, from the access point flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Security {
    Open,
    Owe,
    Wep,
    Personal,
    Wpa3,
    Enterprise,
}

impl Security {
    /// Whether joining asks for a secret.
    pub fn secured(self) -> bool {
        !matches!(self, Security::Open | Security::Owe)
    }

    /// Whether this page can join it: open, enhanced open, WPA2 and WPA3 personal.
    pub fn supported(self) -> bool {
        !matches!(self, Security::Wep | Security::Enterprise)
    }

    /// The `802-11-wireless-security.key-mgmt` value NetworkManager expects.
    pub fn key_mgmt(self) -> Option<&'static str> {
        match self {
            Security::Personal => Some("wpa-psk"),
            Security::Wpa3 => Some("sae"),
            Security::Owe => Some("owe"),
            Security::Open | Security::Wep | Security::Enterprise => None,
        }
    }
}

/// Where a device stands, folded from the NetworkManager device state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    Unavailable,
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

/// One access point as NetworkManager reports it.
#[derive(Clone, Debug, PartialEq)]
pub struct AccessPoint {
    pub path: String,
    pub ssid: String,
    pub strength: u8,
    pub security: Security,
    pub frequency: u32,
}

/// One row of the Wi-Fi list: the strongest access point of an SSID.
#[derive(Clone, Debug, PartialEq)]
pub struct Network {
    pub ssid: String,
    pub ap_path: String,
    pub strength: u8,
    pub security: Security,
    pub frequency: u32,
    pub known: bool,
    pub active: bool,
}

/// Addressing of a connected device.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IpInfo {
    pub address: Option<String>,
    pub gateway: Option<String>,
    pub dns: Vec<String>,
}

/// The Wi-Fi adapter.
#[derive(Clone, Debug, PartialEq)]
pub struct WifiDevice {
    pub path: String,
    pub iface: String,
    pub mac: String,
    pub link: Link,
    pub active_ap: Option<String>,
    pub ip: IpInfo,
    pub last_scan: i64,
}

/// An Ethernet port.
#[derive(Clone, Debug, PartialEq)]
pub struct Wired {
    pub path: String,
    pub iface: String,
    pub mac: String,
    pub link: Link,
    pub speed: u32,
    pub carrier: bool,
    pub ip: IpInfo,
}

/// What a saved connection is for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Wifi { ssid: String },
    Vpn,
    Other,
}

/// A connection profile NetworkManager has on disk.
#[derive(Clone, Debug, PartialEq)]
pub struct Saved {
    pub path: String,
    pub id: String,
    pub uuid: String,
    pub kind: Kind,
    pub autoconnect: bool,
}

/// A VPN profile and whether it is up.
#[derive(Clone, Debug, PartialEq)]
pub struct Vpn {
    pub path: String,
    pub name: String,
    pub active: Option<String>,
    pub link: Link,
}

/// Whether NetworkManager could be read at all.
#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Loading,
    Ready,
    NotRunning,
    Error(String),
}

/// Everything the page draws, read in one pass.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub status: Status,
    pub wifi_enabled: bool,
    pub wifi_hardware: bool,
    pub wwan_enabled: bool,
    pub scanning: bool,
    pub wifi: Option<WifiDevice>,
    pub networks: Vec<Network>,
    pub wired: Vec<Wired>,
    pub vpns: Vec<Vpn>,
    pub saved: Vec<Saved>,
}

impl Snapshot {
    /// A snapshot that only says why there is nothing to show.
    pub fn empty(status: Status) -> Snapshot {
        Snapshot {
            status,
            wifi_enabled: false,
            wifi_hardware: false,
            wwan_enabled: false,
            scanning: false,
            wifi: None,
            networks: Vec::new(),
            wired: Vec::new(),
            vpns: Vec::new(),
            saved: Vec::new(),
        }
    }

    /// Airplane mode: every radio NetworkManager drives is off.
    pub fn airplane(&self) -> bool {
        !self.wifi_enabled && !self.wwan_enabled
    }

    /// The saved Wi-Fi profiles for `ssid`.
    pub fn saved_for<'a>(&'a self, ssid: &'a str) -> impl Iterator<Item = &'a Saved> + 'a {
        self.saved
            .iter()
            .filter(move |saved| matches!(&saved.kind, Kind::Wifi { ssid: own } if own == ssid))
    }
}

#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod tests;
