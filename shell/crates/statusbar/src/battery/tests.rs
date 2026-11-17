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
