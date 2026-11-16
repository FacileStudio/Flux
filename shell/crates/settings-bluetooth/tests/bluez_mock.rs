//! The backend against python-dbusmock's `bluez5` template on a private bus:
//! listing, the pairing agent's callbacks, pairing, connecting, forgetting,
//! power, discovery and BlueZ going away.
//!
//! It needs `dbus-daemon` and a Python with `dbusmock` on `PATH`, and passes
//! without running when either is missing. `scripts/test-mock.sh` in this
//! crate provides both through Nix.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command as Process, Stdio};
use std::time::Duration;

use async_channel::Receiver;
use settings_bluetooth::backend::{self, Answer, Command, Event, Op, AGENT_PATH};
use settings_bluetooth::model::{Prompt, Snapshot, Step};
use serde::Serialize;
use zbus::zvariant::{DynamicType, ObjectPath, OwnedObjectPath, Value};
use zbus::Connection;

const HEADPHONES: &str = "11:22:33:44:55:01";
const KEYBOARD: &str = "11:22:33:44:55:02";
const WAIT: Duration = Duration::from_secs(10);

struct Bus {
    daemon: Child,
    mock: Option<Child>,
    address: String,
}

impl Drop for Bus {
    fn drop(&mut self) {
        if let Some(mock) = self.mock.as_mut() {
            let _ = mock.kill();
            let _ = mock.wait();
        }
        let _ = self.daemon.kill();
        let _ = self.daemon.wait();
    }
}

fn available() -> bool {
    let daemon = Process::new("dbus-daemon").arg("--version").stdout(Stdio::null()).status();
    let mock = Process::new("python3")
        .args(["-c", "import dbusmock"])
        .stderr(Stdio::null())
        .status();
    daemon.is_ok_and(|status| status.success()) && mock.is_ok_and(|status| status.success())
}

const BUS_CONFIG: &str = r#"<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <type>session</type>
  <listen>unix:tmpdir=/tmp</listen>
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
  </policy>
</busconfig>
"#;

fn start_bus() -> Option<Bus> {
    let config = std::env::temp_dir().join(format!("flux-bluetooth-bus-{}.conf", std::process::id()));
    std::fs::write(&config, BUS_CONFIG).ok()?;
    let mut daemon = Process::new("dbus-daemon")
        .arg(format!("--config-file={}", config.display()))
        .args(["--nofork", "--nopidfile", "--print-address=1"])
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;
    let mut address = String::new();
    BufReader::new(daemon.stdout.take()?).read_line(&mut address).ok()?;
    let address = address.trim().to_string();
    let _ = std::fs::remove_file(&config);
    if address.is_empty() {
        return None;
    }
    let mock = Process::new("python3")
        .args(["-m", "dbusmock", "--template", "bluez5", "--session"])
        .env("DBUS_SESSION_BUS_ADDRESS", &address)
        .stdout(Stdio::null())
        .spawn()
        .ok();
    Some(Bus { daemon, mock, address })
}

async fn call<B>(bus: &Connection, path: &str, interface: &str, method: &str, body: &B) -> zbus::Result<zbus::Message>
where
    B: Serialize + DynamicType,
{
    bus.call_method(Some("org.bluez"), path, Some(interface), method, body).await
}

