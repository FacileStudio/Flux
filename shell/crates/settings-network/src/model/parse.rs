//! NetworkManager property maps into the page's structs.

use std::net::Ipv4Addr;

use super::value::{
    bool_of, bytes_of, i64_of, maps_of, path_of, str_of, u32_of, u32s_of, u8_of, Props,
};
use super::{wifi, AccessPoint, IpInfo, Kind, Link, Saved, Settings, WifiDevice, Wired};

/// `NM_DEVICE_TYPE_ETHERNET`.
pub const DEVICE_ETHERNET: u32 = 1;
/// `NM_DEVICE_TYPE_WIFI`.
pub const DEVICE_WIFI: u32 = 2;

/// The link a NetworkManager `NMDeviceState` stands for.
pub fn link(state: u32) -> Link {
    match state {
        100 => Link::Connected,
        40..=90 => Link::Connecting,
        120 => Link::Failed,
        30 | 110 => Link::Disconnected,
        _ => Link::Unavailable,
    }
}

/// The link an `NMActiveConnectionState` stands for.
pub fn active_link(state: u32) -> Link {
    match state {
        1 => Link::Connecting,
        2 => Link::Connected,
        3 | 4 => Link::Disconnected,
        _ => Link::Unavailable,
    }
}

/// An SSID as text. Invalid UTF-8 keeps its readable part.
pub fn ssid(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .trim_end_matches('\0')
        .to_string()
}

/// One `org.freedesktop.NetworkManager.AccessPoint`.
pub fn access_point(path: &str, props: &Props) -> Option<AccessPoint> {
    let ssid = ssid(&bytes_of(props, "Ssid")?);
    Some(AccessPoint {
        path: path.to_string(),
        ssid,
        strength: u8_of(props, "Strength").unwrap_or(0),
        security: wifi::security(
            u32_of(props, "Flags").unwrap_or(0),
            u32_of(props, "WpaFlags").unwrap_or(0),
            u32_of(props, "RsnFlags").unwrap_or(0),
        ),
        frequency: u32_of(props, "Frequency").unwrap_or(0),
    })
}

/// An `org.freedesktop.NetworkManager.IP4Config`, newest properties first.
pub fn ip4(props: &Props) -> IpInfo {
    let address = maps_of(props, "AddressData").into_iter().find_map(|entry| {
        let address = str_of(&entry, "address")?;
        Some(match u32_of(&entry, "prefix") {
            Some(prefix) => format!("{address}/{prefix}"),
            None => address,
        })
    });
    let mut dns: Vec<String> = maps_of(props, "NameserverData")
        .iter()
        .filter_map(|entry| str_of(entry, "address"))
        .collect();
    if dns.is_empty() {
        dns = u32s_of(props, "Nameservers")
            .into_iter()
            .map(|raw| Ipv4Addr::from(raw.to_ne_bytes()).to_string())
            .collect();
    }
    IpInfo {
        address,
        gateway: str_of(props, "Gateway").filter(|gateway| !gateway.is_empty()),
        dns,
    }
}

/// The parts every device shares: path, interface, MAC and link.
fn device_base(path: &str, device: &Props) -> (String, String, String, Link) {
    (
        path.to_string(),
        str_of(device, "Interface").unwrap_or_default(),
        str_of(device, "HwAddress").unwrap_or_default(),
        link(u32_of(device, "State").unwrap_or(0)),
    )
}

/// A Wi-Fi device from its `Device` and `Device.Wireless` properties.
pub fn wifi_device(path: &str, device: &Props, wireless: &Props, ip: IpInfo) -> WifiDevice {
    let (path, iface, mac, link) = device_base(path, device);
    let mac = str_of(wireless, "HwAddress").unwrap_or(mac);
    WifiDevice {
        path,
        iface,
        mac,
        link,
        active_ap: path_of(wireless, "ActiveAccessPoint"),
        ip,
        last_scan: i64_of(wireless, "LastScan").unwrap_or(0),
    }
}

/// An Ethernet device from its `Device` and `Device.Wired` properties.
pub fn wired(path: &str, device: &Props, wired: &Props, ip: IpInfo) -> Wired {
    let (path, iface, mac, link) = device_base(path, device);
    Wired {
        path,
        iface,
        mac: str_of(wired, "HwAddress").unwrap_or(mac),
        link,
        speed: u32_of(wired, "Speed").unwrap_or(0),
        carrier: bool_of(wired, "Carrier").unwrap_or(false),
        ip,
    }
}

/// A saved profile from `Settings.Connection.GetSettings`.
pub fn saved(path: &str, settings: &Settings) -> Saved {
    let empty = Props::new();
    let connection = settings.get("connection").unwrap_or(&empty);
    let kind = match str_of(connection, "type").as_deref() {
        Some("802-11-wireless") => Kind::Wifi {
            ssid: settings
                .get("802-11-wireless")
                .and_then(|wireless| bytes_of(wireless, "ssid"))
                .map(|bytes| ssid(&bytes))
                .unwrap_or_default(),
        },
        Some("vpn" | "wireguard") => Kind::Vpn,
        _ => Kind::Other,
    };
    Saved {
        path: path.to_string(),
        id: str_of(connection, "id").unwrap_or_default(),
        uuid: str_of(connection, "uuid").unwrap_or_default(),
        kind,
        autoconnect: bool_of(connection, "autoconnect").unwrap_or(true),
    }
}
