//! The calls that change NetworkManager's state.

use std::collections::HashMap;

use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};
use zbus::Connection;

use super::bus::{self, CONNECTION, DEVICE, NM, NM_PATH};
use crate::model::{Security, Settings, Snapshot};

/// What joining a network asks for.
#[derive(Clone, Debug, PartialEq)]
pub struct Join {
    pub ssid: String,
    pub ap_path: Option<String>,
    pub security: Security,
    pub password: Option<String>,
    pub hidden: bool,
}

/// Turn the Wi-Fi radio on or off.
pub async fn set_wifi(conn: &Connection, on: bool) -> zbus::Result<()> {
    bus::set(conn, NM_PATH, NM, "WirelessEnabled", Value::from(on)).await
}

/// Airplane mode: Wi-Fi and mobile broadband radios together.
pub async fn set_airplane(conn: &Connection, on: bool) -> zbus::Result<()> {
    set_wifi(conn, !on).await?;
    bus::set(conn, NM_PATH, NM, "WwanEnabled", Value::from(!on)).await
}

/// Ask the adapter for fresh scan results.
pub async fn scan(conn: &Connection, device: &str) -> zbus::Result<()> {
    let options: HashMap<&str, Value> = HashMap::new();
    bus::invoke(conn, device, bus::WIRELESS, "RequestScan", &(options,)).await
}

/// The profiles one join touched: the one it `created`, if any, and the saved
/// ones it `replaces` once the new one has connected.
#[derive(Debug, Default)]
pub struct Joined {
    pub created: Option<String>,
    pub replaces: Vec<String>,
}

/// Join `join.ssid`: reuse the saved profile when no new password is given,
/// otherwise add a fresh one beside any saved profile. The saved ones stay
/// until the new one connects, so a failed attempt never forgets a network.
pub async fn connect(
    conn: &Connection,
    snapshot: &Snapshot,
    device: &str,
    join: &Join,
) -> zbus::Result<Joined> {
    let saved: Vec<String> = snapshot
        .saved_for(&join.ssid)
        .map(|profile| profile.path.clone())
        .collect();
    let device_path = bus::object(Some(device))?;
    let specific = bus::object(join.ap_path.as_deref())?;
    if join.password.is_none() && !join.hidden {
        if let Some(profile) = saved.first() {
            let body = (bus::object(Some(profile))?, device_path, specific);
            bus::invoke(conn, NM_PATH, NM, "ActivateConnection", &body).await?;
            return Ok(Joined::default());
        }
    }
    let settings = profile(join);
    let (created, _active): (OwnedObjectPath, OwnedObjectPath) = bus::call(
        conn,
        NM_PATH,
        NM,
        "AddAndActivateConnection",
        &(settings, device_path, specific),
    )
    .await?;
    Ok(Joined {
        created: Some(created.as_str().to_string()),
        replaces: saved,
    })
}

/// The settings of a new Wi-Fi profile.
fn profile(join: &Join) -> HashMap<&'static str, HashMap<&'static str, Value<'static>>> {
    let mut wireless: HashMap<&str, Value> = HashMap::from([
        ("ssid", Value::from(join.ssid.as_bytes().to_vec())),
        ("mode", Value::from("infrastructure")),
    ]);
    if join.hidden {
        wireless.insert("hidden", Value::from(true));
    }
    let connection: HashMap<&str, Value> = HashMap::from([
        ("id", Value::from(join.ssid.clone())),
        ("type", Value::from("802-11-wireless")),
    ]);
    let mut settings = HashMap::from([("connection", connection), ("802-11-wireless", wireless)]);
    if let Some(key_mgmt) = join.security.key_mgmt() {
        let mut security: HashMap<&str, Value> =
            HashMap::from([("key-mgmt", Value::from(key_mgmt))]);
        if let Some(password) = &join.password {
            security.insert("psk", Value::from(password.clone()));
        }
        settings.insert("802-11-wireless-security", security);
    }
    settings
}

/// Drop the Wi-Fi connection.
pub async fn disconnect(conn: &Connection, device: &str) -> zbus::Result<()> {
    bus::invoke(conn, device, DEVICE, "Disconnect", &()).await
}

/// Remove a saved profile.
pub async fn delete(conn: &Connection, profile: &str) -> zbus::Result<()> {
    bus::invoke(conn, profile, CONNECTION, "Delete", &()).await
}

/// Forget every saved profile of `ssid`.
pub async fn forget(conn: &Connection, snapshot: &Snapshot, ssid: &str) -> zbus::Result<()> {
    for profile in snapshot.saved_for(ssid) {
        delete(conn, &profile.path).await?;
    }
    Ok(())
}

/// Turn auto-connect on or off for every saved profile of `ssid`.
///
/// `Update` replaces the whole profile, so the secrets are read first and sent
/// back with it. A profile whose secrets live in a user agent keeps them there.
pub async fn set_autoconnect(
    conn: &Connection,
    snapshot: &Snapshot,
    ssid: &str,
    on: bool,
) -> zbus::Result<()> {
    for saved in snapshot.saved_for(ssid) {
        let mut settings = bus::get_settings(conn, &saved.path).await?;
        merge_secrets(conn, &saved.path, &mut settings).await;
        if let Some(connection) = settings.get_mut("connection") {
            connection.insert("autoconnect".to_string(), OwnedValue::from(on));
        }
        bus::invoke(conn, &saved.path, CONNECTION, "Update", &(settings,)).await?;
    }
    Ok(())
}

async fn merge_secrets(conn: &Connection, profile: &str, settings: &mut Settings) {
    let name = "802-11-wireless-security";
    if !settings.contains_key(name) {
        return;
    }
    let Ok(secrets) =
        bus::call::<_, Settings>(conn, profile, CONNECTION, "GetSecrets", &(name,)).await
    else {
        return;
    };
    for (setting, values) in secrets {
        settings.entry(setting).or_default().extend(values);
    }
}

/// Bring a VPN up or down.
pub async fn set_vpn(
    conn: &Connection,
    profile: &str,
    active: Option<&str>,
    on: bool,
) -> zbus::Result<()> {
    if on {
        let body = (
            bus::object(Some(profile))?,
            bus::object(None)?,
            bus::object(None)?,
        );
        bus::invoke(conn, NM_PATH, NM, "ActivateConnection", &body).await
    } else {
        let Some(active) = active else {
            return Ok(());
        };
        bus::invoke(
            conn,
            NM_PATH,
            NM,
            "DeactivateConnection",
            &(bus::object(Some(active))?,),
        )
        .await
    }
}
