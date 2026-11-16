use super::*;
use crate::preview;

fn names(devices: &[&Device]) -> Vec<String> {
    devices.iter().map(|device| device.display_name()).collect()
}

#[test]
fn kind_prefers_icon_then_class_then_appearance() {
    let mut device = Device {
        icon: Some("input-keyboard".into()),
        class: Some(0x240404),
        ..Device::default()
    };
    assert_eq!(device.kind(), Kind::Keyboard);
    device.icon = Some("unknown-thing".into());
    assert_eq!(device.kind(), Kind::Headphones);
    device.class = None;
    device.appearance = Some(0x03c2);
    assert_eq!(device.kind(), Kind::Mouse);
    device.appearance = None;
    assert_eq!(device.kind(), Kind::Other);
}

#[test]
fn class_of_device_decodes_majors_and_minors() {
    assert_eq!(Kind::from_class(0x5a020c), Some(Kind::Phone));
    assert_eq!(Kind::from_class(0x240418), Some(Kind::Headphones));
    assert_eq!(Kind::from_class(0x240414), Some(Kind::Speaker));
    assert_eq!(Kind::from_class(0x002540), Some(Kind::Keyboard));
    assert_eq!(Kind::from_class(0x002580), Some(Kind::Mouse));
    assert_eq!(Kind::from_class(0x002508), Some(Kind::Controller));
    assert_eq!(Kind::from_class(0x00010c), Some(Kind::Computer));
    assert_eq!(Kind::from_class(0x000600), None);
}

#[test]
fn every_kind_has_a_symbolic_icon() {
    let kinds = [
        Kind::Headphones,
        Kind::Speaker,
        Kind::Keyboard,
        Kind::Mouse,
        Kind::Phone,
        Kind::Controller,
        Kind::Computer,
        Kind::Other,
    ];
    assert!(kinds.iter().all(|kind| kind.icon().ends_with("-symbolic")));
}

#[test]
fn name_falls_back_from_alias_to_name_to_address() {
    let mut device = Device {
        address: "AA:BB:CC:DD:EE:FF".into(),
        alias: Some("AA-BB-CC-DD-EE-FF".into()),
        ..Device::default()
    };
    assert_eq!(device.display_name(), "AA:BB:CC:DD:EE:FF");
    assert!(!device.has_name());
    device.name = Some("  WH-1000XM4 ".into());
    assert_eq!(device.display_name(), "WH-1000XM4");
    assert!(device.has_name());
    device.alias = Some("Yann's headphones".into());
    assert_eq!(device.display_name(), "Yann's headphones");
}

#[test]
fn paired_lists_connected_first_then_by_name() {
    let snapshot = preview::snapshot();
    assert_eq!(names(&snapshot.paired()), ["MX Keys", "WH-1000XM4", "Pixel 8", "Xbox Controller"]);
}

#[test]
fn nearby_sorts_by_signal_and_hides_unnamed() {
    let snapshot = preview::snapshot();
    assert_eq!(names(&snapshot.nearby(false)), ["JBL Flip 6", "Galaxy Buds2", "Living room TV"]);
    assert_eq!(snapshot.unnamed_count(), 2);
    let all = snapshot.nearby(true);
    assert_eq!(all.len(), 5);
    assert_eq!(all.last().map(|device| device.rssi), Some(None));
}

#[test]
fn devices_of_other_adapters_are_left_out() {
    let mut snapshot = preview::snapshot();
    let before = snapshot.paired().len();
    if let Some(device) = snapshot.devices.first_mut() {
        device.adapter = "/org/bluez/hci1".into();
    }
    assert_eq!(snapshot.paired().len() + snapshot.nearby(true).len(), before + 5 - 1);
}

#[test]
fn availability_covers_every_empty_state() {
    let mut snapshot = preview::snapshot();
    assert_eq!(snapshot.availability(), Availability::On);
    snapshot.radio = Radio::HardBlocked;
    assert_eq!(snapshot.availability(), Availability::HardBlocked);
    snapshot.radio = Radio::SoftBlocked;
    if let Some(adapter) = snapshot.adapter.as_mut() {
        adapter.powered = false;
    }
    assert_eq!(snapshot.availability(), Availability::SoftBlocked);
    snapshot.radio = Radio::Free;
    assert_eq!(snapshot.availability(), Availability::Off);
    snapshot.adapter = None;
    assert_eq!(snapshot.availability(), Availability::NoAdapter);
    snapshot.service = false;
    assert_eq!(snapshot.availability(), Availability::NoService);
}

#[test]
fn rfkill_hard_block_wins_and_other_radios_are_ignored() {
    let entry = |kind: &str, soft, hard| RfkillEntry {
        kind: kind.into(),
        soft,
        hard,
    };
    assert_eq!(radio(&[entry("wlan", true, true)]), Radio::Free);
    assert_eq!(radio(&[entry("bluetooth", true, false)]), Radio::SoftBlocked);
    assert_eq!(
        radio(&[entry("bluetooth", true, false), entry("bluetooth", false, true)]),
        Radio::HardBlocked
    );
}

#[test]
fn adapter_pick_is_the_lowest_path() {
    let adapter = |path: &str| Adapter {
        path: path.into(),
        ..Adapter::default()
    };
    let picked = pick_adapter(vec![adapter("/org/bluez/hci1"), adapter("/org/bluez/hci0")]);
    assert_eq!(picked.map(|adapter| adapter.path), Some("/org/bluez/hci0".to_string()));
    assert_eq!(pick_adapter(Vec::new()), None);
}

