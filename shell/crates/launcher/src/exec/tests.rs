//! The `Exec` line checks: field codes, quoting, terminals and a real detached spawn.

use super::*;

#[test]
fn field_codes_are_stripped() {
    assert_eq!(strip_field_codes("kitty %u"), "kitty ");
    assert_eq!(strip_field_codes("foo%%bar"), "foo%bar");
    assert_eq!(strip_field_codes("%f %F %u %U %i %c %k %%"), "       %");
    assert_eq!(strip_field_codes("trailing %"), "trailing ");
}

#[test]
fn quotes_and_escapes_are_respected() {
    assert_eq!(
        split_exec("sh -c \"echo hi > /tmp/out\""),
        vec!["sh", "-c", "echo hi > /tmp/out"]
    );
    assert_eq!(split_exec("env X=\"a b\" prog"), vec!["env", "X=a b", "prog"]);
    assert_eq!(split_exec("prog a\\ b"), vec!["prog", "a b"]);
    assert_eq!(split_exec("kitty"), vec!["kitty"]);
    assert!(split_exec("").is_empty());
}

#[test]
fn terminal_entries_run_in_the_terminal() {
    assert_eq!(command_line("htop %U", false), vec!["htop"]);
    assert_eq!(command_line("htop %U", true), vec!["ghostty", "-e", "htop"]);
    assert_eq!(
        command_line("nvim --cmd \"set nu\" %F", true),
        vec!["ghostty", "-e", "nvim", "--cmd", "set nu"]
    );
    assert!(command_line("%U", true).is_empty());
}

#[test]
fn launch_spawns_detached() {
    assert!(launch("echo ok", false).is_ok());
    assert!(launch("", false).is_err());
    assert!(launch("flux-no-such-program", false).is_err());
}
