//! Everything that touches the system: niri over its CLI, the user files niri
//! includes, the xkb registry, the time zone and the night light unit. Every
//! function here blocks, so the pages call them through [`spawn`].
//!
//! `FLUX_SETTINGS_PREVIEW=1` swaps the system for sample data and makes every
//! write a no-op, for screenshots and design work.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use crate::model::config::{has_touchpad, input_kdl, outputs_kdl, Layout, NightLight, OutputConfig, State};
use crate::model::night::{env_file, wlsunset_args, zone_coordinates, zone_from_link};
use crate::model::outputs::{format_scale, mode_kdl, parse_outputs, Output};
use crate::model::xkb::{parse_registry, parse_xorg_keyboard, Registry};

/// The user unit that runs `wlsunset` with the arguments in [`night_env`].
pub const NIGHT_UNIT: &str = "flux-night-light.service";

const SAMPLE_OUTPUTS: &str = include_str!("model/fixtures/outputs.json");
const SAMPLE_REGISTRY: &str = include_str!("model/fixtures/evdev.xml");
const SAMPLE_KEYBOARD: &str = include_str!("model/fixtures/00-keyboard.conf");

static STATE_LOCK: Mutex<()> = Mutex::new(());

/// Run `work` on a worker thread and hand its result to `done` on the GTK
/// main thread.
pub fn spawn<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static, done: impl FnOnce(T) + 'static) {
    gtk4::glib::MainContext::default().spawn_local(async move {
        if let Ok(value) = gtk4::gio::spawn_blocking(work).await {
            done(value);
        }
    });
}

pub fn preview() -> bool {
    std::env::var_os("FLUX_SETTINGS_PREVIEW").is_some_and(|value| !value.is_empty())
}

/// The screen a preview opens on, from `FLUX_SETTINGS_PREVIEW`: `keep`
/// shows the revert countdown.
pub fn preview_scene() -> String {
    std::env::var("FLUX_SETTINGS_PREVIEW").unwrap_or_default()
}

fn config_home() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
}

/// `~/.config/flux`, where the state and the generated files live.
pub fn flux_dir() -> Option<PathBuf> {
    config_home().map(|home| home.join("flux"))
}

fn state_file() -> Option<PathBuf> {
    flux_dir().map(|dir| dir.join("display.json"))
}

/// The niri includes: `~/.config/flux/niri/outputs.kdl` and `input.kdl`.
pub fn niri_file(name: &str) -> Option<PathBuf> {
    flux_dir().map(|dir| dir.join("niri").join(name))
}

fn night_env() -> Option<PathBuf> {
    flux_dir().map(|dir| dir.join("night-light.env"))
}

/// Write `contents` to `path` through a temporary file and a rename, so niri
/// never reads half a file.
fn write_atomic(path: &Path, contents: &str) -> Result<(), String> {
    let dir = path.parent().ok_or("no parent directory")?;
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let name = path.file_name().and_then(|name| name.to_str()).ok_or("bad file name")?;
    let temporary = dir.join(format!(".{name}.{}", std::process::id()));
    std::fs::write(&temporary, contents).map_err(|error| error.to_string())?;
    std::fs::rename(&temporary, path).map_err(|error| {
        let _ = std::fs::remove_file(&temporary);
        error.to_string()
    })
}

pub fn load_state() -> State {
    if preview() {
        let mut state = State::default();
        state.night.enabled = true;
        return state;
    }
    state_file().and_then(|path| std::fs::read_to_string(path).ok()).map(|json| State::parse(&json)).unwrap_or_default()
}

/// Change the state file through `change` and rewrite the niri include it
/// feeds. Reads before writing, so the two pages never undo each other.
fn update(change: impl FnOnce(&mut State), include: &str, render: fn(&State) -> String) -> Result<State, String> {
    let _guard = STATE_LOCK.lock().map_err(|_| "state lock poisoned".to_string())?;
    let mut state = load_state();
    change(&mut state);
    if preview() {
        return Ok(state);
    }
    let path = state_file().ok_or("no home directory")?;
    write_atomic(&path, &state.to_json())?;
    let kdl = niri_file(include).ok_or("no home directory")?;
    write_atomic(&kdl, &render(&state))?;
    Ok(state)
}

/// Save output settings and rewrite `outputs.kdl`.
pub fn save_outputs(change: impl FnOnce(&mut State)) -> Result<State, String> {
    update(change, "outputs.kdl", |state| outputs_kdl(&state.outputs))
}

/// Save input settings and rewrite `input.kdl`.
pub fn save_input(change: impl FnOnce(&mut State)) -> Result<State, String> {
    update(change, "input.kdl", |state| input_kdl(&state.input))
}

/// The first `name` on `PATH`, then in the system profile, which a user unit
/// started by NixOS may not have on its `PATH`.
fn program(name: &str) -> PathBuf {
    std::env::var_os("PATH")
        .into_iter()
        .flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
        .chain([PathBuf::from("/run/current-system/sw/bin"), PathBuf::from("/etc/profiles/per-user/bin")])
        .map(|dir| dir.join(name))
        .find(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from(name))
}

