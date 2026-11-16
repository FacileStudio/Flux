//! Connected outputs as `niri msg --json outputs` reports them, and the
//! choices a person can make for one: mode, refresh rate, scale, rotation.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// The scale steps every output offers, before the one niri reports.
pub const SCALE_STEPS: [f64; 7] = [1.0, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0];

/// The narrowest logical width a scale may leave, so a step never makes the
/// desktop unusable.
pub const MIN_LOGICAL_WIDTH: f64 = 640.0;

/// One mode an output can run, as niri lists it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Mode {
    pub width: u16,
    pub height: u16,
    pub refresh_rate: u32,
    #[serde(default)]
    pub is_preferred: bool,
}

/// Where an enabled output sits in the global space, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Logical {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    pub transform: Transform,
}

/// Rotation and flip, counter-clockwise, as niri names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Transform {
    #[default]
    #[serde(alias = "normal")]
    Normal,
    #[serde(rename = "90")]
    R90,
    #[serde(rename = "180")]
    R180,
    #[serde(rename = "270")]
    R270,
    #[serde(alias = "flipped")]
    Flipped,
    #[serde(rename = "Flipped90", alias = "flipped-90")]
    Flipped90,
    #[serde(rename = "Flipped180", alias = "flipped-180")]
    Flipped180,
    #[serde(rename = "Flipped270", alias = "flipped-270")]
    Flipped270,
}

impl Transform {
    pub const ALL: [Transform; 8] = [
        Transform::Normal,
        Transform::R90,
        Transform::R180,
        Transform::R270,
        Transform::Flipped,
        Transform::Flipped90,
        Transform::Flipped180,
        Transform::Flipped270,
    ];

    /// The spelling niri's config and `niri msg output transform` take.
    pub fn niri(self) -> &'static str {
        match self {
            Transform::Normal => "normal",
            Transform::R90 => "90",
            Transform::R180 => "180",
            Transform::R270 => "270",
            Transform::Flipped => "flipped",
            Transform::Flipped90 => "flipped-90",
            Transform::Flipped180 => "flipped-180",
            Transform::Flipped270 => "flipped-270",
        }
    }

    /// Whether the output stands on its side, which swaps width and height.
    pub fn sideways(self) -> bool {
        matches!(
            self,
            Transform::R90 | Transform::R270 | Transform::Flipped90 | Transform::Flipped270
        )
    }
}

/// One connected output.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Output {
    pub name: String,
    #[serde(default)]
    pub make: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub serial: Option<String>,
    #[serde(default)]
    pub physical_size: Option<(u32, u32)>,
    #[serde(default)]
    pub modes: Vec<Mode>,
    #[serde(default)]
    pub current_mode: Option<usize>,
    #[serde(default)]
    pub vrr_supported: bool,
    #[serde(default)]
    pub vrr_enabled: bool,
    #[serde(default)]
    pub logical: Option<Logical>,
}

impl Output {
    /// The mode it runs now, or `None` when it is off.
    pub fn mode(&self) -> Option<Mode> {
        self.modes.get(self.current_mode?).copied()
    }

    /// The mode the monitor asks for, or the largest one.
    pub fn preferred(&self) -> Option<Mode> {
        self.modes
            .iter()
            .find(|mode| mode.is_preferred)
            .or_else(|| self.modes.iter().max_by_key(|mode| mode_rank(mode)))
            .copied()
    }

    pub fn enabled(&self) -> bool {
        self.logical.is_some()
    }

    /// The name niri matches config sections against: make, model and serial
    /// when the monitor reports a serial, which survives a change of port, and
    /// the connector otherwise.
    pub fn identity(&self) -> String {
        match self.serial.as_deref().map(str::trim) {
            Some(serial) if !serial.is_empty() && !self.make.is_empty() => {
                format!("{} {} {}", self.make, self.model, serial)
            }
            _ => self.name.clone(),
        }
    }

    /// "Dell U2723QE", or the connector when the monitor gives no name.
    pub fn display_name(&self) -> String {
        let name = format!("{} {}", self.make.trim(), self.model.trim());
        let name = name.trim();
        if name.is_empty() || name.eq_ignore_ascii_case("unknown unknown") {
            self.name.clone()
        } else {
            name.to_string()
        }
    }

    /// A built-in laptop panel.
    pub fn internal(&self) -> bool {
        let name = self.name.to_ascii_lowercase();
        ["edp", "lvds", "dsi"].iter().any(|prefix| name.starts_with(prefix))
    }

