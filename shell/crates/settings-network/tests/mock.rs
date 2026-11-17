//! The backend against python-dbusmock's NetworkManager template on a private
//! session bus. Run through `tests/mock-nm.sh`, which starts the bus and the
//! mock and sets `FLUX_NM_BUS=session`; plain `cargo test` skips it.

use std::collections::HashMap;
use std::time::Duration;

use async_channel::Receiver;
use settings_network::backend::{Backend, Command, Event, Join};
use settings_network::model::wifi::Failure;
use settings_network::model::{Security, Snapshot, Status};
use zbus::zvariant::{OwnedObjectPath, Value};
use zbus::Connection;

const NM: &str = "org.freedesktop.NetworkManager";
const MOCK: &str = "org.freedesktop.DBus.Mock";
const ROOT: &str = "/org/freedesktop";
const PSK: u32 = 0x100;
const WAIT: Duration = Duration::from_secs(10);

async fn mock<B>(conn: &Connection, path: &str, method: &str, body: &B) -> zbus::Message
where
    B: serde::Serialize + zbus::zvariant::DynamicType,
{
    conn.call_method(Some(NM), path, Some(MOCK), method, body)
        .await
        .expect(method)
}

async fn mock_path<B>(conn: &Connection, method: &str, body: &B) -> String
where
    B: serde::Serialize + zbus::zvariant::DynamicType,
{
    mock(conn, ROOT, method, body)
        .await
        .body()
        .deserialize::<String>()
        .expect("path")
}

