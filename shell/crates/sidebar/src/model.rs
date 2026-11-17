//! The notification history: what is kept, for how long, and where.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const MAX_NOTIFICATIONS: usize = 50;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub ts: i64,
    pub urgent: bool,
}

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

/// The age of `ts` at `now`, short: `just_now` under a minute, then minutes,
/// hours and days.
pub fn relative_time(ts: i64, now: i64, just_now: &str) -> String {
    let age = (now - ts).max(0);
    if age < 60 {
        just_now.to_string()
    } else if age < 3_600 {
        format!("{}m", age / 60)
    } else if age < 86_400 {
        format!("{}h", age / 3_600)
    } else {
        format!("{}d", age / 86_400)
    }
}

/// `$XDG_STATE_HOME/flux/notifications.json`, or the same under
/// `~/.local/state` when the variable is unset.
fn state_file() -> PathBuf {
    let base = std::env::var_os("XDG_STATE_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/state"));
    base.join("flux/notifications.json")
}

pub fn load() -> Vec<Notification> {
    std::fs::read_to_string(state_file())
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save(items: &[Notification]) {
    let path = state_file();
    let Some(parent) = path.parent() else {
        return;
    };
    if std::fs::create_dir_all(parent).is_err() {
        return;
    }
    let Ok(text) = serde_json::to_string_pretty(items) else {
        return;
    };
    let tmp = path.with_extension("json.tmp");
    if std::fs::write(&tmp, text).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}

pub fn insert(items: &mut Vec<Notification>, item: Notification) {
    items.retain(|existing| existing.id != item.id);
    items.insert(0, item);
    items.truncate(MAX_NOTIFICATIONS);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ages_read_short() {
        assert_eq!(relative_time(100, 130, "now"), "now");
        assert_eq!(relative_time(0, 600, "now"), "10m");
        assert_eq!(relative_time(0, 7_200, "now"), "2h");
        assert_eq!(relative_time(0, 172_800, "now"), "2d");
        assert_eq!(relative_time(500, 100, "now"), "now");
    }

    #[test]
    fn the_history_keeps_the_newest_once() {
        let mut items = Vec::new();
        for id in 0..60 {
            insert(&mut items, Notification { id, ..Notification::default() });
        }
        insert(&mut items, Notification { id: 30, summary: "again".into(), ..Notification::default() });
        assert_eq!(items.len(), MAX_NOTIFICATIONS);
        assert_eq!(items[0].id, 30);
        assert_eq!(items.iter().filter(|item| item.id == 30).count(), 1);
    }
}
