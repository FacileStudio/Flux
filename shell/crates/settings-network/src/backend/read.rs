//! Reading the whole of NetworkManager's state into one [`Snapshot`].

use std::collections::{HashMap, HashSet};

use zbus::Connection;

use super::bus::{self, ACCESS_POINT, ACTIVE, DEVICE, IP4, NM, NM_PATH, WIRED, WIRELESS};
use crate::model::parse::{self, DEVICE_ETHERNET, DEVICE_WIFI};
use crate::model::value::{bool_of, path_of, paths_of, u32_of};
use crate::model::{wifi, IpInfo, Kind, Link, Saved, Snapshot, Status, Vpn, WifiDevice};

/// Everything the page shows. A NetworkManager that is not on the bus reads as
/// [`Status::NotRunning`]; any other failure as [`Status::Error`].
pub async fn snapshot(conn: &Connection) -> Snapshot {
    match read(conn).await {
        Ok(snapshot) => snapshot,
        Err(error) if bus::not_running(&error) => Snapshot::empty(Status::NotRunning),
        Err(error) => Snapshot::empty(Status::Error(error.to_string())),
    }
}

async fn read(conn: &Connection) -> zbus::Result<Snapshot> {
    let nm = bus::get_all(conn, NM_PATH, NM).await?;
    let mut snapshot = Snapshot::empty(Status::Ready);
    snapshot.wifi_enabled = bool_of(&nm, "WirelessEnabled").unwrap_or(false);
    snapshot.wifi_hardware = bool_of(&nm, "WirelessHardwareEnabled").unwrap_or(true);
    snapshot.wwan_enabled = bool_of(&nm, "WwanEnabled").unwrap_or(false);
    snapshot.saved = saved(conn).await;
    let mut aps = Vec::new();
    for path in paths_of(&nm, "Devices") {
        let Ok(device) = bus::get_all(conn, &path, DEVICE).await else {
            continue;
        };
        let ip = ip(conn, &device).await;
        match u32_of(&device, "DeviceType") {
            Some(DEVICE_WIFI) if snapshot.wifi.is_none() => {
                let wireless = bus::get_all(conn, &path, WIRELESS)
                    .await
                    .unwrap_or_default();
                aps = access_points(conn, &paths_of(&wireless, "AccessPoints")).await;
                snapshot.wifi = Some(parse::wifi_device(&path, &device, &wireless, ip));
            }
            Some(DEVICE_ETHERNET) => {
                let wired = bus::get_all(conn, &path, WIRED).await.unwrap_or_default();
                snapshot
                    .wired
                    .push(parse::wired(&path, &device, &wired, ip));
            }
            _ => {}
        }
    }
    let active = active(conn, &paths_of(&nm, "ActiveConnections")).await;
    snapshot.vpns = vpns(&snapshot.saved, &active);
    snapshot.networks = networks(&aps, snapshot.wifi.as_ref(), &snapshot.saved);
    Ok(snapshot)
}

async fn ip(conn: &Connection, device: &crate::model::Props) -> IpInfo {
    let Some(path) = path_of(device, "Ip4Config") else {
        return IpInfo::default();
    };
    match bus::get_all(conn, &path, IP4).await {
        Ok(props) => parse::ip4(&props),
        Err(_) => IpInfo::default(),
    }
}

async fn access_points(conn: &Connection, paths: &[String]) -> Vec<crate::model::AccessPoint> {
    let mut list = Vec::new();
    for path in paths {
        if let Ok(props) = bus::get_all(conn, path, ACCESS_POINT).await {
            list.extend(parse::access_point(path, &props));
        }
    }
    list
}

async fn saved(conn: &Connection) -> Vec<Saved> {
    let Ok(paths) = bus::list_connections(conn).await else {
        return Vec::new();
    };
    let mut list = Vec::new();
    for path in paths {
        if let Ok(settings) = bus::get_settings(conn, path.as_str()).await {
            list.push(parse::saved(path.as_str(), &settings));
        }
    }
    list
}

/// Saved profile path to its active connection path and link.
async fn active(conn: &Connection, paths: &[String]) -> HashMap<String, (String, Link)> {
    let mut map = HashMap::new();
    for path in paths {
        if let Ok(props) = bus::get_all(conn, path, ACTIVE).await {
            if let Some(profile) = path_of(&props, "Connection") {
                let link = parse::active_link(u32_of(&props, "State").unwrap_or(0));
                map.insert(profile, (path.clone(), link));
            }
        }
    }
    map
}

fn vpns(saved: &[Saved], active: &HashMap<String, (String, Link)>) -> Vec<Vpn> {
    let mut list: Vec<Vpn> = saved
        .iter()
        .filter(|profile| profile.kind == Kind::Vpn)
        .map(|profile| {
            let state = active.get(&profile.path);
            Vpn {
                path: profile.path.clone(),
                name: profile.id.clone(),
                active: state.map(|(path, _)| path.clone()),
                link: state.map_or(Link::Disconnected, |(_, link)| *link),
            }
        })
        .collect();
    list.sort_by_key(|vpn| vpn.name.to_lowercase());
    list
}

fn networks(
    aps: &[crate::model::AccessPoint],
    device: Option<&WifiDevice>,
    saved: &[Saved],
) -> Vec<crate::model::Network> {
    let known: HashSet<String> = saved
        .iter()
        .filter_map(|profile| match &profile.kind {
            Kind::Wifi { ssid } => Some(ssid.clone()),
            _ => None,
        })
        .collect();
    let active = device.and_then(|device| device.active_ap.as_deref());
    wifi::networks(aps, active, &known)
}
