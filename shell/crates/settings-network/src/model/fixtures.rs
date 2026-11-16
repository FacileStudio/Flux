//! Property maps shaped like the ones NetworkManager 1.48 answers `GetAll`
//! and `GetSettings` with, for the model tests.

use std::collections::HashMap;

use zbus::zvariant::{ObjectPath, OwnedValue, Value};

use super::{Props, Settings};

fn owned(value: Value<'_>) -> OwnedValue {
    OwnedValue::try_from(value).expect("fixture value")
}

fn path(text: &str) -> OwnedValue {
    owned(Value::from(
        ObjectPath::try_from(text).expect("fixture path"),
    ))
}

fn props(entries: Vec<(&str, OwnedValue)>) -> Props {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

/// An access point, wrapped in variants the way a nested decode leaves it.
pub fn access_point(ssid: &str, strength: u8, flags: u32, wpa: u32, rsn: u32) -> Props {
    props(vec![
        ("Ssid", owned(Value::from(ssid.as_bytes().to_vec()))),
        ("Strength", owned(Value::new(Value::from(strength)))),
        ("Flags", owned(Value::from(flags))),
        ("WpaFlags", owned(Value::from(wpa))),
        ("RsnFlags", owned(Value::from(rsn))),
        ("Frequency", owned(Value::from(5180u32))),
        ("HwAddress", owned(Value::from("AA:BB:CC:DD:EE:01"))),
    ])
}

/// An IP4Config with the modern `AddressData` and `NameserverData`.
pub fn ip4_modern() -> Props {
    let address: HashMap<&str, Value> = HashMap::from([
        ("address", Value::from("192.168.1.42")),
        ("prefix", Value::from(24u32)),
    ]);
    let dns_a: HashMap<&str, Value> = HashMap::from([("address", Value::from("1.1.1.1"))]);
    let dns_b: HashMap<&str, Value> = HashMap::from([("address", Value::from("9.9.9.9"))]);
    props(vec![
        ("AddressData", owned(Value::from(vec![address]))),
        ("Gateway", owned(Value::from("192.168.1.1"))),
        ("NameserverData", owned(Value::from(vec![dns_a, dns_b]))),
    ])
}

/// An IP4Config from an older NetworkManager: `Nameservers` as `au` in network order.
pub fn ip4_legacy() -> Props {
    let raw = u32::from_ne_bytes([192, 168, 1, 1]);
    props(vec![
        ("Gateway", owned(Value::from(""))),
        ("Nameservers", owned(Value::from(vec![raw]))),
    ])
}

/// A `Device` plus its `Device.Wireless` half.
pub fn wifi_device() -> (Props, Props) {
    let device = props(vec![
        ("Interface", owned(Value::from("wlp2s0"))),
        ("State", owned(Value::from(100u32))),
        ("DeviceType", owned(Value::from(2u32))),
        ("HwAddress", owned(Value::from("11:22:33:44:55:66"))),
    ]);
    let wireless = props(vec![
        (
            "ActiveAccessPoint",
            path("/org/freedesktop/NetworkManager/AccessPoint/3"),
        ),
        ("LastScan", owned(Value::from(123_456i64))),
        ("HwAddress", owned(Value::from("11:22:33:44:55:66"))),
    ]);
    (device, wireless)
}

/// A `Device` plus its `Device.Wired` half, with no active access point.
pub fn wired_device() -> (Props, Props) {
    let device = props(vec![
        ("Interface", owned(Value::from("enp3s0"))),
        ("State", owned(Value::from(30u32))),
    ]);
    let wired = props(vec![
        ("Speed", owned(Value::from(1000u32))),
        ("Carrier", owned(Value::from(false))),
        ("HwAddress", owned(Value::from("66:55:44:33:22:11"))),
    ]);
    (device, wired)
}

/// A Wi-Fi profile.
pub fn wifi_settings(ssid: &str, autoconnect: Option<bool>) -> Settings {
    let mut connection = props(vec![
        ("id", owned(Value::from(ssid))),
        (
            "uuid",
            owned(Value::from("6a2f3d4e-0000-4000-8000-000000000001")),
        ),
        ("type", owned(Value::from("802-11-wireless"))),
    ]);
    if let Some(flag) = autoconnect {
        connection.insert("autoconnect".to_string(), owned(Value::from(flag)));
    }
    let wireless = props(vec![
        ("ssid", owned(Value::from(ssid.as_bytes().to_vec()))),
        ("mode", owned(Value::from("infrastructure"))),
    ]);
    HashMap::from([
        ("connection".to_string(), connection),
        ("802-11-wireless".to_string(), wireless),
    ])
}

/// A profile of `kind` with no Wi-Fi part.
pub fn plain_settings(id: &str, kind: &str) -> Settings {
    let connection = props(vec![
        ("id", owned(Value::from(id))),
        (
            "uuid",
            owned(Value::from("6a2f3d4e-0000-4000-8000-000000000002")),
        ),
        ("type", owned(Value::from(kind))),
    ]);
    HashMap::from([("connection".to_string(), connection)])
}
