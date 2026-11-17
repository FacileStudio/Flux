//! Volume pill and its popup, driven by PipeWire's pulse server through `pactl`.
//!
//! Every `pactl` call here blocks, so the pill and the popup only call these
//! from [`crate::work::off_main`].

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation};

use crate::control::{Changes, Reading};
use crate::icons::Icon;
use crate::popup::{self, FillBar, Popup};
use crate::{text, work};

/// One step of the popup's buttons, the same as the volume keys.
const STEP: &str = "5%";

/// How long the follower waits before asking a sound server that went away again.
const RETRY: Duration = Duration::from_secs(5);

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

/// The pill's reading, or `None` to hide the pill.
pub fn reading() -> Option<Reading> {
    read().map(|level| Reading::of(Icon::Volume, pill_text(level, text::get())))
}

/// Report every change of a sink, or of the server's default sink, on
/// `changes`: one `pactl subscribe` child for the whole process, read on a
/// thread of its own and started again when the sound server restarts. The
/// volume keys change the level outside the bar, and this is what makes the
/// pill follow them at once. Ends when `pactl` is missing or nobody listens.
pub fn follow(changes: Changes) {
    std::thread::spawn(move || loop {
        let child = Command::new("pactl")
            .arg("subscribe")
            .env("LC_ALL", "C")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();
        let Ok(mut child) = child else {
            return;
        };
        let _ = changes.try_send(());
        let lines = child.stdout.take().map(|stdout| BufReader::new(stdout).lines());
        for line in lines.into_iter().flatten().map_while(Result::ok) {
            if concerns_sink(&line) {
                let _ = changes.try_send(());
            }
            if changes.is_closed() {
                break;
            }
        }
        let _ = child.kill();
        let _ = child.wait();
        if changes.is_closed() {
            return;
        }
        std::thread::sleep(RETRY);
    });
}

/// Whether one `pactl subscribe` line is about a sink or the server, and not
/// about a stream, a source or a client.
pub fn concerns_sink(line: &str) -> bool {
    line.split_whitespace().any(|word| matches!(word, "sink" | "server"))
}

/// The pill's words for a level.
pub fn pill_text(level: Level, strings: &text::Text) -> String {
    if level.muted {
        strings.muted.to_string()
    } else {
        format!("{}%", level.percent)
    }
}

/// The standard output of one successful `pactl` call, in the C locale: the
/// parsers below read English words, and `pactl` translates its output.
fn pactl(args: &[&str]) -> Option<String> {
    let out = Command::new("pactl").args(args).env("LC_ALL", "C").output().ok()?;
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
pub fn draw(popup: &Popup, level: Level) {
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
    fn only_sink_and_server_events_refresh_the_pill() {
        assert!(concerns_sink("Event 'change' on sink #53"));
        assert!(concerns_sink("Event 'change' on server #0"));
        assert!(concerns_sink("Event 'new' on sink #61"));
        assert!(!concerns_sink("Event 'change' on sink-input #120"));
        assert!(!concerns_sink("Event 'change' on source #54"));
        assert!(!concerns_sink("Event 'new' on client #99"));
        assert!(!concerns_sink(""));
    }

    #[test]
    fn a_muted_sink_says_so() {
        let strings = text::english();
        assert_eq!(pill_text(Level { percent: 40, muted: false }, strings), "40%");
        assert_eq!(pill_text(Level { percent: 40, muted: true }, strings), "Muted");
    }
}
