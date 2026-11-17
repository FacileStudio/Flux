//! Brightness pill and its popup, driven by the backlight class.
//!
//! The level is read from `/sys/class/backlight`. Changes go through
//! `brightnessctl`, which works through its udev rule or logind, and fall back
//! to writing the attribute directly when the user may.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use gtk4::prelude::*;
use gtk4::{gio, Box, Button, Orientation};

use crate::control::{Changes, Reading};
use crate::icons::Icon;
use crate::popup::{self, FillBar, Popup};
use crate::{sysfs, text, work};

const STEP_PERCENT: u64 = 5;

/// Where the kernel publishes the backlights.
const BACKLIGHT_DIR: &str = "/sys/class/backlight";

/// The backlight to drive under `root`: the one with the finest range, never
/// the generic `virtual` one.
pub fn device_in(root: &Path) -> Option<PathBuf> {
    std::fs::read_dir(root)
        .ok()?
        .flatten()
        .filter(|entry| !entry.file_name().to_string_lossy().starts_with("virtual"))
        .filter_map(|entry| {
            let path = root.join(entry.file_name());
            let max = sysfs::read_u64(&path, "max_brightness").filter(|max| *max > 0)?;
            Some((max, path))
        })
        .max_by_key(|(max, _)| *max)
        .map(|(_, path)| path)
}

/// The level as a percent of the device's range.
pub fn percent() -> Option<u16> {
    let device = device_in(Path::new(BACKLIGHT_DIR))?;
    let max = sysfs::read_u64(&device, "max_brightness").filter(|max| *max > 0)?;
    let current = sysfs::read_u64(&device, "brightness")?;
    Some((current * 100 / max).min(100) as u16)
}

/// The pill's reading, or `None` to hide the pill.
pub fn reading() -> Option<Reading> {
    percent().map(|percent| Reading::of(Icon::Brightness, format!("{percent}%")))
}

/// Report every change of the backlight on `changes`. The kernel signals a
/// write to `brightness` on the device's `actual_brightness` attribute, so a
/// file monitor on that attribute sees the brightness keys. The monitor lives
/// as long as the process.
pub fn follow(changes: Changes) {
    let Some(device) = device_in(Path::new(BACKLIGHT_DIR)) else {
        return;
    };
    let attribute = gio::File::for_path(device.join("actual_brightness"));
    let Ok(monitor) = attribute.monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) else {
        return;
    };
    monitor.connect_changed(move |_, _, _, _| {
        let _ = changes.try_send(());
    });
    std::mem::forget(monitor);
}

/// Move the backlight one step up or down.
fn step(up: bool) {
    let change = format!("{STEP_PERCENT}%{}", if up { "+" } else { "-" });
    let ran = Command::new("brightnessctl")
        .args(["--quiet", "set", &change])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    if !ran {
        write_step(up);
    }
}

/// The same step, written to sysfs directly.
fn write_step(up: bool) {
    let Some(device) = device_in(Path::new(BACKLIGHT_DIR)) else {
        return;
    };
    let Some(max) = sysfs::read_u64(&device, "max_brightness").filter(|max| *max > 0) else {
        return;
    };
    let current = sysfs::read_u64(&device, "brightness").unwrap_or(0);
    let _ = sysfs::write_u64(&device, "brightness", stepped(current, max, up));
}

/// The raw level one step from `current` on a `max` range.
pub fn stepped(current: u64, max: u64, up: bool) -> u64 {
    let increment = (max * STEP_PERCENT).div_ceil(100).max(1);
    if up {
        current.saturating_add(increment).min(max)
    } else {
        current.saturating_sub(increment)
    }
}

/// Open the brightness popup once the level is read.
pub fn show(popup: &Popup) {
    let popup = popup.clone();
    work::off_main(percent, move |level| match level {
        Some(level) => draw(&popup, level),
        None => popup::message(&popup, text::get().brightness, text::get().unavailable),
    });
}

fn draw(popup: &Popup, level: u16) {
    popup.show(move |pop| {
        let value = popup::value(&format!("{level}%"));
        let fill = FillBar::new(level);
        let row = Box::new(Orientation::Horizontal, 8);
        row.append(&step_button(&value, &fill, false));
        row.append(&fill.root);
        row.append(&step_button(&value, &fill, true));
        pop.append(&popup::header(text::get().brightness));
        pop.append(&value);
        pop.append(&row);
    });
}

/// A `-` or `+` button that steps the backlight and redraws the popup.
fn step_button(value: &gtk4::Label, fill: &FillBar, up: bool) -> Button {
    let button = Button::with_label(if up { "+" } else { "-" });
    button.add_css_class("step");
    let (value, fill) = (value.clone(), fill.clone());
    button.connect_clicked(move |_| {
        let (value, fill) = (value.clone(), fill.clone());
        work::off_main(
            move || {
                step(up);
                percent()
            },
            move |level| {
                if let Some(level) = level {
                    value.set_text(&format!("{level}%"));
                    fill.set(level);
                }
            },
        );
    });
    button
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_step_is_five_percent_and_stays_in_range() {
        assert_eq!(stepped(50, 100, true), 55);
        assert_eq!(stepped(98, 100, true), 100);
        assert_eq!(stepped(3, 100, false), 0);
        assert_eq!(stepped(0, 7, true), 1);
        assert_eq!(stepped(0, 255, true), 13);
    }

    #[test]
    fn the_finest_real_backlight_wins() {
        let root = std::env::temp_dir().join(format!("flux-backlight-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (name, max) in [("virtual0", "1000"), ("acpi_video0", "15"), ("intel_backlight", "96000")] {
            std::fs::create_dir_all(root.join(name)).expect("device dir");
            std::fs::write(root.join(name).join("max_brightness"), max).expect("max");
        }
        assert_eq!(device_in(&root), Some(root.join("intel_backlight")));
        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(device_in(&root), None);
    }
}
