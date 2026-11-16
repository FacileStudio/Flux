//! Durations and battery lines as the page writes them.

use crate::i18n::Text;
use crate::upower::{Battery, State};

/// `seconds` as `2 h 15 min`, `45 min` or `2 h`. Under a minute rounds up to
/// one minute; `None` is the word for never.
pub fn duration(text: &Text, seconds: Option<u32>) -> String {
    let Some(seconds) = seconds else {
        return text.never.to_string();
    };
    let minutes = seconds.div_ceil(60).max(1);
    let (hours, minutes) = (minutes / 60, minutes % 60);
    match (hours, minutes) {
        (0, minutes) => format!("{minutes} {}", text.minutes),
        (hours, 0) => format!("{hours} {}", text.hours),
        (hours, minutes) => format!("{hours} {} {minutes} {}", text.hours, text.minutes),
    }
}

/// The battery's state with its time estimate when UPower has one:
/// `On battery · 3 h 12 min left`.
pub fn battery_line(text: &Text, battery: &Battery) -> String {
    let (state, estimate, suffix) = match battery.state {
        State::Charging => (text.charging, battery.time_to_full, text.until_full),
        State::Discharging => (text.discharging, battery.time_to_empty, text.left),
        State::Full => (text.full, 0, ""),
        State::NotCharging => (text.not_charging, 0, ""),
    };
    match u32::try_from(estimate).ok().filter(|seconds| *seconds > 0) {
        Some(seconds) => format!("{state} · {} {suffix}", duration(text, Some(seconds))),
        None => state.to_string(),
    }
}

/// A percentage as the page prints it: `76 %` in both languages.
pub fn percent(value: f64) -> String {
    format!("{} %", value.clamp(0.0, 100.0).round() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{EN, FR};

    #[test]
    fn durations_read_naturally() {
        assert_eq!(duration(&EN, None), "Never");
        assert_eq!(duration(&FR, None), "Jamais");
        assert_eq!(duration(&EN, Some(20)), "1 min");
        assert_eq!(duration(&EN, Some(300)), "5 min");
        assert_eq!(duration(&EN, Some(3600)), "1 h");
        assert_eq!(duration(&EN, Some(8100)), "2 h 15 min");
        assert_eq!(duration(&EN, Some(61)), "2 min");
    }

    #[test]
    fn battery_lines_carry_the_estimate_when_known() {
        let mut battery = Battery { percentage: 76.0, state: State::Discharging, time_to_empty: 11_520, ..Battery::default() };
        assert_eq!(battery_line(&EN, &battery), "On battery · 3 h 12 min left");
        battery.time_to_empty = 0;
        assert_eq!(battery_line(&EN, &battery), "On battery");
        battery.state = State::Charging;
        battery.time_to_full = 1800;
        assert_eq!(battery_line(&FR, &battery), "En charge · 30 min avant la charge complète");
        assert_eq!(percent(75.6), "76 %");
        assert_eq!(percent(140.0), "100 %");
    }
}