async fn wait_for_bluez(bus: &Connection) -> bool {
    for _ in 0..100 {
        let reply = bus
            .call_method(
                Some("org.freedesktop.DBus"),
                "/org/freedesktop/DBus",
                Some("org.freedesktop.DBus"),
                "NameHasOwner",
                &("org.bluez",),
            )
            .await;
        if reply.and_then(|reply| reply.body().deserialize::<bool>()).unwrap_or(false) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

async fn next(events: &Receiver<Event>, mut wanted: impl FnMut(&Event) -> bool) -> Event {
    let found = tokio::time::timeout(WAIT, async {
        loop {
            match events.recv().await {
                Ok(event) if wanted(&event) => return Some(event),
                Ok(_) => {}
                Err(_) => return None,
            }
        }
    })
    .await;
    found.ok().flatten().expect("the backend never sent the expected event")
}

async fn snapshot(events: &Receiver<Event>, mut wanted: impl FnMut(&Snapshot) -> bool) -> Snapshot {
    match next(events, |event| matches!(event, Event::Snapshot(snapshot) if wanted(snapshot))).await {
        Event::Snapshot(snapshot) => snapshot,
        _ => unreachable!(),
    }
}

async fn request(events: &Receiver<Event>) -> backend::Request {
    match next(events, |event| matches!(event, Event::Request(_))).await {
        Event::Request(request) => request,
        _ => unreachable!(),
    }
}

async fn agent_owner(bus: &Connection) -> Option<String> {
    let names = bus
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "ListNames",
            &(),
        )
        .await
        .ok()?
        .body()
        .deserialize::<Vec<String>>()
        .ok()?;
    let own = bus.unique_name().map(|name| name.to_string());
    for name in names.into_iter().filter(|name| name.starts_with(':') && Some(name) != own.as_ref()) {
        let xml = bus
            .call_method(
                Some(name.as_str()),
                AGENT_PATH,
                Some("org.freedesktop.DBus.Introspectable"),
                "Introspect",
                &(),
            )
            .await
            .and_then(|reply| reply.body().deserialize::<String>());
        if xml.is_ok_and(|xml| xml.contains("org.bluez.Agent1")) {
            return Some(name);
        }
    }
    None
}

async fn within<T>(label: &str, future: impl std::future::Future<Output = T>) -> T {
    match tokio::time::timeout(WAIT, future).await {
        Ok(value) => value,
        Err(_) => panic!("timed out: {label}"),
    }
}

fn device_path(address: &str) -> String {
    format!("/org/bluez/hci0/dev_{}", address.replace(':', "_"))
}

async fn agent_call<B>(bus: Connection, owner: String, method: &'static str, body: B) -> Result<zbus::Message, String>
where
    B: Serialize + DynamicType + Send + 'static,
{
    bus.call_method(Some(owner.as_str()), AGENT_PATH, Some("org.bluez.Agent1"), method, &body)
        .await
        .map_err(|error| match error {
            zbus::Error::MethodError(name, _, _) => name.to_string(),
            other => other.to_string(),
        })
}

#[test]
fn backend_against_the_bluez5_mock() {
    if !available() {
        eprintln!("skipped: dbus-daemon or python3 -m dbusmock is missing");
        return;
    }
    std::env::set_var("FLUX_BLUETOOTH_RFKILL", "/nonexistent");
    let mut bus = start_bus().expect("private bus");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    runtime.block_on(async {
        let control = zbus::connection::Builder::address(bus.address.as_str())
            .expect("address")
            .build()
            .await
            .expect("control connection");
        assert!(wait_for_bluez(&control).await, "the mock never claimed org.bluez");
        call(&control, "/", "org.bluez.Mock", "AddAdapter", &("hci0", "flux-test")).await.expect("adapter");
        for (address, name) in [(HEADPHONES, "Test Headphones"), (KEYBOARD, "Keyboard")] {
            call(&control, "/", "org.bluez.Mock", "AddDevice", &("hci0", address, name)).await.expect("device");
        }
        call(&control, "/", "org.bluez.Mock", "PairDevice", &("hci0", KEYBOARD)).await.expect("pair keyboard");
        let mut battery = HashMap::new();
        battery.insert("Percentage", Value::U8(77));
        call(
            &control,
            &device_path(KEYBOARD),
            "org.freedesktop.DBus.Mock",
            "AddProperties",
            &("org.bluez.Battery1", battery),
        )
        .await
        .expect("battery");
        call(
            &control,
            &device_path(HEADPHONES),
            "org.freedesktop.DBus.Properties",
            "Set",
            &("org.bluez.Device1", "Icon", Value::from("audio-headphones")),
        )
        .await
        .expect("icon");

        let no_filter: HashMap<&str, Value> = HashMap::new();
        call(&control, "/org/bluez/hci0", "org.bluez.Adapter1", "SetDiscoveryFilter", &(no_filter,))
            .await
            .expect("the mock's StartDiscovery needs a filter set first");

        let backend = backend::start_on(Some(bus.address.clone()));
        let events = backend.events.clone();
        let commands = backend.commands.clone();
        let headphones = device_path(HEADPHONES);

        let listed = snapshot(&events, |snapshot| snapshot.devices.len() == 2).await;
        let adapter = listed.adapter.clone().expect("adapter listed");
        assert_eq!(adapter.name, "flux-test");
        assert!(adapter.powered);
        let paired = listed.paired();
        assert_eq!(paired.len(), 1);
        assert_eq!(paired[0].display_name(), "Keyboard");
        assert_eq!(paired[0].battery, Some(77));
        let nearby = listed.nearby(false);
        assert_eq!(nearby.len(), 1);
        assert_eq!(nearby[0].display_name(), "Test Headphones");
        assert_eq!(nearby[0].rssi, Some(-79));
        assert_eq!(nearby[0].kind(), settings_bluetooth::model::Kind::Headphones);
        eprintln!("list: ok");

        let calls = within("method calls", call(&control, "/org/bluez", "org.freedesktop.DBus.Mock", "GetMethodCalls", &("RegisterAgent",)))
            .await
            .expect("method calls")
            .body()
            .deserialize::<Vec<(u64, Vec<zbus::zvariant::OwnedValue>)>>()
            .expect("calls body");
        let registered = calls.iter().any(|(_, args)| {
            let path = args.first().and_then(|arg| OwnedObjectPath::try_from(arg.try_clone().ok()?).ok());
            let capability = args.get(1).and_then(|arg| String::try_from(arg.try_clone().ok()?).ok());
            path.is_some_and(|path| path.as_str() == AGENT_PATH) && capability.as_deref() == Some("KeyboardDisplay")
        });
        assert!(registered, "the agent was not registered with KeyboardDisplay: {calls:?}");
        eprintln!("agent registered");
        let owner = within("agent owner", agent_owner(&control)).await.expect("agent served on the bus");
        let device = ObjectPath::try_from(headphones.as_str()).expect("path").into_owned();

        let pending = tokio::spawn(agent_call(
            control.clone(),
            owner.clone(),
            "RequestConfirmation",
            (device.clone(), 123_456u32),
        ));
        let asked = request(&events).await;
        assert_eq!(asked.device, headphones);
        assert_eq!(asked.prompt, Prompt::Confirm(123_456));
        asked.reply.expect("confirmation needs a reply").try_send(Answer::Accept).expect("reply");
        assert!(within("agent reply", pending).await.expect("join").is_ok(), "an accepted confirmation must succeed");

        let pending = tokio::spawn(agent_call(control.clone(), owner.clone(), "RequestPinCode", (device.clone(),)));
        let asked = request(&events).await;
        assert_eq!(asked.prompt, Prompt::RequestPin);
        asked.reply.expect("pin needs a reply").try_send(Answer::Pin("0000".into())).expect("reply");
        let pin = within("agent reply", pending).await.expect("join").expect("pin reply");
        assert_eq!(pin.body().deserialize::<String>().expect("pin body"), "0000");

        let pending = tokio::spawn(agent_call(
            control.clone(),
            owner.clone(),
            "RequestConfirmation",
            (device.clone(), 1u32),
        ));
        request(&events).await.reply.expect("reply").try_send(Answer::Reject).expect("reply");
        assert_eq!(within("agent reply", pending).await.expect("join").err().as_deref(), Some("org.bluez.Error.Rejected"));

        let pending = tokio::spawn(agent_call(control.clone(), owner.clone(), "RequestPasskey", (device.clone(),)));
        let _held = request(&events).await;
        within("cancel", agent_call(control.clone(), owner.clone(), "Cancel", ())).await.expect("cancel");
        next(&events, |event| matches!(event, Event::RequestCancelled)).await;
        assert_eq!(within("agent reply", pending).await.expect("join").err().as_deref(), Some("org.bluez.Error.Canceled"));

        agent_call(control.clone(), owner.clone(), "DisplayPasskey", (device.clone(), 81_034u32, 2u16))
            .await
            .expect("display passkey");
        let shown = request(&events).await;
        assert_eq!(shown.prompt, Prompt::DisplayPasskey { passkey: 81_034, entered: 2 });
        assert!(shown.reply.is_none());
        eprintln!("agent: ok");

        commands.try_send(Command::Pair(headphones.clone())).expect("send");
        for expected in [Step::Start(headphones.clone()), Step::Paired(headphones.clone())] {
            next(&events, |event| matches!(event, Event::Pairing(step) if *step == expected)).await;
        }
        next(&events, |event| {
            matches!(event, Event::Done { op: Op::Connect, failure: None, device } if *device == headphones)
        })
        .await;
        next(&events, |event| matches!(event, Event::Pairing(Step::Finished(_)))).await;
        snapshot(&events, |snapshot| {
            snapshot
                .device(&headphones)
                .is_some_and(|device| device.paired && device.trusted && device.connected)
        })
        .await;
        eprintln!("pair, trust, connect: ok");

        commands.try_send(Command::Pair(headphones.clone())).expect("send");
        next(&events, |event| matches!(event, Event::Pairing(Step::Paired(_)))).await;
        next(&events, |event| matches!(event, Event::Done { op: Op::Connect, failure: None, .. })).await;
        eprintln!("pairing an already paired, connected device succeeds: ok");

        commands.try_send(Command::Disconnect(headphones.clone())).expect("send");
        next(&events, |event| matches!(event, Event::Done { op: Op::Disconnect, failure: None, .. })).await;
        snapshot(&events, |snapshot| snapshot.device(&headphones).is_some_and(|device| !device.connected)).await;
        commands.try_send(Command::Disconnect(headphones.clone())).expect("send");
        next(&events, |event| matches!(event, Event::Done { op: Op::Disconnect, failure: None, .. })).await;
        commands.try_send(Command::Connect(headphones.clone())).expect("send");
        next(&events, |event| matches!(event, Event::Done { op: Op::Connect, failure: None, .. })).await;
        snapshot(&events, |snapshot| snapshot.device(&headphones).is_some_and(|device| device.connected)).await;
        eprintln!("connect, disconnect: ok");

        commands.try_send(Command::SetTrusted(headphones.clone(), false)).expect("send");
        snapshot(&events, |snapshot| snapshot.device(&headphones).is_some_and(|device| !device.trusted)).await;
        commands.try_send(Command::Forget(headphones.clone())).expect("send");
        next(&events, |event| matches!(event, Event::Done { op: Op::Forget, failure: None, .. })).await;
        snapshot(&events, |snapshot| snapshot.device(&headphones).is_none() && snapshot.devices.len() == 1).await;
        eprintln!("trust, forget: ok");

        commands.try_send(Command::Discover(true)).expect("send");
        snapshot(&events, |snapshot| snapshot.adapter.as_ref().is_some_and(|adapter| adapter.discovering)).await;
        commands.try_send(Command::SetVisible(true)).expect("send");
        snapshot(&events, |snapshot| snapshot.adapter.as_ref().is_some_and(|adapter| adapter.discoverable)).await;
        commands.try_send(Command::Discover(false)).expect("send");
        commands.try_send(Command::SetVisible(false)).expect("send");
        snapshot(&events, |snapshot| {
            snapshot.adapter.as_ref().is_some_and(|adapter| !adapter.discovering && !adapter.discoverable)
        })
        .await;
        commands.try_send(Command::SetPowered(false)).expect("send");
        snapshot(&events, |snapshot| snapshot.adapter.as_ref().is_some_and(|adapter| !adapter.powered)).await;
        commands.try_send(Command::SetPowered(true)).expect("send");
        snapshot(&events, |snapshot| snapshot.adapter.as_ref().is_some_and(|adapter| adapter.powered)).await;
        eprintln!("discovery, visibility, power: ok");

        if let Some(mut mock) = bus.mock.take() {
            let _ = mock.kill();
            let _ = mock.wait();
        }
        let gone = snapshot(&events, |snapshot| !snapshot.service).await;
        assert_eq!(gone.availability(), settings_bluetooth::model::Availability::NoService);
        eprintln!("service loss: ok");
    });
}
