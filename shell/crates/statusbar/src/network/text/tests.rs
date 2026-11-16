use super::short_name;

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