    /// The diagonal in inches, from the physical size in millimetres.
    pub fn diagonal_inches(&self) -> Option<f64> {
        let (width, height) = self.physical_size?;
        if width == 0 || height == 0 {
            return None;
        }
        let millimetres = f64::from(width).hypot(f64::from(height));
        Some((millimetres / 25.4 * 10.0).round() / 10.0)
    }
}

fn mode_rank(mode: &Mode) -> (u32, u32) {
    (u32::from(mode.width) * u32::from(mode.height), mode.refresh_rate)
}

/// Parse `niri msg --json outputs`: a map from connector to output, returned
/// sorted with built-in panels first, then by connector.
pub fn parse_outputs(json: &str) -> Result<Vec<Output>, String> {
    let map: HashMap<String, Output> = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let mut outputs: Vec<Output> = map.into_values().collect();
    outputs.sort_by(|a, b| b.internal().cmp(&a.internal()).then_with(|| a.name.cmp(&b.name)));
    Ok(outputs)
}

/// A resolution the output offers, with whether the monitor prefers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    pub width: u16,
    pub height: u16,
    pub preferred: bool,
}

/// Every distinct resolution, largest first.
pub fn resolutions(modes: &[Mode]) -> Vec<Resolution> {
    let mut list: Vec<Resolution> = Vec::new();
    for mode in modes {
        match list.iter_mut().find(|r| r.width == mode.width && r.height == mode.height) {
            Some(existing) => existing.preferred |= mode.is_preferred,
            None => list.push(Resolution { width: mode.width, height: mode.height, preferred: mode.is_preferred }),
        }
    }
    list.sort_by(|a, b| {
        let area = |r: &Resolution| u32::from(r.width) * u32::from(r.height);
        area(b).cmp(&area(a)).then(b.width.cmp(&a.width))
    });
    list
}

/// The refresh rates of one resolution, fastest first, without duplicates.
pub fn refresh_rates(modes: &[Mode], width: u16, height: u16) -> Vec<Mode> {
    let mut list: Vec<Mode> = modes.iter().filter(|m| m.width == width && m.height == height).copied().collect();
    list.sort_by_key(|mode| std::cmp::Reverse(mode.refresh_rate));
    list.dedup_by_key(|mode| mode.refresh_rate);
    list
}

/// "144", "59.94": whole hertz unless the fraction matters.
pub fn format_refresh(millihertz: u32) -> String {
    let hundredths = (f64::from(millihertz) / 10.0).round() as u64;
    if hundredths.is_multiple_of(100) {
        format!("{}", hundredths / 100)
    } else if hundredths.is_multiple_of(10) {
        format!("{}.{}", hundredths / 100, (hundredths % 100) / 10)
    } else {
        format!("{}.{:02}", hundredths / 100, hundredths % 100)
    }
}

/// "2560 x 1440".
pub fn format_resolution(width: u16, height: u16) -> String {
    format!("{width} x {height}")
}

/// "2560 x 1440 at 144 Hz", with `at` in the reader's language.
pub fn format_mode(mode: &Mode, at: &str) -> String {
    format!(
        "{} {at} {} Hz",
        format_resolution(mode.width, mode.height),
        format_refresh(mode.refresh_rate)
    )
}

/// The spelling niri wants in `mode` and `niri msg output mode`: the refresh
/// rate to the exact millihertz, as `niri msg outputs` prints it.
pub fn mode_kdl(width: u16, height: u16, millihertz: u32) -> String {
    format!("{width}x{height}@{}.{:03}", millihertz / 1000, millihertz % 1000)
}

/// Whether niri accepts `scale`.
pub fn valid_scale(scale: f64) -> bool {
    scale.is_finite() && scale > 0.0 && scale <= 10.0
}

/// "1.25" and "2": the shortest exact spelling of a scale.
pub fn format_scale(scale: f64) -> String {
    let text = format!("{:.2}", scale);
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// The scale steps offered for a mode: the fixed steps that keep the desktop
/// at least [`MIN_LOGICAL_WIDTH`] wide, plus `current` when it is valid and
/// not one of them. Sorted, without duplicates.
pub fn scale_choices(current: Option<f64>, width: u16) -> Vec<f64> {
    let mut list: Vec<f64> = SCALE_STEPS
        .iter()
        .copied()
        .filter(|scale| f64::from(width) / scale >= MIN_LOGICAL_WIDTH || *scale == 1.0)
        .collect();
    if let Some(current) = current.filter(|scale| valid_scale(*scale)) {
        if !list.iter().any(|step| (step - current).abs() < 0.005) {
            list.push(current);
        }
    }
    list.sort_by(|a, b| a.total_cmp(b));
    list
}
