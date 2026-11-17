//! Battery pill for the statusbar: the level on the bar, and a detail popup on
//! click. The ACPI reading itself lives in [`power`].

mod power;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{gio, glib};

use crate::control::{self, Changes};
use crate::icons::Icon;
use crate::popup::{self, FillBar, Popup};
use crate::{text, work};
use power::{Battery, State};

/// At or under this level, on battery, the pill turns red and shows its level.
const LOW: u8 = 20;

/// The levels that send a notification when the battery drains past them.
const WARNINGS: [u8; 2] = [5, 15];

/// A level no battery reports, standing for "not draining when last read".
const NOT_DRAINING: u8 = u8::MAX;

/// How often the low-battery watch reads the level.
const WATCH: Duration = Duration::from_secs(30);

/// The pill's reading, or `None` to hide the pill.
pub fn pill() -> Option<control::Reading> {
    let battery = Battery::read()?;
    let state = battery.state();
    Some(control::Reading {
        text: pill_text(battery.pct, state, shows_percentage()),
        icon: glyph(battery.pct, state),
        low: is_low(battery.pct, state),
    })
}

/// Watch the level for the whole process, on a thread of its own, and send one
/// notification each time the battery drains past a warning level. `changes`
/// gets a tick with it, so the pills turn red at the same moment. Ends when
/// nobody listens.
pub fn follow(changes: Changes) {
    std::thread::spawn(move || {
        let mut last = NOT_DRAINING;
        while !changes.is_closed() {
            if let Some(battery) = Battery::read() {
                let (level, warning) = watch_step(last, battery.pct, battery.state());
                last = level;
                if warning {
                    let strings = text::get();
                    let body = strings.battery_low_body.replace("{n}", &battery.pct.to_string());
                    notify(strings.battery, strings.battery_low, &body);
                    let _ = changes.try_send(());
                }
            }
            std::thread::sleep(WATCH);
        }
    });
}

/// One step of the watch: the level to remember, and whether this read is the
/// one that drains past a warning level. Plugging in forgets the level, so
/// unplugging a battery that is still low warns again.
fn watch_step(last: u8, pct: u8, state: State) -> (u8, bool) {
    if state != State::Discharging {
        return (NOT_DRAINING, false);
    }
    (pct, crossed(last, pct).is_some())
}

/// Whether the battery is running out: low, and nothing is feeding it.
fn is_low(pct: u8, state: State) -> bool {
    pct <= LOW && state == State::Discharging
}

/// The words beside the icon: the level and a `+` while the battery charges,
/// when Settings turned the percentage on or the battery is running out, and
/// nothing otherwise.
fn pill_text(pct: u8, state: State, always: bool) -> String {
    if !always && !is_low(pct, state) {
        return String::new();
    }
    format!("{pct}%{}", if state == State::Charging { "+" } else { "" })
}

/// The battery glyph for a level: the bolt while charging, then one, two or
/// three bars, cut at the same levels as the Flux icon theme.
fn glyph(pct: u8, state: State) -> Icon {
    match (state, pct) {
        (State::Charging, _) => Icon::BatteryCharge,
        (_, 0..=20) => Icon::BatteryLow,
        (_, 21..=60) => Icon::BatteryHalf,
        _ => Icon::Battery,
    }
}

/// The warning level the battery just drained past, going from `previous` to
/// `now`. A first read while draining counts as coming from above every level.
fn crossed(previous: u8, now: u8) -> Option<u8> {
    WARNINGS.into_iter().find(|level| now <= *level && previous > *level)
}

/// Show a critical notification through the session's notification service.
/// A critical one stays on screen until it is dismissed.
fn notify(app: &str, summary: &str, body: &str) {
    const CRITICAL: u8 = 2;
    const SERVER_DECIDES: i32 = -1;
    let hints: HashMap<&str, glib::Variant> = HashMap::from([("urgency", CRITICAL.to_variant())]);
    let message = (app, 0u32, "", summary, body, Vec::<String>::new(), hints, SERVER_DECIDES).to_variant();
    let sent = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).and_then(|bus| {
        bus.call_sync(
            Some("org.freedesktop.Notifications"),
            "/org/freedesktop/Notifications",
            "org.freedesktop.Notifications",
            "Notify",
            Some(&message),
            None,
            gio::DBusCallFlags::NONE,
            5000,
            gio::Cancellable::NONE,
        )
    });
    if let Err(error) = sent {
        eprintln!("statusbar: low battery notification: {error}");
    }
}

/// Whether `~/.config/flux/shell.json` turns the percentage on. A missing file or key means off.
fn shows_percentage() -> bool {
    std::env::var_os("HOME")
        .map(|home| std::path::PathBuf::from(home).join(".config/flux/shell.json"))
        .and_then(|path| std::fs::read_to_string(path).ok())
        .is_some_and(|document| percentage_on(&document))
}

/// Read the boolean `battery_percentage` key of a `shell.json` document.
fn percentage_on(document: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(document)
        .ok()
        .and_then(|value| value.get("battery_percentage").and_then(serde_json::Value::as_bool))
        .unwrap_or(false)
}

/// The state in words, for the popup.
fn state_words(state: State, strings: &text::Text) -> &'static str {
    match state {
        State::Charging => strings.charging,
        State::Discharging => strings.discharging,
        State::Full => strings.full,
        State::Plugged => strings.plugged,
        State::Unknown => strings.unknown,
    }
}

/// The "Full in 1h 20m" line, when the kernel reports a time and the state
/// gives that time a meaning. A plugged battery held at a charge threshold is
/// not draining, so it gets no line, and neither does a full or unknown one.
fn remaining_text(state: State, minutes: Option<u32>, strings: &text::Text) -> Option<String> {
    let minutes = minutes.filter(|minutes| *minutes > 0)?;
    let prefix = match state {
        State::Charging => strings.to_full,
        State::Discharging => strings.to_empty,
        State::Plugged | State::Full | State::Unknown => return None,
    };
    Some(format!("{prefix} {}h {:02}m", minutes / 60, minutes % 60))
}

/// What the popup shows, read in one pass off the main loop.
struct Reading {
    pct: u8,
    state: State,
    minutes: Option<u32>,
}

fn reading() -> Option<Reading> {
    let battery = Battery::read()?;
    Some(Reading {
        pct: battery.pct,
        state: battery.state(),
        minutes: battery.time_remaining(),
    })
}

/// Open the battery popup once the battery is read.
pub fn show(popup: &Popup) {
    let popup = popup.clone();
    work::off_main(reading, move |reading| {
        let strings = text::get();
        let Some(reading) = reading else {
            popup::message(&popup, strings.battery, strings.unavailable);
            return;
        };
        popup.show(move |pop| {
            pop.append(&popup::header(strings.battery));
            pop.append(&popup::value(&format!("{}%", reading.pct)));
            pop.append(&popup::detail(state_words(reading.state, strings)));
            if let Some(line) = remaining_text(reading.state, reading.minutes, strings) {
                pop.append(&popup::detail(&line));
            }
            let fill = FillBar::new(reading.pct.into());
            fill.root.set_margin_top(8);
            pop.append(&fill.root);
        });
    });
}
