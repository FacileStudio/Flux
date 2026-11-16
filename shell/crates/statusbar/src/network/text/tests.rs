use super::*;
use crate::text;

fn state(kind: StateKind, name: &str) -> NetState {
    NetState {
        kind,
        name: name.to_string(),
    }
}

#[test]
fn short_names_are_left_alone() {
    assert_eq!(short_name("  home  "), "home");
    assert_eq!(short_name("exactly-16-chars"), "exactly-16-chars");
}

#[test]
fn long_names_are_cut_by_characters() {
    assert_eq!(short_name("a-very-long-network-name"), "a-very-long-n…");
    let cjk = "我的家庭无线网络名称很长很长";
    assert_eq!(short_name(cjk), cjk);
    let longer = "我的家庭无线网络名称很长很长很长很长";
    assert_eq!(short_name(longer), "我的家庭无线网络名称很长很…");
    let emoji = "🏠🏠🏠🏠🏠🏠🏠🏠🏠🏠🏠🏠🏠🏠🏠🏠🏠";
    assert_eq!(short_name(emoji).chars().count(), 14);
}

#[test]
fn the_pill_names_the_network_or_the_state() {
    let strings = text::english();
    assert_eq!(label(&state(StateKind::Connected, "Café Wi‑Fi ☕ très très long"), strings), "Café Wi‑Fi ☕ …");
    assert_eq!(label(&state(StateKind::Connected, ""), strings), "Online");
    assert_eq!(label(&state(StateKind::Disconnected, ""), strings), "Offline");
    assert_eq!(detail(&state(StateKind::Connected, "eth0"), strings), "Online\nInterface: eth0");
    assert_eq!(detail(&state(StateKind::Connecting, ""), strings), "Connecting");
}
