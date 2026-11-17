use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use super::config::{
    input_kdl, outputs_kdl, quote, AccelProfile, InputConfig, Layout, ModeChoice, NightLight, OutputConfig, Schedule,
    ScrollMethod, State,
};
use super::geometry::{bounds, fit, normalize, place, Rect};
use super::night::{env_file, parse_iso6709, parse_time, wlsunset_args, zone_coordinates, zone_from_link};
use super::outputs::{
    format_mode, format_refresh, format_scale, mode_kdl, parse_outputs, refresh_rates, resolutions,
    scale_choices, valid_scale, Transform,
};
use super::xkb::{parse_registry, parse_xorg_keyboard, switch_option, zip_layouts};

const OUTPUTS: &str = include_str!("fixtures/outputs.json");
const EVDEV: &str = include_str!("fixtures/evdev.xml");
const XORG: &str = include_str!("fixtures/00-keyboard.conf");

#[test]
fn outputs_parse_with_the_panel_first() {
    let outputs = parse_outputs(OUTPUTS).expect("fixture parses");
    let names: Vec<&str> = outputs.iter().map(|o| o.name.as_str()).collect();
    assert_eq!(names, ["eDP-1", "DP-1", "HDMI-A-1"]);
    assert!(outputs[0].internal());
    assert!(!outputs[2].enabled());
    assert_eq!(outputs[1].logical.map(|l| l.scale), Some(2.0));
}

#[test]
fn outputs_reject_garbage() {
    assert!(parse_outputs("not json").is_err());
    assert_eq!(parse_outputs("{}").map(|list| list.len()), Ok(0));
}

#[test]
fn identity_prefers_the_serial() {
    let outputs = parse_outputs(OUTPUTS).expect("fixture parses");
    assert_eq!(outputs[0].identity(), "eDP-1");
    assert_eq!(outputs[1].identity(), "Dell Inc. DELL U2723QE 7XKF3H3");
    assert_eq!(outputs[2].display_name(), "HDMI-A-1");
    assert_eq!(outputs[1].display_name(), "Dell Inc. DELL U2723QE");
    assert_eq!(outputs[1].diagonal_inches(), Some(27.2));
    assert_eq!(outputs[2].diagonal_inches(), None);
}

