//! The kernel's radio switches: read from sysfs, and the Bluetooth soft block
//! lifted through `/dev/rfkill`, which logind gives the active seat's user.
//!
//! `FLUX_BLUETOOTH_RFKILL` points the reader at another directory, for tests.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::model::RfkillEntry;

const SYSFS: &str = "/sys/class/rfkill";
const DEVICE: &str = "/dev/rfkill";
const TYPE_BLUETOOTH: u8 = 2;
const OP_CHANGE_ALL: u8 = 3;

fn root() -> PathBuf {
    std::env::var_os("FLUX_BLUETOOTH_RFKILL")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(SYSFS))
}

fn field(dir: &Path, name: &str) -> Option<String> {
    std::fs::read_to_string(dir.join(name)).ok().map(|text| text.trim().to_string())
}

fn entry(dir: &Path) -> Option<RfkillEntry> {
    Some(RfkillEntry {
        kind: field(dir, "type")?,
        soft: field(dir, "soft")? == "1",
        hard: field(dir, "hard")? == "1",
    })
}

/// Every switch under the rfkill class; empty when the directory is missing.
pub fn read() -> Vec<RfkillEntry> {
    let Ok(entries) = std::fs::read_dir(root()) else {
        return Vec::new();
    };
    entries.flatten().filter_map(|item| entry(&item.path())).collect()
}

/// The `rfkill_event` that clears the soft block on every Bluetooth radio.
pub fn unblock_event() -> [u8; 8] {
    let mut event = [0u8; 8];
    event[4] = TYPE_BLUETOOTH;
    event[5] = OP_CHANGE_ALL;
    event
}

/// Lift the Bluetooth soft block.
pub fn unblock() -> std::io::Result<()> {
    std::fs::OpenOptions::new().write(true).open(DEVICE)?.write_all(&unblock_event())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_targets_bluetooth_and_clears_the_soft_block() {
        assert_eq!(unblock_event(), [0, 0, 0, 0, 2, 3, 0, 0]);
    }

    #[test]
    fn reads_switches_from_a_directory() {
        let root = std::env::temp_dir().join(format!("flux-rfkill-{}", std::process::id()));
        let write = |name: &str, kind: &str, soft: &str| {
            let dir = root.join(name);
            std::fs::create_dir_all(&dir).ok();
            std::fs::write(dir.join("type"), format!("{kind}\n")).ok();
            std::fs::write(dir.join("soft"), format!("{soft}\n")).ok();
            std::fs::write(dir.join("hard"), "0\n").ok();
        };
        write("rfkill0", "wlan", "0");
        write("rfkill1", "bluetooth", "1");
        let entries = std::fs::read_dir(&root)
            .map(|entries| entries.flatten().filter_map(|item| entry(&item.path())).count())
            .unwrap_or(0);
        let bluetooth = entry(&root.join("rfkill1"));
        std::fs::remove_dir_all(&root).ok();
        assert_eq!(entries, 2);
        assert_eq!(
            bluetooth,
            Some(RfkillEntry {
                kind: "bluetooth".into(),
                soft: true,
                hard: false
            })
        );
    }
}
