use std::collections::HashSet;

use super::fixtures;
use super::parse;
use super::wifi::{self, Failure, PasswordProblem};
use super::{AccessPoint, Kind, Link, Security, Snapshot, Status};

const PSK: u32 = 0x100 | 0x8 | 0x80;
const SAE: u32 = 0x400 | 0x8 | 0x80;
const EAP: u32 = 0x200 | 0x8 | 0x80;

fn ap(path: &str, ssid: &str, strength: u8) -> AccessPoint {
    AccessPoint {
        path: path.to_string(),
        ssid: ssid.to_string(),
        strength,
        security: Security::Personal,
        frequency: 2412,
    }
}

#[test]
fn parses_an_access_point_through_variant_wrappers() {
    let props = fixtures::access_point("Maison", 72, 0x1, 0, PSK);
    let parsed = parse::access_point("/ap/1", &props).expect("access point");
    assert_eq!(parsed.ssid, "Maison");
    assert_eq!(parsed.strength, 72);
    assert_eq!(parsed.security, Security::Personal);
    assert_eq!(parsed.frequency, 5180);
}

#[test]
fn an_access_point_without_ssid_is_skipped() {
    let mut props = fixtures::access_point("x", 10, 0, 0, 0);
    props.remove("Ssid");
    assert!(parse::access_point("/ap/1", &props).is_none());
}

#[test]
fn security_from_flags() {
    assert_eq!(wifi::security(0, 0, 0), Security::Open);
    assert_eq!(wifi::security(0x1, 0, 0), Security::Wep);
    assert_eq!(wifi::security(0x1, PSK, 0), Security::Personal);
    assert_eq!(wifi::security(0x1, 0, SAE), Security::Wpa3);
    assert_eq!(wifi::security(0x1, 0, PSK | SAE), Security::Personal);
    assert_eq!(wifi::security(0x1, 0, EAP), Security::Enterprise);
    assert_eq!(wifi::security(0x1, 0, 0x2000), Security::Enterprise);
    assert_eq!(wifi::security(0, 0, 0x800), Security::Owe);
}

#[test]
fn security_capabilities() {
    assert!(!Security::Open.secured());
    assert!(!Security::Owe.secured());
    assert!(Security::Personal.secured());
    assert!(!Security::Enterprise.supported());
    assert!(!Security::Wep.supported());
    assert!(Security::Wpa3.supported());
    assert_eq!(Security::Personal.key_mgmt(), Some("wpa-psk"));
    assert_eq!(Security::Wpa3.key_mgmt(), Some("sae"));
    assert_eq!(Security::Open.key_mgmt(), None);
}

#[test]
fn signal_to_bars() {
    let cases = [
        (0, 0),
        (5, 0),
        (6, 1),
        (30, 1),
        (31, 2),
        (55, 2),
        (56, 3),
        (80, 3),
        (81, 4),
        (100, 4),
    ];
    for (strength, expected) in cases {
        assert_eq!(wifi::bars(strength), expected, "strength {strength}");
    }
}

#[test]
fn dedup_keeps_the_strongest_access_point_per_ssid() {
    let aps = vec![
        ap("/a", "Cafe", 40),
        ap("/b", "Cafe", 90),
        ap("/c", "Cafe", 60),
        ap("/d", "", 99),
    ];
    let list = wifi::networks(&aps, None, &HashSet::new());
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].ap_path, "/b");
    assert_eq!(list[0].strength, 90);
}

#[test]
fn dedup_keeps_the_active_access_point_even_when_weaker() {
    let aps = vec![ap("/a", "Home", 30), ap("/b", "Home", 90)];
    let list = wifi::networks(&aps, Some("/a"), &HashSet::new());
    assert_eq!(list[0].ap_path, "/a");
    assert!(list[0].active);
    let reversed = vec![ap("/b", "Home", 90), ap("/a", "Home", 30)];
    assert_eq!(
        wifi::networks(&reversed, Some("/a"), &HashSet::new())[0].ap_path,
        "/a"
    );
}

#[test]
fn sort_puts_connected_then_known_then_signal_then_name() {
    let aps = vec![
        ap("/1", "zeta", 90),
        ap("/2", "Alpha", 90),
        ap("/3", "known", 20),
        ap("/4", "weak", 10),
        ap("/5", "current", 15),
    ];
    let known = HashSet::from(["known".to_string()]);
    let list = wifi::networks(&aps, Some("/5"), &known);
    let names: Vec<&str> = list.iter().map(|network| network.ssid.as_str()).collect();
    assert_eq!(names, ["current", "known", "Alpha", "zeta", "weak"]);
    assert!(list[1].known);
}

#[test]
fn wpa2_password_rules() {
    let personal = Security::Personal;
    assert_eq!(
        wifi::check_password(personal, ""),
        Err(PasswordProblem::Empty)
    );
    assert_eq!(
        wifi::check_password(personal, "short"),
        Err(PasswordProblem::TooShort)
    );
    assert_eq!(wifi::check_password(personal, "12345678"), Ok(()));
    assert_eq!(wifi::check_password(personal, &"a".repeat(63)), Ok(()));
    assert_eq!(
        wifi::check_password(personal, &"a".repeat(65)),
        Err(PasswordProblem::TooLong)
    );
    assert_eq!(wifi::check_password(personal, &"0f".repeat(32)), Ok(()));
    assert_eq!(
        wifi::check_password(personal, &"zz".repeat(32)),
        Err(PasswordProblem::TooLong)
    );
    assert_eq!(
        wifi::check_password(personal, "mot de passé"),
        Err(PasswordProblem::NotAscii)
    );
}

