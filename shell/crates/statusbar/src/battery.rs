//! Battery pill for the statusbar: the level on the bar, and a detail popup on
//! click. The ACPI reading itself lives in [`power`].

mod power;

#[cfg(test)]
mod tests;

use gtk4::prelude::*;

use crate::popup::{self, FillBar, Popup};
use crate::{text, work};
use power::{Battery, State};

/// The pill's label, or `None` to hide the pill: the level, and a `+` while
/// the battery charges.
pub fn label() -> Option<String> {
    let battery = Battery::read()?;
    let charging = battery.state() == State::Charging;
    Some(format!("{}%{}", battery.pct, if charging { "+" } else { "" }))
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
