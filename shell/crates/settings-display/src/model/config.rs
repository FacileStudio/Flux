//! What the person chose, kept in `~/.config/flux/display.json`, and the niri
//! KDL generated from it into `~/.config/flux/niri/{outputs,input}.kdl`, which
//! `/etc/niri/config.kdl` includes and niri reloads on every write.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use super::outputs::{format_scale, mode_kdl, valid_scale, Output, Transform};

/// The settings of one output, keyed by its niri identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputConfig {
    pub connector: String,
    #[serde(default)]
    pub off: bool,
    #[serde(default)]
    pub mode: Option<ModeChoice>,
    #[serde(default)]
    pub scale: Option<f64>,
    #[serde(default)]
    pub transform: Transform,
    #[serde(default)]
    pub position: Option<(i32, i32)>,
    #[serde(default)]
    pub vrr: bool,
}

/// A mode by resolution and exact refresh rate in millihertz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeChoice {
    pub width: u16,
    pub height: u16,
    pub refresh: u32,
}

impl OutputConfig {
    /// What the output runs now, so a change to one field keeps the others.
    pub fn from_live(output: &Output) -> OutputConfig {
        let logical = output.logical;
        OutputConfig {
            connector: output.name.clone(),
            off: logical.is_none(),
            mode: output.mode().map(|mode| ModeChoice {
                width: mode.width,
                height: mode.height,
                refresh: mode.refresh_rate,
            }),
            scale: logical.map(|l| l.scale),
            transform: logical.map(|l| l.transform).unwrap_or_default(),
            position: logical.map(|l| (l.x, l.y)),
            vrr: output.vrr_enabled,
        }
    }
}

/// Pointer acceleration, as libinput names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AccelProfile {
    #[default]
    Adaptive,
    Flat,
}

impl AccelProfile {
    pub fn niri(self) -> &'static str {
        match self {
            AccelProfile::Adaptive => "adaptive",
            AccelProfile::Flat => "flat",
        }
    }
}

/// How a touchpad scrolls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScrollMethod {
    #[default]
    TwoFinger,
    Edge,
    NoScroll,
}

impl ScrollMethod {
    pub const ALL: [ScrollMethod; 3] = [ScrollMethod::TwoFinger, ScrollMethod::Edge, ScrollMethod::NoScroll];

    pub fn niri(self) -> &'static str {
        match self {
            ScrollMethod::TwoFinger => "two-finger",
            ScrollMethod::Edge => "edge",
            ScrollMethod::NoScroll => "no-scroll",
        }
    }
}

/// One keyboard layout and its variant, empty for the default.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Layout {
    pub layout: String,
    #[serde(default)]
    pub variant: String,
}

/// Keyboard, touchpad and mouse settings. The defaults match niri's own and
/// the touchpad lines of `/etc/niri/config.kdl`, because a pointing-device
/// section in an include replaces the system one whole.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    pub layouts: Vec<Layout>,
    pub switch: String,
    pub repeat_delay: u32,
    pub repeat_rate: u32,
    pub numlock: bool,
    pub tap: bool,
    pub touchpad_natural: bool,
    pub dwt: bool,
    pub touchpad_speed: f64,
    pub scroll: ScrollMethod,
    pub mouse_speed: f64,
    pub mouse_profile: AccelProfile,
    pub mouse_natural: bool,
}

impl Default for InputConfig {
    fn default() -> InputConfig {
        InputConfig {
            layouts: Vec::new(),
            switch: String::new(),
            repeat_delay: 600,
            repeat_rate: 25,
            numlock: false,
            tap: true,
            touchpad_natural: true,
            dwt: false,
            touchpad_speed: 0.0,
            scroll: ScrollMethod::TwoFinger,
            mouse_speed: 0.0,
            mouse_profile: AccelProfile::Adaptive,
            mouse_natural: false,
        }
    }
}

/// When night light is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Schedule {
    Sun,
    Manual { from: (u8, u8), to: (u8, u8) },
}

/// Night light: warm colours from evening to morning.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NightLight {
    pub enabled: bool,
    pub temperature: u32,
    pub schedule: Schedule,
}

impl Default for NightLight {
    fn default() -> NightLight {
        NightLight { enabled: false, temperature: 4000, schedule: Schedule::Sun }
    }
}

