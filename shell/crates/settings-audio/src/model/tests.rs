use super::*;

fn laptop() -> Snapshot {
    serde_json::from_str(include_str!("../../fixtures/laptop.json")).unwrap_or_default()
}

fn virtual_rig() -> Snapshot {
    serde_json::from_str(include_str!("../../fixtures/virtual.json")).unwrap_or_default()
}

fn titles(snapshot: &Snapshot, direction: Direction) -> Vec<(String, Option<String>, Kind)> {
    snapshot
        .devices(direction)
        .iter()
        .map(|device| {
            let label = device.label(direction);
            (label.title, label.detail, label.kind)
        })
        .collect()
}

#[test]
fn fixtures_parse() {
    assert_eq!(laptop().sinks.len(), 4);
    assert_eq!(virtual_rig().sinks.len(), 2);
}

#[test]
fn outputs_read_as_people_say_them() {
    assert_eq!(
        titles(&laptop(), Direction::Output),
        vec![
            (
                "Speakers".to_string(),
                Some("Built-in Audio".to_string()),
                Kind::Speakers
            ),
            (
                "HDMI / DisplayPort".to_string(),
                Some("Built-in Audio".to_string()),
                Kind::Hdmi
            ),
            ("WH-1000XM4".to_string(), None, Kind::Bluetooth),
            ("Scarlett Solo USB".to_string(), None, Kind::Usb),
        ]
    );
}

#[test]
fn inputs_skip_monitors() {
    assert_eq!(
        titles(&laptop(), Direction::Input),
        vec![
            (
                "Internal Microphone".to_string(),
                Some("Built-in Audio".to_string()),
                Kind::Microphone
            ),
            ("Scarlett Solo USB".to_string(), None, Kind::Usb),
        ]
    );
}

#[test]
fn headphones_plugged_in_rename_the_internal_card() {
    let mut snapshot = laptop();
    let sink = &mut snapshot.sinks[0];
    sink.ports[1].available = Availability::Yes;
    sink.active_port = Some("analog-output-headphones".to_string());
    let label = sink.label(Direction::Output);
    assert_eq!(label.title, "Headphones");
    assert_eq!(label.kind, Kind::Headphones);
}

#[test]
fn virtual_and_unknown_devices_fall_back_to_their_description() {
    assert_eq!(
        titles(&virtual_rig(), Direction::Output),
        vec![
            ("Null Output".to_string(), None, Kind::Virtual),
            ("Loopback".to_string(), None, Kind::Other),
        ]
    );
    assert!(virtual_rig().devices(Direction::Input).is_empty());
}

#[test]
fn bare_names_survive_an_empty_description() {
    let device = Device {
        name: "alsa_output.weird".to_string(),
        ..Device::default()
    };
    assert_eq!(device.label(Direction::Output).title, "alsa_output.weird");
}

#[test]
fn clean_description_strips_layout_and_monitor() {
    assert_eq!(
        clean_description("Built-in Audio Analog Stereo"),
        "Built-in Audio"
    );
    assert_eq!(clean_description("Monitor of Null Output"), "Null Output");
    assert_eq!(
        clean_description("HD Pro Webcam C920 Analog Mono"),
        "HD Pro Webcam C920"
    );
    assert_eq!(clean_description("WH-1000XM4"), "WH-1000XM4");
}

#[test]
fn default_device_falls_back_to_the_first() {
    assert_eq!(
        laptop().default_device(Direction::Output).map(|d| d.index),
        Some(48)
    );
    assert_eq!(
        virtual_rig()
            .default_device(Direction::Output)
            .map(|d| d.index),
        Some(0)
    );
    assert_eq!(virtual_rig().default_device(Direction::Input), None);
}

#[test]
fn unplugged_ports_are_hidden_unless_active() {
    let snapshot = laptop();
    let names: Vec<&str> = snapshot.sinks[0]
        .port_choices()
        .iter()
        .map(|p| p.name.as_str())
        .collect();
    assert_eq!(names, vec!["analog-output-speaker"]);
    let mut sink = snapshot.sinks[0].clone();
    sink.active_port = Some("analog-output-headphones".to_string());
    assert_eq!(sink.port_choices().len(), 2);
}