fn run(name: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program(name)).args(args).output().map_err(|error| format!("{name}: {error}"))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Err(if stderr.is_empty() { stdout } else { stderr })
}

/// The connected outputs, from `niri msg --json outputs`.
pub fn outputs() -> Result<Vec<Output>, String> {
    if preview() {
        return parse_outputs(SAMPLE_OUTPUTS);
    }
    parse_outputs(&run("niri", &["msg", "--json", "outputs"])?)
}

fn output_msg(connector: &str, args: &[&str]) -> Result<(), String> {
    if preview() {
        return Ok(());
    }
    let mut full = vec!["msg", "output", connector];
    full.extend_from_slice(args);
    run("niri", &full).map(|_| ())
}

/// Apply `config` to the running session only, through `niri msg output`.
/// niri forgets it on the next config reload, which is what makes the revert
/// countdown safe: nothing is written until the person keeps it.
pub fn apply_live(config: &OutputConfig) -> Result<(), String> {
    let connector = config.connector.as_str();
    if config.off {
        return output_msg(connector, &["off"]);
    }
    output_msg(connector, &["on"])?;
    let mode = config.mode.map(|m| mode_kdl(m.width, m.height, m.refresh)).unwrap_or_else(|| "auto".to_string());
    output_msg(connector, &["mode", &mode])?;
    let scale = config.scale.map(format_scale).unwrap_or_else(|| "auto".to_string());
    output_msg(connector, &["scale", &scale])?;
    output_msg(connector, &["transform", config.transform.niri()])
}

fn first_existing(candidates: impl IntoIterator<Item = PathBuf>) -> Option<PathBuf> {
    candidates.into_iter().find(|path| path.is_file())
}

/// The xkb registry, or the sample in preview when the system has none.
pub fn registry() -> Option<Registry> {
    let env = |key: &str| std::env::var_os(key).map(PathBuf::from);
    let path = first_existing(
        env("FLUX_XKB_RULES")
            .into_iter()
            .chain(env("XKB_CONFIG_ROOT").map(|root| root.join("rules/evdev.xml")))
            .chain(
                [
                    "/run/current-system/sw/share/xkeyboard-config-2/rules/evdev.xml",
                    "/run/current-system/sw/share/X11/xkb/rules/evdev.xml",
                    "/etc/X11/xkb/rules/evdev.xml",
                    "/usr/share/xkeyboard-config-2/rules/evdev.xml",
                    "/usr/share/X11/xkb/rules/evdev.xml",
                ]
                .map(PathBuf::from),
            ),
    );
    match path.and_then(|path| std::fs::read_to_string(path).ok()) {
        Some(xml) => parse_registry(&xml).ok(),
        None if preview() => parse_registry(SAMPLE_REGISTRY).ok(),
        None => None,
    }
}

/// The layouts and switch option the system sets, which niri reads from
/// locale1 while the user has not chosen their own.
pub fn system_keyboard() -> (Vec<Layout>, String) {
    let text = if preview() {
        SAMPLE_KEYBOARD.to_string()
    } else {
        std::fs::read_to_string("/etc/X11/xorg.conf.d/00-keyboard.conf").unwrap_or_default()
    };
    let (layouts, options) = parse_xorg_keyboard(&text);
    if layouts.is_empty() {
        return (vec![Layout { layout: "us".to_string(), variant: String::new() }], options);
    }
    (layouts, options)
}

/// Latitude and longitude of the system time zone, for sunset and sunrise.
pub fn coordinates() -> Option<(f64, f64)> {
    if preview() {
        return Some((48.87, 2.33));
    }
    let target = std::fs::read_link("/etc/localtime").ok()?;
    let zone = zone_from_link(target.to_str()?)?;
    let dirs = std::env::var_os("TZDIR")
        .map(PathBuf::from)
        .into_iter()
        .chain(["/etc/zoneinfo", "/usr/share/zoneinfo"].map(PathBuf::from));
    dirs.flat_map(|dir| [dir.join("zone1970.tab"), dir.join("zone.tab")])
        .filter_map(|path| std::fs::read_to_string(path).ok())
        .find_map(|tab| zone_coordinates(&tab, &zone))
}

/// Whether the machine has a touchpad, so the page can skip its section.
pub fn touchpad() -> bool {
    preview() || std::fs::read_to_string("/proc/bus/input/devices").is_ok_and(|text| has_touchpad(&text))
}

/// Save night light and start, restart or stop its unit to match.
pub fn apply_night(night: NightLight) -> Result<(), String> {
    let _guard = STATE_LOCK.lock().map_err(|_| "state lock poisoned".to_string())?;
    if preview() {
        return Ok(());
    }
    let mut state = load_state();
    state.night = night;
    write_atomic(&state_file().ok_or("no home directory")?, &state.to_json())?;
    let env = night_env().ok_or("no home directory")?;
    match wlsunset_args(&night, coordinates()) {
        Some(args) => {
            write_atomic(&env, &env_file(&args))?;
            run("systemctl", &["--user", "restart", NIGHT_UNIT]).map(|_| ())
        }
        None => {
            let _ = std::fs::remove_file(&env);
            run("systemctl", &["--user", "stop", NIGHT_UNIT]).map(|_| ())
        }
    }
}
