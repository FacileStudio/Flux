//! Sample data for the preview hook and the tests. Nothing here reaches a bus.

use crate::model::{Adapter, Device, Prompt, Radio, Service, Snapshot};

pub const ADAPTER: &str = "/org/bluez/hci0";

fn device(slot: u8, name: Option<&str>, icon: Option<&str>) -> Device {
    let address = format!("A4:C1:38:5E:{slot:02X}:{:02X}", slot.wrapping_mul(37));
    Device {
        path: format!("{ADAPTER}/dev_{}", address.replace(':', "_")),
        adapter: ADAPTER.to_string(),
        alias: Some(name.map(str::to_string).unwrap_or_else(|| address.replace(':', "-"))),
        name: name.map(str::to_string),
        address,
        address_type: "public".to_string(),
        icon: icon.map(str::to_string),
        ..Device::default()
    }
}

fn paired(mut device: Device, connected: bool, battery: Option<u8>) -> Device {
    device.paired = true;
    device.trusted = true;
    device.connected = connected;
    device.battery = battery;
    device
}

fn nearby(mut device: Device, rssi: Option<i16>) -> Device {
    device.rssi = rssi;
    device
}

/// A laptop with four paired devices and five in range, two of them unnamed.
pub fn snapshot() -> Snapshot {
    Snapshot {
        service: true,
        adapter: Some(Adapter {
            path: ADAPTER.to_string(),
            name: "flux-laptop".to_string(),
            address: "00:1A:7D:DA:71:13".to_string(),
            powered: true,
            discoverable: true,
            discovering: true,
        }),
        devices: vec![
            paired(device(1, Some("WH-1000XM4"), Some("audio-headphones")), true, Some(80)),
            paired(device(2, Some("Pixel 8"), Some("phone")), false, None),
            paired(device(3, Some("MX Keys"), Some("input-keyboard")), true, Some(45)),
            paired(device(4, Some("Xbox Controller"), Some("input-gaming")), false, Some(12)),
            nearby(device(5, Some("Galaxy Buds2"), Some("audio-headset")), Some(-63)),
            nearby(device(6, None, None), Some(-55)),
            nearby(device(7, Some("JBL Flip 6"), Some("audio-card")), Some(-48)),
            nearby(device(8, Some("Living room TV"), None), Some(-80)),
            nearby(device(9, None, None), None),
        ],
        radio: Radio::Free,
    }
}

/// A named preview state: the snapshot, and the prompt the page opens on.
pub struct Scene {
    pub snapshot: Snapshot,
    pub prompt: Option<(String, Prompt)>,
    pub expanded: Option<String>,
    pub pairing: bool,
}

/// The scene called `name`, from `FLUX_BLUETOOTH_PREVIEW`.
pub fn scene(name: &str) -> Scene {
    let mut snapshot = snapshot();
    let first_nearby = |snapshot: &Snapshot| snapshot.nearby(false).first().map(|device| device.path.clone());
    let headphones = snapshot.paired().get(1).map(|device| device.path.clone());
    let target = first_nearby(&snapshot).unwrap_or_default();
    let mut scene = Scene {
        snapshot: Snapshot::default(),
        prompt: None,
        expanded: None,
        pairing: false,
    };
    match name {
        "details" => scene.expanded = headphones,
        "confirm" => scene.prompt = Some((target, Prompt::Confirm(492_017))),
        "pin" => scene.prompt = Some((target, Prompt::RequestPin)),
        "display" => scene.prompt = Some((target, Prompt::DisplayPasskey { passkey: 81_034, entered: 2 })),
        "service" => scene.prompt = Some((target, Prompt::AuthorizeService(Service::Audio))),
        "pairing" => scene.pairing = true,
        "off" | "blocked" | "hardblocked" => {
            if let Some(adapter) = snapshot.adapter.as_mut() {
                adapter.powered = false;
            }
            snapshot.radio = match name {
                "blocked" => Radio::SoftBlocked,
                "hardblocked" => Radio::HardBlocked,
                _ => Radio::Free,
            };
        }
        "noadapter" => snapshot.adapter = None,
        "noservice" => snapshot = Snapshot::default(),
        _ => {}
    }
    scene.snapshot = snapshot;
    scene
}