/// Everything the two pages persist.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub outputs: BTreeMap<String, OutputConfig>,
    pub input: InputConfig,
    pub night: NightLight,
}

impl State {
    /// Read the state file, falling back to defaults on anything unreadable.
    pub fn parse(json: &str) -> State {
        serde_json::from_str(json).unwrap_or_default()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Whether `/proc/bus/input/devices` lists a touchpad.
pub fn has_touchpad(devices: &str) -> bool {
    devices
        .lines()
        .filter_map(|line| line.strip_prefix("N: Name="))
        .map(str::to_ascii_lowercase)
        .any(|name| name.contains("touchpad") || name.contains("trackpad") || name.contains("glidepoint"))
}

/// A KDL string literal.
pub fn quote(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

const HEADER: &str = "// Written by Flux Settings. Changes here are replaced on the next save.\n";

/// The `output` sections for every saved output.
pub fn outputs_kdl(outputs: &BTreeMap<String, OutputConfig>) -> String {
    let mut out = String::from(HEADER);
    for (identity, config) in outputs {
        let _ = writeln!(out, "\noutput {} {{", quote(identity));
        if config.off {
            out.push_str("    off\n");
        }
        if let Some(mode) = config.mode {
            let _ = writeln!(out, "    mode \"{}\"", mode_kdl(mode.width, mode.height, mode.refresh));
        }
        if let Some(scale) = config.scale.filter(|scale| valid_scale(*scale)) {
            let _ = writeln!(out, "    scale {}", format_scale(scale));
        }
        let _ = writeln!(out, "    transform \"{}\"", config.transform.niri());
        if let Some((x, y)) = config.position {
            let _ = writeln!(out, "    position x={x} y={y}");
        }
        if config.vrr {
            out.push_str("    variable-refresh-rate\n");
        }
        out.push_str("}\n");
    }
    out
}

fn speed(value: f64) -> String {
    format!("{:.2}", value.clamp(-1.0, 1.0))
}

fn flag(out: &mut String, name: &str, on: bool) {
    if on {
        let _ = writeln!(out, "        {name}");
    }
}

fn keyboard_kdl(out: &mut String, input: &InputConfig) {
    out.push_str("    keyboard {\n");
    let layouts: Vec<&Layout> = input.layouts.iter().filter(|l| !l.layout.trim().is_empty()).collect();
    if !layouts.is_empty() {
        let names: Vec<&str> = layouts.iter().map(|l| l.layout.as_str()).collect();
        let variants: Vec<&str> = layouts.iter().map(|l| l.variant.as_str()).collect();
        out.push_str("        xkb {\n");
        let _ = writeln!(out, "            layout {}", quote(&names.join(",")));
        if variants.iter().any(|variant| !variant.is_empty()) {
            let _ = writeln!(out, "            variant {}", quote(&variants.join(",")));
        }
        if layouts.len() > 1 && !input.switch.is_empty() {
            let _ = writeln!(out, "            options {}", quote(&input.switch));
        }
        out.push_str("        }\n");
    }
    let _ = writeln!(out, "        repeat-delay {}", input.repeat_delay.clamp(100, 2000));
    let _ = writeln!(out, "        repeat-rate {}", input.repeat_rate.clamp(1, 100));
    let _ = writeln!(out, "        numlock{}", if input.numlock { "" } else { " false" });
    out.push_str("    }\n");
}

/// The `input` section. The xkb block is written only once layouts were
/// chosen here, so until then niri keeps the system layout from locale1.
pub fn input_kdl(input: &InputConfig) -> String {
    let mut out = String::from(HEADER);
    out.push_str("\ninput {\n");
    keyboard_kdl(&mut out, input);
    out.push_str("    touchpad {\n");
    flag(&mut out, "tap", input.tap);
    flag(&mut out, "natural-scroll", input.touchpad_natural);
    flag(&mut out, "dwt", input.dwt);
    let _ = writeln!(out, "        accel-speed {}", speed(input.touchpad_speed));
    let _ = writeln!(out, "        scroll-method \"{}\"", input.scroll.niri());
    out.push_str("    }\n    mouse {\n");
    flag(&mut out, "natural-scroll", input.mouse_natural);
    let _ = writeln!(out, "        accel-speed {}", speed(input.mouse_speed));
    let _ = writeln!(out, "        accel-profile \"{}\"", input.mouse_profile.niri());
    out.push_str("    }\n}\n");
    out
}
