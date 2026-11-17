//! Sample data for design checks, touching no system service:
//! `FLUX_NETWORK_PREVIEW=<en|fr>:<screen>`, where the screen is one of `list`,
//! `password`, `wrong`, `details`, `connecting`, `unsupported`, `other`, `missing`, `error`,
//! `no-adapter`, `off`, `airplane` or `loading`.

use std::rc::Rc;

use super::{Expand, Page};
use crate::model::wifi::{self, Failure};
use crate::model::{
    AccessPoint, IpInfo, Kind, Link, Saved, Security, Snapshot, Status, Vpn, WifiDevice, Wired,
};

fn ap(index: usize, ssid: &str, strength: u8, security: Security, frequency: u32) -> AccessPoint {
    AccessPoint {
        path: format!("/ap/{index}"),
        ssid: ssid.to_string(),
        strength,
        security,
        frequency,
    }
}

fn saved(index: usize, ssid: &str, autoconnect: bool) -> Saved {
    Saved {
        path: format!("/saved/{index}"),
        id: ssid.to_string(),
        uuid: String::new(),
        kind: Kind::Wifi {
            ssid: ssid.to_string(),
        },
        autoconnect,
    }
}

/// A laptop at home: on its own network, neighbours around, a cable unplugged and a VPN.
pub fn sample() -> Snapshot {
    let aps = vec![
        ap(1, "Maison", 86, Security::Personal, 5180),
        ap(2, "Maison", 52, Security::Personal, 2437),
        ap(3, "Freebox-7A21C4", 71, Security::Personal, 5220),
        ap(4, "Café du Port", 64, Security::Open, 2412),
        ap(5, "Bureau", 38, Security::Wpa3, 5745),
        ap(6, "eduroam", 45, Security::Enterprise, 5500),
        ap(7, "Livebox-90E2", 22, Security::Personal, 2462),
        ap(8, "", 60, Security::Personal, 2412),
    ];
    let saved = vec![saved(1, "Maison", true), saved(2, "Bureau", false)];
    let mut snapshot = Snapshot::empty(Status::Ready);
    snapshot.wifi_enabled = true;
    snapshot.wifi_hardware = true;
    snapshot.wwan_enabled = true;
    snapshot.wifi = Some(WifiDevice {
        path: "/dev/wlan0".to_string(),
        iface: "wlp2s0".to_string(),
        mac: "3C:22:FB:91:0E:47".to_string(),
        link: Link::Connected,
        active_ap: Some("/ap/1".to_string()),
        ip: IpInfo {
            address: Some("192.168.1.42/24".to_string()),
            gateway: Some("192.168.1.1".to_string()),
            dns: vec!["192.168.1.1".to_string(), "9.9.9.9".to_string()],
        },
        last_scan: 1,
    });
    snapshot.wired = vec![Wired {
        path: "/dev/eth0".to_string(),
        iface: "enp3s0".to_string(),
        mac: "A0:36:9F:12:44:01".to_string(),
        link: Link::Unavailable,
        speed: 0,
        carrier: false,
        ip: IpInfo::default(),
    }];
    snapshot.vpns = vec![Vpn {
        path: "/saved/9".to_string(),
        name: "Facile Studio".to_string(),
        active: None,
        link: Link::Disconnected,
    }];
    let known = saved.iter().filter_map(|profile| match &profile.kind {
        Kind::Wifi { ssid } => Some(ssid.clone()),
        _ => None,
    });
    snapshot.networks = wifi::networks(&aps, Some("/ap/1"), &known.collect());
    snapshot.saved = saved;
    snapshot
}

/// Fill `page` with the screen `name` names.
pub fn apply(page: &Rc<Page>, name: &str) {
    let screen = name.split_once(':').map_or(name, |(_, screen)| screen);
    let mut snapshot = sample();
    match screen {
        "password" => {
            *page.open.borrow_mut() = Some(("Freebox-7A21C4".to_string(), Expand::Password))
        }
        "wrong" => {
            *page.open.borrow_mut() = Some(("Freebox-7A21C4".to_string(), Expand::Password));
            page.failures
                .borrow_mut()
                .insert("Freebox-7A21C4".to_string(), Failure::WrongPassword);
        }
        "details" => *page.open.borrow_mut() = Some(("Maison".to_string(), Expand::Details)),
        "connecting" => {
            snapshot.scanning = true;
            *page.joining.borrow_mut() = Some("Café du Port".to_string());
        }
        "unsupported" => {
            *page.open.borrow_mut() = Some(("eduroam".to_string(), Expand::Unsupported))
        }
        "missing" => snapshot = Snapshot::empty(Status::NotRunning),
        "error" => {
            snapshot = Snapshot::empty(Status::Error(
                "org.freedesktop.DBus.Error.AccessDenied".to_string(),
            ))
        }
        "loading" => snapshot = Snapshot::empty(Status::Loading),
        "no-adapter" => {
            snapshot.wifi = None;
            snapshot.networks.clear();
        }
        "other" => {
            if let Some(other) = &page.other {
                other.set_open(true);
            }
        }
        "off" => snapshot.wifi_enabled = false,
        "airplane" => {
            snapshot.wifi_enabled = false;
            snapshot.wwan_enabled = false;
        }
        _ => {}
    }
    *page.snapshot.borrow_mut() = snapshot;
}