#[test]
fn profiles_hide_off_and_unavailable_and_sort_by_priority() {
    let snapshot = laptop();
    let card = snapshot
        .card_of(&snapshot.sinks[2])
        .map(|card| card.profile_choices());
    let names: Vec<&str> = card
        .unwrap_or_default()
        .iter()
        .map(|p| p.name.as_str())
        .collect();
    assert_eq!(names, vec!["a2dp-sink", "headset-head-unit"]);
    let internal = &snapshot.cards[0];
    assert_eq!(internal.profile_choices().len(), 2);
    let mut off = internal.clone();
    off.active_profile = Some("off".to_string());
    assert_eq!(
        off.profile_choices().last().map(|p| p.name.as_str()),
        Some("off")
    );
}

#[test]
fn streams_group_by_app_without_our_own() {
    let groups = group_streams(&laptop().streams);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].name, "Firefox");
    assert_eq!(groups[0].icon, "firefox");
    assert_eq!(groups[0].streams, vec![120, 121]);
    assert_eq!(groups[0].volume, NORM);
    assert!(!groups[0].muted);
    assert_eq!(groups[1].name, "Spotify");
    assert_eq!(groups[1].icon, "spotify");
    assert!(groups[1].muted);
}

#[test]
fn nameless_streams_stay_apart() {
    let groups = group_streams(&virtual_rig().streams);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].name, "Stream-3");
    assert_eq!(groups[1].name, "Playback");
    assert_eq!(groups[1].icon, "application-x-executable-symbolic");
}

#[test]
fn percent_and_raw_round_trip() {
    assert_eq!(percent(NORM), 100);
    assert_eq!(percent(0), 0);
    assert_eq!(percent(45875), 70);
    for value in 0..=BOOST_PERCENT {
        assert_eq!(percent(raw(value)), value);
    }
}

#[test]
fn step_clamps_to_the_cap() {
    assert_eq!(percent(step(raw(95), 10, false)), 100);
    assert_eq!(percent(step(raw(95), 10, true)), 105);
    assert_eq!(percent(step(raw(148), 5, true)), 150);
    assert_eq!(percent(step(raw(3), -5, false)), 0);
    assert_eq!(percent(step(raw(130), 5, false)), 130);
    assert_eq!(percent(step(raw(130), -5, false)), 125);
}

#[test]
fn volume_is_cubic() {
    assert!((amplitude(50) - 0.125).abs() < 1e-9);
    assert!((amplitude(100) - 1.0).abs() < 1e-9);
    assert!((meter(0.125) - 0.5).abs() < 1e-6);
    assert_eq!(meter(4.0), 1.0);
    assert_eq!(meter(-1.0), 0.0);
}

#[test]
fn meters_rise_fast_and_fall_slow() {
    assert_eq!(smooth(0.2, 0.9), 0.9);
    let fallen = smooth(0.9, 0.0);
    assert!(fallen < 0.9 && fallen > 0.5);
    assert!(smooth(0.001, 0.0) >= 0.0);
}

#[test]
fn icons_follow_the_level() {
    assert_eq!(volume_icon(80, true), "audio-volume-muted-symbolic");
    assert_eq!(volume_icon(0, false), "audio-volume-muted-symbolic");
    assert_eq!(volume_icon(20, false), "audio-volume-low-symbolic");
    assert_eq!(volume_icon(50, false), "audio-volume-medium-symbolic");
    assert_eq!(volume_icon(120, false), "audio-volume-high-symbolic");
}

#[test]
fn options_tolerate_bad_files() {
    assert_eq!(Options::parse("not json"), Options::default());
    assert_eq!(Options::parse("{}"), Options::default());
    let on = Options { allow_boost: true };
    assert_eq!(Options::parse(&on.to_json()), on);
}

#[test]
fn test_tone_plays_left_then_right() {
    let samples = test_tone(1000);
    let frames: Vec<(f32, f32)> = samples.chunks(2).map(|pair| (pair[0], pair[1])).collect();
    let half = frames.len() / 2;
    assert!(frames[..half].iter().all(|(_, right)| *right == 0.0));
    assert!(frames[half..].iter().all(|(left, _)| *left == 0.0));
    assert!(frames[..half].iter().any(|(left, _)| left.abs() > 0.1));
    assert!(frames[half..].iter().any(|(_, right)| right.abs() > 0.1));
    assert_eq!(frames[0], (0.0, 0.0));
    assert!(samples.iter().all(|sample| sample.abs() <= 0.3));
}
