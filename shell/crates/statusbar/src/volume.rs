//! Volume pill and its popup, driven by PipeWire's pulse server through `pactl`.
//!
//! Every `pactl` call here blocks, so the pill and the popup only call these
//! from [`crate::work::off_main`].

use std::process::Command;

use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation};

use crate::popup::{self, FillBar, Popup};
use crate::{text, work};

/// One step of the popup's buttons, the same as the volume keys.
const STEP: &str = "5%";

/// What the default sink reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Level {
    pub percent: u16,
    pub muted: bool,
}

/// The default sink's level, or `None` when `pactl` is absent or has no server.
pub fn read() -> Option<Level> {
    let volume = pactl(&["get-sink-volume", "@DEFAULT_SINK@"])?;
    let mute = pactl(&["get-sink-mute", "@DEFAULT_SINK@"]).unwrap_or_default();
    Some(Level {
        percent: parse_percent(&volume)?,
        muted: parse_mute(&mute),
    })
}

/// The pill's label, or `None` to hide the pill.
pub fn label() -> Option<String> {
    read().map(|level| pill_text(level, text::get()))
}

/// The pill's words for a level.
pub fn pill_text(level: Level, strings: &text::Text) -> String {
    if level.muted {
        strings.muted.to_string()
    } else {
        format!("{}%", level.percent)
    }
}

/// The standard output of one successful `pactl` call.
fn pactl(args: &[&str]) -> Option<String> {
    let out = Command::new("pactl").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Move the default sink one step up or down.
fn step(up: bool) {
    let change = format!("{}{STEP}", if up { "+" } else { "-" });
    let _ = pactl(&["set-sink-volume", "@DEFAULT_SINK@", &change]);
}

/// Flip the default sink's mute.
fn toggle_mute() {
    let _ = pactl(&["set-sink-mute", "@DEFAULT_SINK@", "toggle"]);
}

/// The first channel's percent in `pactl get-sink-volume` output, capped at 100.
pub fn parse_percent(stdout: &str) -> Option<u16> {
    let line = stdout.lines().find_map(|line| line.strip_prefix("Volume:"))?;
    let first = line
        .split_whitespace()
        .find_map(|token| token.strip_suffix('%')?.trim().parse::<u16>().ok())?;
    Some(first.min(100))
}

/// Whether `pactl get-sink-mute` output says the sink is muted.
pub fn parse_mute(stdout: &str) -> bool {
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("Mute:"))
        .is_some_and(|value| value.trim() == "yes")
}

/// Open the volume popup once the level is read.
pub fn show(popup: &Popup) {
    let popup = popup.clone();
    work::off_main(read, move |level| match level {
        Some(level) => draw(&popup, level),
        None => popup::message(&popup, text::get().volume, text::get().unavailable),
    });
}

/// The popup: the level, a fill bar between the step buttons, and mute.
fn draw(popup: &Popup, level: Level) {
    let strings = text::get();
    popup.show(move |pop| {
        let value = popup::value(&pill_text(level, strings));
        let fill = FillBar::new(level.percent);
        let mute = Button::with_label(if level.muted { strings.unmute } else { strings.mute });
        mute.add_css_class("wide");

        let row = Box::new(Orientation::Horizontal, 8);
        row.append(&action_button("-", value.clone(), fill.clone(), mute.clone(), || step(false)));
        row.append(&fill.root);
        row.append(&action_button("+", value.clone(), fill.clone(), mute.clone(), || step(true)));
        let (mute_value, mute_fill, mute_label) = (value.clone(), fill.clone(), mute.clone());
        mute.connect_clicked(move |_| {
            apply(toggle_mute, mute_value.clone(), mute_fill.clone(), mute_label.clone());
        });

        pop.append(&popup::header(strings.volume));
        pop.append(&value);
        pop.append(&row);
        pop.append(&mute);
    });
}

/// A step button: run `change`, then show the level it left.
fn action_button(
    label: &str,
    value: gtk4::Label,
    fill: FillBar,
    mute: Button,
    change: fn(),
) -> Button {
    let button = Button::with_label(label);
    button.add_css_class("step");
    button.connect_clicked(move |_| apply(change, value.clone(), fill.clone(), mute.clone()));
    button
}

/// Run `change` off the main loop, then redraw the popup from a fresh read.
fn apply(change: fn(), value: gtk4::Label, fill: FillBar, mute: Button) {
    work::off_main(
        move || {
            change();
            read()
        },
        move |level| {
            let Some(level) = level else {
                return;
            };
            let strings = text::get();
            value.set_text(&pill_text(level, strings));
            fill.set(level.percent);
            mute.set_label(if level.muted { strings.unmute } else { strings.mute });
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pactl_output_reads_as_a_level() {
        let volume = "Volume: front-left: 34734 /  53% / -16.54 dB,   front-right: 34734 /  53% / -16.54 dB\n        balance 0.00\n";
        assert_eq!(parse_percent(volume), Some(53));
        assert_eq!(parse_percent("Volume: mono: 98304 / 150% / 10.57 dB"), Some(100));
        assert_eq!(parse_percent("Connection failure"), None);
        assert!(parse_mute("Mute: yes\n"));
        assert!(!parse_mute("Mute: no\n"));
        assert!(!parse_mute(""));
    }

    #[test]
    fn a_muted_sink_says_so() {
        let strings = text::english();
        assert_eq!(pill_text(Level { percent: 40, muted: false }, strings), "40%");
        assert_eq!(pill_text(Level { percent: 40, muted: true }, strings), "Muted");
    }
}