#[test]
fn transforms_read_niri_json_spelling() {
    let parsed: Vec<Transform> =
        serde_json::from_str(r#"["Normal","90","180","270","Flipped","Flipped90","Flipped180","Flipped270"]"#)
            .expect("transforms parse");
    assert_eq!(parsed, Transform::ALL);
    let names: Vec<&str> = Transform::ALL.iter().map(|t| t.niri()).collect();
    assert_eq!(names, ["normal", "90", "180", "270", "flipped", "flipped-90", "flipped-180", "flipped-270"]);
}

#[test]
fn resolutions_are_unique_and_largest_first() {
    let outputs = parse_outputs(OUTPUTS).expect("fixture parses");
    let list = resolutions(&outputs[1].modes);
    let sizes: Vec<(u16, u16, bool)> = list.iter().map(|r| (r.width, r.height, r.preferred)).collect();
    assert_eq!(sizes, [(3840, 2160, true), (2560, 1440, false), (1920, 1080, false)]);
    let rates: Vec<u32> = refresh_rates(&outputs[1].modes, 2560, 1440).iter().map(|m| m.refresh_rate).collect();
    assert_eq!(rates, [143998, 59951]);
    assert_eq!(outputs[0].preferred().map(|m| m.refresh_rate), Some(60002));
}

#[test]
fn refresh_rates_read_like_people_say_them() {
    assert_eq!(format_refresh(143998), "144");
    assert_eq!(format_refresh(60002), "60");
    assert_eq!(format_refresh(59940), "59.94");
    assert_eq!(format_refresh(29970), "29.97");
    assert_eq!(format_refresh(74500), "74.5");
    let outputs = parse_outputs(OUTPUTS).expect("fixture parses");
    let mode = outputs[1].modes[2];
    assert_eq!(format_mode(&mode, "at"), "2560 x 1440 at 144 Hz");
    assert_eq!(format_mode(&mode, "à"), "2560 x 1440 à 144 Hz");
}

#[test]
fn mode_kdl_keeps_exact_millihertz() {
    assert_eq!(mode_kdl(2560, 1440, 143998), "2560x1440@143.998");
    assert_eq!(mode_kdl(1920, 1080, 60000), "1920x1080@60.000");
    assert_eq!(mode_kdl(1280, 800, 59040), "1280x800@59.040");
}

#[test]
fn scales_are_validated_and_offered() {
    assert!(valid_scale(1.25));
    assert!(!valid_scale(0.0));
    assert!(!valid_scale(11.0));
    assert!(!valid_scale(f64::NAN));
    assert_eq!(format_scale(1.25), "1.25");
    assert_eq!(format_scale(2.0), "2");
    assert_eq!(format_scale(1.5), "1.5");
    assert_eq!(scale_choices(Some(1.6), 3840), [1.0, 1.25, 1.5, 1.6, 1.75, 2.0, 2.5, 3.0]);
    assert_eq!(scale_choices(Some(1.5), 1366), [1.0, 1.25, 1.5, 1.75, 2.0]);
    assert_eq!(scale_choices(Some(42.0), 1024), [1.0, 1.25, 1.5]);
    assert_eq!(scale_choices(None, 800), [1.0, 1.25]);
}

#[test]
fn a_drop_lands_flush_and_aligned() {
    let laptop = Rect::new(0, 0, 1707, 1067);
    let monitor = Rect::new(1650, 40, 1920, 1080);
    assert_eq!(place(&monitor, &[laptop], 64), (1707, 0));
    let above = Rect::new(200, -1000, 1920, 1080);
    assert_eq!(place(&above, &[laptop], 64), (200, -1080));
    let far = Rect::new(9000, 5000, 1920, 1080);
    let (x, y) = place(&far, &[laptop], 64);
    let landed = far.at(x, y);
    assert!(landed.touches(&laptop) && !landed.overlaps(&laptop));
}

#[test]
fn a_drop_never_overlaps_a_third_output() {
    let left = Rect::new(0, 0, 1920, 1080);
    let right = Rect::new(1920, 0, 1920, 1080);
    let moving = Rect::new(1800, 900, 1280, 800);
    let (x, y) = place(&moving, &[left, right], 64);
    let landed = moving.at(x, y);
    assert!(!landed.overlaps(&left) && !landed.overlaps(&right));
    assert!(landed.touches(&left) || landed.touches(&right));
    assert_eq!(place(&moving, &[], 64), (0, 0));
}

#[test]
fn geometry_normalizes_and_fits() {
    let mut rects = [Rect::new(-1920, -100, 1920, 1080), Rect::new(0, 0, 1707, 1067)];
    normalize(&mut rects);
    assert_eq!((rects[0].x, rects[0].y, rects[1].x, rects[1].y), (0, 0, 1920, 100));
    let whole = bounds(&rects).expect("two rects");
    assert_eq!((whole.width, whole.height), (3627, 1167));
    let (factor, ox, oy) = fit(&whole, 400.0, 200.0, 20.0);
    assert!((factor - 360.0 / 3627.0).abs() < 1e-9);
    assert!((ox - 20.0).abs() < 1e-9);
    assert!(oy > 20.0);
    assert!(bounds(&[]).is_none());
}

#[test]
fn touching_needs_a_shared_stretch() {
    let a = Rect::new(0, 0, 100, 100);
    assert!(a.touches(&Rect::new(100, 50, 100, 100)));
    assert!(!a.touches(&Rect::new(100, 100, 100, 100)));
    assert!(!a.overlaps(&Rect::new(100, 0, 100, 100)));
    assert!(a.overlaps(&Rect::new(99, 0, 100, 100)));
}

fn sample_outputs() -> BTreeMap<String, OutputConfig> {
    let outputs = parse_outputs(OUTPUTS).expect("fixture parses");
    let mut map: BTreeMap<String, OutputConfig> =
        outputs.iter().map(|output| (output.identity(), OutputConfig::from_live(output))).collect();
    if let Some(dell) = map.get_mut("Dell Inc. DELL U2723QE 7XKF3H3") {
        dell.mode = Some(ModeChoice { width: 2560, height: 1440, refresh: 143998 });
        dell.scale = Some(1.25);
        dell.transform = Transform::R90;
        dell.vrr = true;
    }
    map
}

#[test]
fn outputs_kdl_writes_every_field() {
    let kdl = outputs_kdl(&sample_outputs());
    assert!(kdl.contains(
        "output \"Dell Inc. DELL U2723QE 7XKF3H3\" {\n    mode \"2560x1440@143.998\"\n    scale 1.25\n    transform \"90\"\n    position x=1707 y=0\n    variable-refresh-rate\n}"
    ));
    assert!(kdl.contains("output \"eDP-1\" {\n    mode \"2560x1600@60.002\"\n    scale 1.5\n"));
    assert!(kdl.contains("output \"HDMI-A-1\" {\n    off\n    transform \"normal\"\n}"));
}

fn sample_input() -> InputConfig {
    InputConfig {
        layouts: vec![
            Layout { layout: "fr".into(), variant: "azerty".into() },
            Layout { layout: "us".into(), variant: String::new() },
        ],
        switch: "grp:alt_shift_toggle".into(),
        repeat_delay: 300,
        repeat_rate: 40,
        numlock: true,
        tap: false,
        touchpad_natural: true,
        dwt: true,
        touchpad_speed: 0.3,
        scroll: ScrollMethod::Edge,
        mouse_speed: -2.0,
        mouse_profile: AccelProfile::Flat,
        mouse_natural: true,
    }
}

#[test]
fn input_kdl_writes_keyboard_and_pointers() {
    let kdl = input_kdl(&sample_input());
    assert!(kdl.contains(
        "        xkb {\n            layout \"fr,us\"\n            variant \"azerty,\"\n            options \"grp:alt_shift_toggle\"\n        }\n"
    ));
    assert!(kdl.contains("        repeat-delay 300\n        repeat-rate 40\n        numlock\n"));
    assert!(kdl.contains("    touchpad {\n        natural-scroll\n        dwt\n        accel-speed 0.30\n        scroll-method \"edge\"\n    }"));
    assert!(kdl.contains("    mouse {\n        natural-scroll\n        accel-speed -1.00\n        accel-profile \"flat\"\n    }"));
}

#[test]
fn input_kdl_leaves_the_system_layout_alone_by_default() {
    let kdl = input_kdl(&InputConfig::default());
    assert!(!kdl.contains("xkb"));
    assert!(kdl.contains("numlock false"));
    assert!(kdl.contains("    touchpad {\n        tap\n        natural-scroll\n        accel-speed 0.00\n"));
    let single = InputConfig { layouts: vec![Layout { layout: "de".into(), variant: String::new() }], switch: "grp:caps_toggle".into(), ..InputConfig::default() };
    let kdl = input_kdl(&single);
    assert!(kdl.contains("layout \"de\"\n        }"));
    assert!(!kdl.contains("options"));
}

#[test]
fn kdl_strings_are_escaped() {
    assert_eq!(quote(r#"a "b" \c"#), r#""a \"b\" \\c""#);
    assert_eq!(quote("x\ny\u{7}"), "\"x\\ny\"");
}

#[test]
fn state_round_trips_and_survives_garbage() {
    let state = State {
        outputs: sample_outputs(),
        input: sample_input(),
        night: NightLight { enabled: true, temperature: 3500, schedule: Schedule::Manual { from: (21, 30), to: (6, 45) } },
    };
    assert_eq!(State::parse(&state.to_json()), state);
    assert_eq!(State::parse("{oops"), State::default());
    assert_eq!(State::parse("{\"night\":{\"enabled\":true}}").night.temperature, 4000);
}

#[test]
fn registry_lists_layouts_variants_and_switches() {
    let registry = parse_registry(EVDEV).expect("fixture parses");
    let names: Vec<&str> = registry.layouts.iter().map(|l| l.item.name.as_str()).collect();
    assert_eq!(names, ["us", "fr", "de"]);
    let french = registry.layout("fr").expect("french");
    assert_eq!(french.item.description, "French");
    assert!(french.variants.iter().any(|v| v.name == "azerty"));
    let switches: Vec<&str> = registry.switches.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(switches[0], "grp:alt_shift_toggle");
    assert!(switches.contains(&"grp:win_space_toggle"));
    assert!(parse_registry("<nope").is_err());
}

#[test]
fn the_real_registry_parses_when_present() {
    let Some(path) = ["/run/current-system/sw/share/xkeyboard-config-2/rules/evdev.xml", "/usr/share/X11/xkb/rules/evdev.xml"]
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
    else {
        return;
    };
    let xml = std::fs::read_to_string(path).unwrap_or_default();
    let registry = parse_registry(&xml).expect("system registry parses");
    assert!(registry.layouts.len() > 50);
}

#[test]
fn xorg_keyboard_gives_the_system_layouts() {
    let (layouts, options) = parse_xorg_keyboard(XORG);
    assert_eq!(layouts, zip_layouts("fr,us", "azerty,"));
    assert_eq!(layouts[1].variant, "");
    assert_eq!(switch_option(&options), "grp:alt_shift_toggle");
    assert_eq!(parse_xorg_keyboard("").0, Vec::<Layout>::new());
    assert_eq!(zip_layouts("us,de", "").len(), 2);
}

#[test]
fn zone_coordinates_come_from_the_tab() {
    assert_eq!(parse_iso6709("+4852+00220").map(|(a, b)| ((a * 100.0).round(), (b * 100.0).round())), Some((4887.0, 233.0)));
    let (lat, lon) = parse_iso6709("-332508+1513000").expect("seconds form");
    assert!((lat + 33.4189).abs() < 1e-3 && (lon - 151.5).abs() < 1e-3);
    assert_eq!(parse_iso6709("4852+00220"), None);
    assert_eq!(parse_iso6709("+48+002"), None);
    let tab = "# comment\nFR,MC\t+4852+00220\tEurope/Paris\nAU\t-3352+15113\tAustralia/Sydney\tNew South Wales\n";
    assert!(zone_coordinates(tab, "Australia/Sydney").is_some_and(|(lat, _)| lat < 0.0));
    assert!(zone_coordinates(tab, "Mars/Olympus").is_none());
    assert_eq!(zone_from_link("/etc/zoneinfo/Europe/Paris").as_deref(), Some("Europe/Paris"));
    assert_eq!(zone_from_link("../usr/share/zoneinfo/UTC").as_deref(), Some("UTC"));
    assert_eq!(zone_from_link("/etc/localtime"), None);
}

#[test]
fn night_light_builds_wlsunset_arguments() {
    let off = NightLight::default();
    assert_eq!(wlsunset_args(&off, Some((48.87, 2.33))), None);
    let sun = NightLight { enabled: true, temperature: 1000, schedule: Schedule::Sun };
    assert_eq!(
        wlsunset_args(&sun, Some((48.866, 2.333))).expect("on"),
        ["-t", "2500", "-T", "6500", "-l", "48.87", "-L", "2.33"]
    );
    assert_eq!(wlsunset_args(&sun, None).expect("on"), ["-t", "2500", "-T", "6500", "-S", "07:00", "-s", "20:00"]);
    let manual = NightLight { enabled: true, temperature: 3600, schedule: Schedule::Manual { from: (22, 5), to: (6, 0) } };
    let args = wlsunset_args(&manual, Some((1.0, 1.0))).expect("on");
    assert_eq!(args, ["-t", "3600", "-T", "6500", "-S", "06:00", "-s", "22:05"]);
    assert_eq!(env_file(&args), "FLUX_NIGHT_LIGHT_ARGS=-t 3600 -T 6500 -S 06:00 -s 22:05\n");
    assert_eq!(parse_time("7:05"), Some((7, 5)));
    assert_eq!(parse_time("24:00"), None);
    assert_eq!(parse_time("nope"), None);
}

#[test]
fn touchpads_are_found_by_name() {
    let devices = "I: Bus=0018\nN: Name=\"ELAN0676:00 04F3:3195 Touchpad\"\nP: Phys=i2c\n\nN: Name=\"AT Translated Set 2 keyboard\"\n";
    assert!(super::config::has_touchpad(devices));
    assert!(!super::config::has_touchpad("N: Name=\"Logitech USB Receiver\"\n"));
}

fn niri() -> Option<PathBuf> {
    std::env::var_os("PATH")?
        .to_str()?
        .split(':')
        .map(|dir| PathBuf::from(dir).join("niri"))
        .find(|path| path.is_file())
}

fn validate(niri: &PathBuf, dir: &std::path::Path, name: &str, text: &str) {
    let path = dir.join(name);
    std::fs::write(&path, text).expect("write sample");
    let out = Command::new(niri).arg("validate").arg("-c").arg(&path).output().expect("run niri");
    assert!(out.status.success(), "{name} rejected by niri:\n{text}\n{}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn generated_kdl_passes_niri_validate() {
    let Some(niri) = niri() else {
        eprintln!("niri not on PATH, skipping validation");
        return;
    };
    let dir = std::env::temp_dir().join(format!("flux-display-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    validate(&niri, &dir, "outputs.kdl", &outputs_kdl(&sample_outputs()));
    validate(&niri, &dir, "input.kdl", &input_kdl(&sample_input()));
    validate(&niri, &dir, "default-input.kdl", &input_kdl(&InputConfig::default()));
    let system = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../modules/niri.kdl"))
        .unwrap_or_default();
    let main = format!(
        "{system}\ninclude optional=true \"input.kdl\"\ninclude optional=true \"outputs.kdl\"\ninclude optional=true \"absent.kdl\"\n"
    );
    validate(&niri, &dir, "config.kdl", &main);
    let _ = std::fs::remove_dir_all(&dir);
}
