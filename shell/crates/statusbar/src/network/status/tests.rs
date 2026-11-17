use super::*;

#[test]
fn a_network_without_internet_is_limited_not_unknown() {
    assert_eq!(nm_kind("connected"), Some(StateKind::Connected));
    assert_eq!(nm_kind("connected (site only)"), Some(StateKind::Limited));
    assert_eq!(nm_kind("connected (local only)"), Some(StateKind::Limited));
    assert_eq!(nm_kind("connecting"), Some(StateKind::Connecting));
    assert_eq!(nm_kind("disconnected"), Some(StateKind::Disconnected));
    assert_eq!(nm_kind("asleep"), Some(StateKind::Unknown));
    assert_eq!(nm_kind(""), None);
}

#[test]
fn the_active_line_names_the_network() {
    assert_eq!(active_value("no:Neighbour\nyes:Home\n").as_deref(), Some("Home"));
    assert_eq!(active_value("no:Neighbour\n"), None);
    assert_eq!(active_value("yes:\nyes:Wired connection 1\n").as_deref(), Some("Wired connection 1"));
    assert_eq!(active_value(""), None);
}

#[test]
fn escaped_colons_and_backslashes_read_as_typed() {
    assert_eq!(active_value(r"yes:Cafe\: Guest").as_deref(), Some("Cafe: Guest"));
    assert_eq!(active_value(r"yes:back\\slash").as_deref(), Some(r"back\slash"));
}
