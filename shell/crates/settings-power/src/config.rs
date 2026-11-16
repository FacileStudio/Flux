//! `~/.config/flux/power.json`, which the page writes and `flux-idle` reads,
//! and the one key of `~/.config/flux/shell.json` the page owns.
//!
//! Every duration is in seconds, and `null` means never. A missing, unreadable
//! or partial file reads as the defaults for what it lacks, so a broken file
//! never stops the idle rules.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The shortest timeout the rules accept, so a typo cannot lock the user out
/// every few seconds.
pub const MIN_SECONDS: u32 = 30;
/// The longest timeout, one day.
pub const MAX_SECONDS: u32 = 86_400;

/// The key of `shell.json` the statusbar reads to show the battery percentage.
pub const BATTERY_PERCENTAGE_KEY: &str = "battery_percentage";

/// What closing the lid does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lid {
    #[default]
    Suspend,
    Nothing,
}

/// The idle and sleep rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PowerConfig {
    pub screen_off: Option<u32>,
    pub lock: Option<u32>,
    pub suspend_battery: Option<u32>,
    pub suspend_ac: Option<u32>,
    pub lid: Lid,
}

impl Default for PowerConfig {
    fn default() -> Self {
        PowerConfig {
            screen_off: Some(360),
            lock: Some(300),
            suspend_battery: Some(1200),
            suspend_ac: None,
            lid: Lid::Suspend,
        }
    }
}

impl PowerConfig {
    /// The same rules with every timeout clamped to what swayidle can take.
    pub fn sanitized(mut self) -> Self {
        for value in [&mut self.screen_off, &mut self.lock, &mut self.suspend_battery, &mut self.suspend_ac] {
            *value = value.map(|seconds| seconds.clamp(MIN_SECONDS, MAX_SECONDS));
        }
        self
    }

    /// The rules in a JSON document; anything unreadable gives the defaults.
    pub fn parse(json: &str) -> Self {
        serde_json::from_str::<PowerConfig>(json).unwrap_or_default().sanitized()
    }

    /// The JSON document for these rules.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into())
    }
}

/// `~/.config/flux/<name>`.
pub fn flux_file(name: &str) -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("HOME")?).join(".config/flux").join(name))
}

/// The rules from `~/.config/flux/power.json`.
pub fn load() -> PowerConfig {
    flux_file("power.json")
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|json| PowerConfig::parse(&json))
        .unwrap_or_default()
}

/// Write `contents` to `path` through a temporary file and a rename, so a
/// reader never sees half a document.
pub fn write_atomic(path: &Path, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, contents)?;
    std::fs::rename(&temporary, path)
}

/// Save the rules to `~/.config/flux/power.json`.
pub fn save(config: &PowerConfig) -> std::io::Result<()> {
    let path = flux_file("power.json").ok_or_else(|| std::io::Error::other("HOME is not set"))?;
    write_atomic(&path, &config.to_json())
}

/// `shell.json` with `key` set to `value`, keeping every other key. An
/// unreadable document is replaced by one holding only this key.
pub fn with_shell_key(document: &str, key: &str, value: serde_json::Value) -> String {
    let mut root = serde_json::from_str::<serde_json::Value>(document)
        .ok()
        .filter(serde_json::Value::is_object)
        .unwrap_or_else(|| serde_json::json!({}));
    if let Some(map) = root.as_object_mut() {
        map.insert(key.to_string(), value);
    }
    serde_json::to_string_pretty(&root).unwrap_or_else(|_| "{}".into())
}

/// Whether `shell.json` asks for the battery percentage in the bar.
pub fn battery_percentage(document: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(document)
        .ok()
        .and_then(|root| root.get(BATTERY_PERCENTAGE_KEY)?.as_bool())
        .unwrap_or(false)
}

/// Read the battery-percentage switch from `~/.config/flux/shell.json`.
pub fn load_battery_percentage() -> bool {
    flux_file("shell.json")
        .and_then(|path| std::fs::read_to_string(path).ok())
        .is_some_and(|document| battery_percentage(&document))
}

/// Set the battery-percentage switch in `~/.config/flux/shell.json`.
pub fn save_battery_percentage(on: bool) -> std::io::Result<()> {
    let path = flux_file("shell.json").ok_or_else(|| std::io::Error::other("HOME is not set"))?;
    let current = std::fs::read_to_string(&path).unwrap_or_default();
    write_atomic(&path, &with_shell_key(&current, BATTERY_PERCENTAGE_KEY, on.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_json_round_trips() {
        let config = PowerConfig {
            screen_off: Some(600),
            lock: None,
            suspend_battery: Some(900),
            suspend_ac: Some(3600),
            lid: Lid::Nothing,
        };
        assert_eq!(PowerConfig::parse(&config.to_json()), config);
    }

    #[test]
    fn broken_or_partial_files_fall_back_to_defaults() {
        assert_eq!(PowerConfig::parse("not json"), PowerConfig::default());
        let partial = PowerConfig::parse(r#"{"lock": null, "lid": "nothing"}"#);
        assert_eq!(partial.lock, None);
        assert_eq!(partial.lid, Lid::Nothing);
        assert_eq!(partial.screen_off, Some(360));
        assert_eq!(PowerConfig::parse(r#"{"lid": "explode"}"#), PowerConfig::default());
    }

    #[test]
    fn timeouts_are_clamped() {
        let config = PowerConfig::parse(r#"{"lock": 1, "screen_off": 999999}"#);
        assert_eq!(config.lock, Some(MIN_SECONDS));
        assert_eq!(config.screen_off, Some(MAX_SECONDS));
    }

    #[test]
    fn shell_json_keeps_other_keys() {
        let document = with_shell_key(r#"{"clock": "24h"}"#, BATTERY_PERCENTAGE_KEY, true.into());
        assert!(battery_percentage(&document));
        assert!(document.contains("\"clock\": \"24h\""));
        assert!(!battery_percentage(&with_shell_key("[]", BATTERY_PERCENTAGE_KEY, false.into())));
        assert!(!battery_percentage("garbage"));
    }

    #[test]
    fn writes_are_atomic_and_create_the_directory() {
        let dir = std::env::temp_dir().join(format!("flux-power-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("flux/power.json");
        write_atomic(&path, "{}").expect("write");
        assert_eq!(std::fs::read_to_string(&path).expect("read"), "{}");
        assert!(!path.with_extension("json.tmp").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
