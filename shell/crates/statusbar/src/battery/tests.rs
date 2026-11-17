//! The popup's wording: the state in words and the time line under it.

use super::*;

#[test]
fn every_state_has_words() {
    let strings = text::english();
    assert_eq!(state_words(State::Charging, strings), "Charging");
    assert_eq!(state_words(State::Discharging, strings), "On battery");
    assert_eq!(state_words(State::Full, strings), "Full");
    assert_eq!(state_words(State::Plugged, strings), "Plugged in, not charging");
    assert_eq!(state_words(State::Unknown, strings), "Unknown");
}

#[test]
fn a_battery_that_is_moving_gets_a_time_line() {
    let strings = text::english();
    assert_eq!(
        remaining_text(State::Charging, Some(80), strings).as_deref(),
        Some("Full in 1h 20m")
    );
    assert_eq!(
        remaining_text(State::Discharging, Some(65), strings).as_deref(),
        Some("Empty in 1h 05m")
    );
}

#[test]
fn nothing_is_counted_when_nothing_is_moving() {
    let strings = text::english();
    assert_eq!(remaining_text(State::Plugged, Some(95), strings), None);
    assert_eq!(remaining_text(State::Full, Some(95), strings), None);
    assert_eq!(remaining_text(State::Unknown, Some(95), strings), None);
    assert_eq!(remaining_text(State::Charging, Some(0), strings), None);
    assert_eq!(remaining_text(State::Discharging, None, strings), None);
}

#[test]
fn the_percentage_follows_shell_json() {
    assert!(percentage_on(r#"{"battery_percentage": true, "other": 1}"#));
    assert!(!percentage_on(r#"{"battery_percentage": false}"#));
    assert!(!percentage_on("{}"));
    assert!(!percentage_on("not json"));
}

#[test]
fn a_low_battery_shows_its_level_whatever_the_setting() {
    assert_eq!(pill_text(80, State::Discharging, false), "");
    assert_eq!(pill_text(80, State::Discharging, true), "80%");
    assert_eq!(pill_text(20, State::Discharging, false), "20%");
    assert_eq!(pill_text(20, State::Charging, false), "");
    assert_eq!(pill_text(20, State::Charging, true), "20%+");
    assert!(is_low(20, State::Discharging));
    assert!(!is_low(21, State::Discharging));
    assert!(!is_low(5, State::Charging));
    assert!(!is_low(5, State::Plugged));
}

#[test]
fn the_glyph_follows_the_level() {
    assert_eq!(glyph(100, State::Full), Icon::Battery);
    assert_eq!(glyph(61, State::Discharging), Icon::Battery);
    assert_eq!(glyph(60, State::Discharging), Icon::BatteryHalf);
    assert_eq!(glyph(20, State::Discharging), Icon::BatteryLow);
    assert_eq!(glyph(3, State::Plugged), Icon::BatteryLow);
    assert_eq!(glyph(3, State::Charging), Icon::BatteryCharge);
}

#[test]
fn each_warning_level_is_crossed_once_on_the_way_down() {
    assert_eq!(crossed(16, 15), Some(15));
    assert_eq!(crossed(15, 14), None);
    assert_eq!(crossed(6, 5), Some(5));
    assert_eq!(crossed(5, 4), None);
    assert_eq!(crossed(40, 39), None);
    assert_eq!(crossed(14, 16), None);
    assert_eq!(crossed(30, 3), Some(5), "a long sleep can skip a level");
    assert_eq!(crossed(NOT_DRAINING, 12), Some(15), "unplugged while already low");
    assert_eq!(crossed(NOT_DRAINING, 50), None);
}

#[test]
fn the_watch_warns_once_per_level_and_forgets_on_the_charger() {
    assert_eq!(watch_step(NOT_DRAINING, 16, State::Discharging), (16, false));
    assert_eq!(watch_step(16, 15, State::Discharging), (15, true));
    assert_eq!(watch_step(15, 15, State::Discharging), (15, false));
    assert_eq!(watch_step(15, 15, State::Charging), (NOT_DRAINING, false));
    assert_eq!(watch_step(NOT_DRAINING, 15, State::Discharging), (15, true));
    assert_eq!(watch_step(6, 5, State::Plugged), (NOT_DRAINING, false));
}