#[test]
fn battery_formats_and_clamps() {
    assert_eq!(battery_label(85, "{n}%"), "85%");
    assert_eq!(battery_label(140, "{n}\u{202f}%"), "100\u{202f}%");
    assert_eq!(battery_icon(85), "battery-level-80-symbolic");
    assert_eq!(battery_icon(4), "battery-level-0-symbolic");
    assert_eq!(battery_icon(255), "battery-level-100-symbolic");
}

#[test]
fn signal_levels() {
    assert_eq!(signal(-40), Signal::Strong);
    assert_eq!(signal(-60), Signal::Strong);
    assert_eq!(signal(-70), Signal::Good);
    assert_eq!(signal(-90), Signal::Weak);
}

#[test]
fn passkeys_read_and_parse() {
    assert_eq!(passkey_text(123456), "123 456");
    assert_eq!(passkey_text(42), "000 042");
    assert_eq!(parse_passkey("012 345"), Some(12345));
    assert_eq!(parse_passkey("1234567"), None);
    assert_eq!(parse_passkey("12a4"), None);
    assert_eq!(parse_passkey(""), None);
    assert!(valid_pin("0000"));
    assert!(!valid_pin(""));
    assert!(!valid_pin("12 34"));
    assert!(!valid_pin("12345678901234567"));
}

#[test]
fn services_from_uuids() {
    assert_eq!(service("0000110B-0000-1000-8000-00805F9B34FB"), Service::Audio);
    assert_eq!(service("0000111e-0000-1000-8000-00805f9b34fb"), Service::Calls);
    assert_eq!(service("00001124-0000-1000-8000-00805f9b34fb"), Service::Input);
    assert_eq!(service("not-a-uuid"), Service::Other);
}

#[test]
fn failures_from_bluez_errors() {
    assert_eq!(failure("org.bluez.Error.AuthenticationFailed", ""), Failure::WrongCode);
    assert_eq!(failure("org.bluez.Error.AuthenticationCanceled", ""), Failure::Canceled);
    assert_eq!(failure("org.bluez.Error.ConnectionAttemptFailed", ""), Failure::Unreachable);
    assert_eq!(failure("org.bluez.Error.Failed", "Page Timeout"), Failure::Unreachable);
    assert_eq!(failure("org.bluez.Error.Failed", "Blocked through rfkill"), Failure::Blocked);
    assert_eq!(failure("org.freedesktop.DBus.Error.NoReply", ""), Failure::Timeout);
    assert_eq!(failure("org.bluez.Error.Failed", "br-connection-unknown"), Failure::Other);
}

#[test]
fn initiated_pairing_confirms_then_connects() {
    let device = "/org/bluez/hci0/dev_1".to_string();
    let state = Pairing::Idle.next(Step::Start(device.clone()));
    assert_eq!(state, Pairing::Working { device: device.clone(), stage: Stage::Pairing });
    let state = state.next(Step::Prompt(device.clone(), Prompt::Confirm(123456)));
    assert!(matches!(state, Pairing::Asking { initiated: true, .. }));
    let state = state.next(Step::Answered);
    assert_eq!(state, Pairing::Working { device: device.clone(), stage: Stage::Pairing });
    let state = state.next(Step::Paired(device.clone()));
    assert_eq!(state, Pairing::Working { device: device.clone(), stage: Stage::Connecting });
    assert_eq!(state.next(Step::Finished(device)), Pairing::Idle);
}

#[test]
fn keyboard_passkey_display_updates_in_place() {
    let device = "/dev/kb".to_string();
    let state = Pairing::Idle
        .next(Step::Start(device.clone()))
        .next(Step::Prompt(device.clone(), Prompt::DisplayPasskey { passkey: 1, entered: 0 }))
        .next(Step::Prompt(device.clone(), Prompt::DisplayPasskey { passkey: 1, entered: 3 }));
    assert_eq!(
        state,
        Pairing::Asking {
            device: device.clone(),
            prompt: Prompt::DisplayPasskey { passkey: 1, entered: 3 },
            initiated: true
        }
    );
    assert!(!Prompt::DisplayPasskey { passkey: 1, entered: 3 }.needs_answer());
    assert_eq!(state.next(Step::Paired(device.clone())), Pairing::Working { device, stage: Stage::Connecting });
}

#[test]
fn failures_show_until_dismissed_and_cancel_is_silent() {
    let device = "/dev/a".to_string();
    let state = Pairing::Idle.next(Step::Start(device.clone()));
    let failed = state.clone().next(Step::Failed(device.clone(), Failure::WrongCode));
    assert_eq!(failed, Pairing::Failed { device: device.clone(), failure: Failure::WrongCode });
    assert_eq!(failed.clone().next(Step::Dismiss), Pairing::Idle);
    assert_eq!(failed.next(Step::Start(device.clone())), state.clone());
    assert_eq!(state.next(Step::Failed(device, Failure::Canceled)), Pairing::Idle);
}

#[test]
fn steps_for_another_device_are_ignored() {
    let state = Pairing::Idle.next(Step::Start("/dev/a".into()));
    assert_eq!(state.clone().next(Step::Start("/dev/b".into())), state);
    assert_eq!(state.clone().next(Step::Paired("/dev/b".into())), state);
    assert_eq!(state.clone().next(Step::Failed("/dev/b".into(), Failure::Other)), state);
    assert_eq!(state.clone().next(Step::Prompt("/dev/b".into(), Prompt::Authorize)), state);
}

#[test]
fn incoming_requests_end_once_answered() {
    let state = Pairing::Idle.next(Step::Prompt("/dev/phone".into(), Prompt::AuthorizeService(Service::Audio)));
    assert!(matches!(state, Pairing::Asking { initiated: false, .. }));
    assert_eq!(state.clone().next(Step::Answered), Pairing::Idle);
    assert_eq!(state.next(Step::Cancelled), Pairing::Idle);
}