#[test]
fn wpa3_accepts_any_non_empty_password() {
    assert_eq!(wifi::check_password(Security::Wpa3, "abc"), Ok(()));
    assert_eq!(wifi::check_password(Security::Wpa3, "é"), Ok(()));
    assert_eq!(
        wifi::check_password(Security::Wpa3, ""),
        Err(PasswordProblem::Empty)
    );
}

#[test]
fn failure_reasons() {
    assert_eq!(wifi::failure(7), Failure::WrongPassword);
    assert_eq!(wifi::failure(8), Failure::WrongPassword);
    assert_eq!(wifi::failure(11), Failure::Timeout);
    assert_eq!(wifi::failure(53), Failure::NotFound);
    assert_eq!(wifi::failure(5), Failure::NoAddress);
    assert_eq!(wifi::failure(36), Failure::Other(36));
}

#[test]
fn device_and_active_states() {
    assert_eq!(parse::link(100), Link::Connected);
    assert_eq!(parse::link(60), Link::Connecting);
    assert_eq!(parse::link(120), Link::Failed);
    assert_eq!(parse::link(30), Link::Disconnected);
    assert_eq!(parse::link(20), Link::Unavailable);
    assert_eq!(parse::active_link(1), Link::Connecting);
    assert_eq!(parse::active_link(2), Link::Connected);
    assert_eq!(parse::active_link(4), Link::Disconnected);
}

#[test]
fn ip4_modern_and_legacy() {
    let modern = parse::ip4(&fixtures::ip4_modern());
    assert_eq!(modern.address.as_deref(), Some("192.168.1.42/24"));
    assert_eq!(modern.gateway.as_deref(), Some("192.168.1.1"));
    assert_eq!(modern.dns, ["1.1.1.1", "9.9.9.9"]);
    let legacy = parse::ip4(&fixtures::ip4_legacy());
    assert_eq!(legacy.address, None);
    assert_eq!(legacy.gateway, None);
    assert_eq!(legacy.dns, ["192.168.1.1"]);
}

#[test]
fn wifi_and_wired_devices() {
    let (device, wireless) = fixtures::wifi_device();
    let wifi = parse::wifi_device("/dev/2", &device, &wireless, Default::default());
    assert_eq!(wifi.iface, "wlp2s0");
    assert_eq!(wifi.link, Link::Connected);
    assert_eq!(
        wifi.active_ap.as_deref(),
        Some("/org/freedesktop/NetworkManager/AccessPoint/3")
    );
    assert_eq!(wifi.last_scan, 123_456);
    let (device, wired) = fixtures::wired_device();
    let port = parse::wired("/dev/1", &device, &wired, Default::default());
    assert_eq!(port.speed, 1000);
    assert!(!port.carrier);
    assert_eq!(port.link, Link::Disconnected);
    assert_eq!(port.mac, "66:55:44:33:22:11");
}

#[test]
fn saved_profiles() {
    let home = parse::saved("/s/1", &fixtures::wifi_settings("Home", None));
    assert_eq!(
        home.kind,
        Kind::Wifi {
            ssid: "Home".to_string()
        }
    );
    assert!(home.autoconnect);
    let manual = parse::saved("/s/2", &fixtures::wifi_settings("Work", Some(false)));
    assert!(!manual.autoconnect);
    assert_eq!(
        parse::saved("/s/3", &fixtures::plain_settings("Office", "vpn")).kind,
        Kind::Vpn
    );
    assert_eq!(
        parse::saved("/s/4", &fixtures::plain_settings("wg0", "wireguard")).kind,
        Kind::Vpn
    );
    assert_eq!(
        parse::saved("/s/5", &fixtures::plain_settings("eth", "802-3-ethernet")).kind,
        Kind::Other
    );
}

#[test]
fn snapshot_helpers() {
    let mut snapshot = Snapshot::empty(Status::Ready);
    assert!(snapshot.airplane());
    snapshot.wwan_enabled = true;
    assert!(!snapshot.airplane());
    snapshot.saved = vec![
        parse::saved("/s/1", &fixtures::wifi_settings("Home", None)),
        parse::saved("/s/2", &fixtures::plain_settings("Office", "vpn")),
        parse::saved("/s/3", &fixtures::wifi_settings("Home", Some(false))),
    ];
    assert_eq!(snapshot.saved_for("Home").count(), 2);
    assert_eq!(snapshot.saved_for("Office").count(), 0);
}

#[test]
fn bands_and_ssid_bytes() {
    assert_eq!(wifi::band(2437), "2.4 GHz");
    assert_eq!(wifi::band(5180), "5 GHz");
    assert_eq!(wifi::band(6115), "6 GHz");
    assert_eq!(parse::ssid(b"Caf\xc3\xa9\0"), "Café");
    assert_eq!(parse::ssid(b"bad\xff"), "bad\u{fffd}");
}
