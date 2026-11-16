//! The few D-Bus calls the backend is built from, against NetworkManager.

use serde::Serialize;
use zbus::zvariant::{DynamicType, OwnedObjectPath, Value};
use zbus::Connection;

use crate::model::{Props, Settings};

pub const NM: &str = "org.freedesktop.NetworkManager";
pub const NM_PATH: &str = "/org/freedesktop/NetworkManager";
pub const SETTINGS_PATH: &str = "/org/freedesktop/NetworkManager/Settings";
pub const DEVICE: &str = "org.freedesktop.NetworkManager.Device";
pub const WIRELESS: &str = "org.freedesktop.NetworkManager.Device.Wireless";
pub const WIRED: &str = "org.freedesktop.NetworkManager.Device.Wired";
pub const ACCESS_POINT: &str = "org.freedesktop.NetworkManager.AccessPoint";
pub const IP4: &str = "org.freedesktop.NetworkManager.IP4Config";
pub const ACTIVE: &str = "org.freedesktop.NetworkManager.Connection.Active";
pub const SETTINGS: &str = "org.freedesktop.NetworkManager.Settings";
pub const CONNECTION: &str = "org.freedesktop.NetworkManager.Settings.Connection";
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";

/// The bus NetworkManager lives on: the system bus, or the session bus when
/// `FLUX_NM_BUS=session`, which is how the backend is tested against a mock.
pub async fn connect() -> zbus::Result<Connection> {
    if std::env::var("FLUX_NM_BUS").is_ok_and(|bus| bus == "session") {
        Connection::session().await
    } else {
        Connection::system().await
    }
}

/// Whether `error` says NetworkManager is not on the bus at all.
pub fn not_running(error: &zbus::Error) -> bool {
    let text = error.to_string();
    text.contains("ServiceUnknown") || text.contains("NameHasNoOwner")
}

/// Call `method` and decode its reply as `R`.
pub async fn call<B, R>(
    conn: &Connection,
    path: &str,
    iface: &str,
    method: &str,
    body: &B,
) -> zbus::Result<R>
where
    B: Serialize + DynamicType,
    R: for<'d> serde::Deserialize<'d> + zbus::zvariant::Type,
{
    let reply = conn
        .call_method(Some(NM), path, Some(iface), method, body)
        .await?;
    reply.body().deserialize::<R>()
}

/// Call `method` and drop its reply.
pub async fn invoke<B>(
    conn: &Connection,
    path: &str,
    iface: &str,
    method: &str,
    body: &B,
) -> zbus::Result<()>
where
    B: Serialize + DynamicType,
{
    conn.call_method(Some(NM), path, Some(iface), method, body)
        .await?;
    Ok(())
}

/// Every property of `iface` on `path`.
pub async fn get_all(conn: &Connection, path: &str, iface: &str) -> zbus::Result<Props> {
    call(conn, path, PROPERTIES, "GetAll", &(iface,)).await
}

/// Set one property.
pub async fn set(
    conn: &Connection,
    path: &str,
    iface: &str,
    name: &str,
    value: Value<'_>,
) -> zbus::Result<()> {
    invoke(conn, path, PROPERTIES, "Set", &(iface, name, value)).await
}

/// The saved profiles' object paths.
pub async fn list_connections(conn: &Connection) -> zbus::Result<Vec<OwnedObjectPath>> {
    call(conn, SETTINGS_PATH, SETTINGS, "ListConnections", &()).await
}

/// A saved profile without its secrets.
pub async fn get_settings(conn: &Connection, path: &str) -> zbus::Result<Settings> {
    call(conn, path, CONNECTION, "GetSettings", &()).await
}

/// An object path argument, with `/` for none.
pub fn object(path: Option<&str>) -> zbus::Result<OwnedObjectPath> {
    Ok(OwnedObjectPath::try_from(path.unwrap_or("/"))?)
}