async fn until_owned(conn: &Connection) {
    let proxy = zbus::fdo::DBusProxy::new(conn).await.expect("bus proxy");
    for _ in 0..100 {
        let name = zbus::names::BusName::try_from(NM).expect("name");
        if proxy.name_has_owner(name).await.unwrap_or(false) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("the NetworkManager mock never took its name");
}

async fn next(events: &Receiver<Event>) -> Event {
    tokio::time::timeout(WAIT, events.recv())
        .await
        .expect("event in time")
        .expect("worker alive")
}

async fn snapshot_where(events: &Receiver<Event>, test: impl Fn(&Snapshot) -> bool) -> Snapshot {
    loop {
        if let Event::Snapshot(snapshot) = next(events).await {
            if test(&snapshot) {
                return *snapshot;
            }
        }
    }
}

async fn outcome(events: &Receiver<Event>, ssid: &str) -> Event {
    loop {
        let event = next(events).await;
        if matches!(&event, Event::Connected(name) | Event::Failed(name, _) if name == ssid) {
            return event;
        }
    }
}

async fn stage(conn: &Connection) -> String {
    let device = mock_path(conn, "AddWiFiDevice", &("wlan0", "wlan0", 30i32)).await;
    let aps = [
        ("ap1", "Maison", 82u8, PSK),
        ("ap2", "Maison", 40, PSK),
        ("ap3", "Cafe", 60, 0),
    ];
    for (name, ssid, strength, security) in aps {
        let body = (
            device.as_str(),
            name,
            ssid,
            "AA:BB:CC:DD:EE:01",
            2u32,
            2412u32,
            54000u32,
            strength,
            security,
        );
        mock_path(conn, "AddAccessPoint", &body).await;
    }
    mock_path(conn, "AddEthernetDevice", &("eth0", "eth0", 100i32)).await;
    let vpn: HashMap<&str, HashMap<&str, Value>> = HashMap::from([
        (
            "connection",
            HashMap::from([
                ("id", Value::from("Office VPN")),
                ("type", Value::from("vpn")),
            ]),
        ),
        (
            "vpn",
            HashMap::from([(
                "service-type",
                Value::from("org.freedesktop.NetworkManager.openvpn"),
            )]),
        ),
    ]);
    let settings = "/org/freedesktop/NetworkManager/Settings";
    let _: OwnedObjectPath = conn
        .call_method(
            Some(NM),
            settings,
            Some("org.freedesktop.NetworkManager.Settings"),
            "AddConnection",
            &(vpn,),
        )
        .await
        .expect("add vpn")
        .body()
        .deserialize()
        .expect("vpn path");
    device
}

/// Make the next AddAndActivateConnection create its profile and then fail the
/// way NetworkManager does on a wrong WPA password: device state 120, reason 7.
async fn fail_next_join(conn: &Connection, device: &str) {
    let code = format!(
        "name = args[2].rsplit('/', 1)[1]\n\
         ssid = ''.join(chr(b) for b in args[0]['802-11-wireless']['ssid'])\n\
         path = objects['{ROOT}'].AddWiFiConnection(args[1], name, ssid, '')\n\
         objects['{device}'].EmitSignal('org.freedesktop.NetworkManager.Device', 'StateChanged', 'uuu', \
         [dbus.UInt32(120), dbus.UInt32(60), dbus.UInt32(7)])\n\
         ret = (dbus.ObjectPath(path), dbus.ObjectPath('/'))"
    );
    let body = (
        NM,
        "AddAndActivateConnection",
        "a{sa{sv}}oo",
        "oo",
        code.as_str(),
    );
    mock(conn, "/org/freedesktop/NetworkManager", "AddMethod", &body).await;
}

fn join(ssid: &str, ap: &str, security: Security, password: Option<&str>) -> Command {
    Command::Connect(Join {
        ssid: ssid.to_string(),
        ap_path: Some(format!("/org/freedesktop/NetworkManager/AccessPoint/{ap}")),
        security,
        password: password.map(str::to_string),
        hidden: false,
    })
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "needs tests/mock-nm.sh"]
async fn list_connect_forget_and_radios_against_the_mock() {
    let conn = Connection::session().await.expect("session bus");
    until_owned(&conn).await;
    let device = stage(&conn).await;
    let (backend, events) = Backend::start();

    let first = snapshot_where(&events, |s| {
        s.status == Status::Ready && s.networks.len() == 2
    })
    .await;
    assert_eq!(first.networks[0].ssid, "Maison");
    assert_eq!(first.networks[0].strength, 82);
    assert_eq!(first.networks[0].security, Security::Personal);
    assert_eq!(first.networks[1].security, Security::Open);
    assert_eq!(first.wired.len(), 1);
    assert_eq!(first.vpns.len(), 1);
    assert!(first.wifi_enabled);

    backend.send(join("Cafe", "ap3", Security::Open, None));
    assert_eq!(
        outcome(&events, "Cafe").await,
        Event::Connected("Cafe".to_string())
    );
    snapshot_where(&events, |s| s.saved_for("Cafe").count() == 1).await;

    backend.send(Command::SetAutoconnect("Cafe".to_string(), false));
    snapshot_where(&events, |s| s.saved_for("Cafe").all(|p| !p.autoconnect)).await;

    backend.send(Command::Forget("Cafe".to_string()));
    snapshot_where(&events, |s| s.saved_for("Cafe").count() == 0).await;

    fail_next_join(&conn, &device).await;
    backend.send(join(
        "Maison",
        "ap1",
        Security::Personal,
        Some("wrong-password"),
    ));
    assert_eq!(
        outcome(&events, "Maison").await,
        Event::Failed("Maison".to_string(), Failure::WrongPassword)
    );
    snapshot_where(&events, |s| s.saved_for("Maison").count() == 0).await;

    let vpn = first.vpns[0].path.clone();
    backend.send(Command::SetVpn(vpn.clone(), true));
    snapshot_where(&events, |s| s.vpns[0].active.is_some()).await;
    backend.send(Command::SetVpn(vpn, false));
    snapshot_where(&events, |s| s.vpns[0].active.is_none()).await;

    backend.send(Command::SetWifi(false));
    snapshot_where(&events, |s| !s.wifi_enabled && !s.airplane()).await;
    backend.send(Command::SetAirplane(true));
    snapshot_where(&events, |s| s.airplane()).await;
    backend.send(Command::SetAirplane(false));
    snapshot_where(&events, |s| s.wifi_enabled && !s.airplane()).await;
}
