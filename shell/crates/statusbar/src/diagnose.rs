//! What the bar placed and what it did not: `statusbar --diagnose`.
//!
//! A control pill shows only while its backend answers, and its icon is painted
//! only when the artwork reaches a texture. From outside the bar those two look
//! the same: a gap in the row, or a pill with a label and no glyph. This prints
//! which control answers, which does not and why, whether the glyphs painted,
//! whether niri is reachable, and whether the machine is virtualised.
//!
//! It reads and reports; it changes nothing.

use std::fmt::Write as _;

use gtk4::prelude::*;

use crate::{control, icons, niri};

/// What to look for when a control answers, and what to install when it does not.
fn reasons(name: &str) -> (&'static str, &'static str) {
    match name {
        "volume" => (
            "pactl answers for the default sink",
            "no pactl, or nothing for it to answer for (pulseaudio, pipewire-pulse)",
        ),
        "brightness" => (
            "a backlight is present",
            "nothing in /sys/class/backlight (a virtual machine has none)",
        ),
        "battery" => (
            "a BAT device is present",
            "no BAT device in /sys/class/power_supply (a virtual machine has none)",
        ),
        _ => (
            "a link answers",
            "no NetworkManager, no systemd-networkd, no non-loopback interface",
        ),
    }
}

/// The vendors whose guests are virtual machines, as the firmware names them.
const VIRTUAL_VENDORS: [&str; 9] = [
    "QEMU",
    "KVM",
    "VMware",
    "VirtualBox",
    "innotek",
    "Apple Virtualization",
    "Parallels",
    "Xen",
    "Microsoft Corporation",
];

/// The whole report, one block per question, in the order someone reads it.
pub fn report() -> String {
    let mut text = String::new();
    let _ = writeln!(text, "{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let niri = if niri::present() { "NIRI_SOCKET is set" } else { "NIRI_SOCKET is unset" };
    let _ = writeln!(text, "niri: {niri}");
    let _ = writeln!(text, "machine: {}", machine());
    let _ = writeln!(text, "display: {}", display());
    let _ = writeln!(text, "GSK_RENDERER: {}", renderer());
    text.push_str(&pills());
    text.push_str(&glyphs());
    text
}

/// The control pills, each one answering or not, with the reason either way
/// and what it reads.
fn pills() -> String {
    let mut text = String::from("pills\n");
    for control in &control::CONTROLS {
        let reading = (control.label)();
        let (found, missing) = reasons(control.name);
        let _ = writeln!(text, "{}", row(control.name, reading.is_some(), found, missing));
        if let Some(reading) = reading {
            let _ = writeln!(text, "  {:<11}reads {reading}", "");
        }
    }
    text
}

/// One control's line: its name, whether it is in the row, and the reason.
fn row(name: &str, placed: bool, found: &str, missing: &str) -> String {
    let verdict = if placed { "placed" } else { "absent" };
    let why = if placed { found } else { missing };
    format!("  {name:<11}{verdict:<8}{why}")
}

/// Whether every icon reaches a texture, and the size it paints at.
///
/// A texture is built from bytes in memory, so this needs no window — but it does
/// need a display, which is what GTK is initialised for here. Without one the
/// icons cannot be painted at all, and saying that is the honest answer.
fn glyphs() -> String {
    let mut text = String::from("glyphs\n");
    if gtk4::init().is_err() {
        text.push_str("  not checked: no display is up, so nothing can be painted\n");
        return text;
    }
    let tint = icons::tint();
    for icon in icons::ALL {
        let texture = icons::texture(icon, &tint);
        let _ = writeln!(
            text,
            "  {:<11}{}×{}",
            format!("{icon:?}").to_lowercase(),
            texture.width(),
            texture.height()
        );
    }
    text
}

/// What this machine says it is, from the identifiers the firmware publishes.
///
/// A virtual machine is worth naming: on a virtual GPU, GTK4's renderer wants an
/// OpenGL version a guest often does not have, and that shows up as content that
/// never arrives rather than as an error.
fn machine() -> String {
    let id = |field: &str| {
        let path = format!("/sys/class/dmi/id/{field}");
        std::fs::read_to_string(path)
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    let name = format!("{} {}", id("sys_vendor"), id("product_name"))
        .trim()
        .to_string();
    if name.is_empty() {
        return "not published".to_string();
    }
    if VIRTUAL_VENDORS.iter().any(|mark| name.contains(mark)) {
        return format!("virtual ({name})");
    }
    name
}

/// Which display the bar is drawing on, since that decides what it can draw with.
fn display() -> String {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        return "wayland".to_string();
    }
    if std::env::var_os("DISPLAY").is_some() {
        return "x11, which has no layer surfaces to draw on".to_string();
    }
    "none".to_string()
}

/// The renderer `GSK_RENDERER` names, or the fact that it is unset and GTK picks.
fn renderer() -> String {
    std::env::var("GSK_RENDERER").unwrap_or_else(|_| "unset, GTK chooses".to_string())
}

#[cfg(test)]
mod tests;
