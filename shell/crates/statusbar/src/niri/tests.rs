//! The niri state checks, fed with events shaped like niri's own JSON.

use super::*;

const TWO_OUTPUTS: &str = r#"{"WorkspacesChanged":{"workspaces":[
  {"id":4,"idx":2,"name":null,"output":"HDMI-A-1","is_urgent":false,"is_active":false,"is_focused":false,"active_window_id":null},
  {"id":1,"idx":1,"name":null,"output":"eDP-1","is_urgent":false,"is_active":true,"is_focused":true,"active_window_id":10},
  {"id":3,"idx":1,"name":"web","output":"HDMI-A-1","is_urgent":false,"is_active":true,"is_focused":false,"active_window_id":20},
  {"id":2,"idx":2,"name":null,"output":"eDP-1","is_urgent":false,"is_active":false,"is_focused":false,"active_window_id":null}
]}}"#;

const WINDOWS: &str = r#"{"WindowsChanged":{"windows":[
  {"id":10,"title":"Terminal","app_id":"com.mitchellh.ghostty","pid":7,"workspace_id":1,"is_focused":true,"is_floating":false,"is_urgent":false},
  {"id":20,"title":null,"app_id":"firefox","pid":8,"workspace_id":3,"is_focused":false,"is_floating":false,"is_urgent":false}
]}}"#;

fn event(line: &str) -> Event {
    serde_json::from_str(line).expect("a niri event")
}

fn state() -> State {
    let mut state = State::default();
    assert!(state.apply(event(TWO_OUTPUTS)));
    assert!(state.apply(event(WINDOWS)));
    state
}

fn labels(view: &View) -> Vec<&str> {
    view.chips.iter().map(|chip| chip.label.as_str()).collect()
}

#[test]
fn each_output_gets_its_own_workspaces_in_order() {
    let state = state();
    let laptop = state.view(Some("eDP-1"));
    assert_eq!(laptop.chips.iter().map(|chip| chip.id).collect::<Vec<_>>(), vec![1, 2]);
    assert_eq!(labels(&laptop), vec!["1", "2"]);
    assert!(laptop.chips[0].focused && laptop.chips[0].active);

    let screen = state.view(Some("HDMI-A-1"));
    assert_eq!(labels(&screen), vec!["web", "2"]);
    assert!(screen.chips[0].active && !screen.chips[0].focused);

    assert!(state.view(Some("DP-9")).chips.is_empty());
    let all = state.view(None);
    assert_eq!(all.chips.iter().map(|chip| chip.id).collect::<Vec<_>>(), vec![3, 4, 1, 2]);
}

#[test]
fn each_output_shows_the_window_in_front_of_it() {
    let state = state();
    assert_eq!(state.view(Some("eDP-1")).title.as_deref(), Some("Terminal"));
    assert_eq!(state.view(Some("HDMI-A-1")).title.as_deref(), Some("firefox"));
    assert_eq!(state.view(None).title.as_deref(), Some("Terminal"));
}

#[test]
fn activating_moves_focus_but_keeps_other_outputs_active() {
    let mut state = state();
    assert!(state.apply(event(r#"{"WorkspaceActivated":{"id":4,"focused":true}}"#)));
    let screen = state.view(Some("HDMI-A-1"));
    assert!(!screen.chips[0].active, "web is no longer active on its output");
    assert!(screen.chips[1].active && screen.chips[1].focused);
    let laptop = state.view(Some("eDP-1"));
    assert!(laptop.chips[0].active, "the laptop keeps its active workspace");
    assert!(!laptop.chips[0].focused, "but loses the focus");
    assert_eq!(screen.title, None, "the new workspace is empty");

    assert!(state.apply(event(r#"{"WorkspaceActivated":{"id":2,"focused":false}}"#)));
    let laptop = state.view(Some("eDP-1"));
    assert!(!laptop.chips[0].active && laptop.chips[1].active);
    assert!(state.view(Some("HDMI-A-1")).chips[1].focused, "focus stays where it was");
}

#[test]
fn urgency_and_window_events_update_the_view() {
    let mut state = state();
    assert!(state.apply(event(r#"{"WorkspaceUrgencyChanged":{"id":2,"urgent":true}}"#)));
    assert!(state.view(Some("eDP-1")).chips[1].urgent);
    assert!(!state.apply(event(r#"{"WorkspaceUrgencyChanged":{"id":99,"urgent":true}}"#)));

    assert!(state.apply(event(
        r#"{"WindowOpenedOrChanged":{"window":{"id":10,"title":"nvim","app_id":"x","pid":7,"workspace_id":1,"is_focused":true,"is_floating":false,"is_urgent":false}}}"#
    )));
    assert_eq!(state.view(Some("eDP-1")).title.as_deref(), Some("nvim"));

    assert!(state.apply(event(
        r#"{"WorkspaceActiveWindowChanged":{"workspace_id":1,"active_window_id":null}}"#
    )));
    assert_eq!(state.view(Some("eDP-1")).title, None);

    assert!(state.apply(event(r#"{"WindowFocusChanged":{"id":20}}"#)));
    assert_eq!(state.view(None).title.as_deref(), Some("firefox"));
    assert!(state.apply(event(r#"{"WindowClosed":{"id":20}}"#)));
    assert_eq!(state.view(Some("HDMI-A-1")).title, None);
}

#[test]
fn unknown_events_do_not_parse() {
    assert!(serde_json::from_str::<Event>(r#"{"OverviewOpenedOrClosed":{"is_open":true}}"#).is_err());
}

#[test]
fn the_layout_shows_only_with_several_layouts() {
    let mut state = State::default();
    assert!(state.apply(event(
        r#"{"KeyboardLayoutsChanged":{"keyboard_layouts":{"names":["English (US)"],"current_idx":0}}}"#
    )));
    assert_eq!(state.layout(), None);
    assert!(state.apply(event(
        r#"{"KeyboardLayoutsChanged":{"keyboard_layouts":{"names":["English (US)","French"],"current_idx":0}}}"#
    )));
    assert_eq!(state.layout(), Some(("US".into(), "English (US)".into())));
    assert!(state.apply(event(r#"{"KeyboardLayoutSwitched":{"idx":1}}"#)));
    assert_eq!(state.layout(), Some(("FR".into(), "French".into())));
}

#[test]
fn layout_names_shorten_to_a_code() {
    assert_eq!(short_layout("English (US)"), "US");
    assert_eq!(short_layout("French"), "FR");
    assert_eq!(short_layout("German (Switzerland)"), "GE");
    assert_eq!(short_layout("English (UK)"), "UK");
}

#[test]
fn requests_are_niri_json() {
    let focus: serde_json::Value = serde_json::from_str(&focus_request(5)).expect("json");
    assert_eq!(focus["Action"]["FocusWorkspace"]["reference"]["Id"], 5);
    let layout: serde_json::Value = serde_json::from_str(NEXT_LAYOUT).expect("json");
    assert_eq!(layout["Action"]["SwitchLayout"]["layout"], "Next");
}
