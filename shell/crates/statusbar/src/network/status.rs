//! Where the network state comes from: NetworkManager's `nmcli` when it answers,
//! `networkctl` (systemd-networkd) otherwise, and the kernel's own interfaces
//! when neither daemon is installed — see [`super::link`]. Nothing here formats
//! or draws.

use std::process::Command;

use super::link;

/// A network manager's view of the connection.
pub(super) struct NetState {
    pub(super) kind: StateKind,
    pub(super) name: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum StateKind {
    Connected,
    Limited,
    Connecting,
    Disconnecting,
    Disconnected,
    Unknown,
}

/// The current state, or `None` when no manager and no interface can answer.
pub(super) fn state() -> Option<NetState> {
    nm_state().or_else(netctl_state).or_else(link::state)
}

fn nm_state() -> Option<NetState> {
    let text = nmcli(&["-t", "-f", "STATE", "general"])?;
    let kind = nm_kind(text.trim())?;
    let name = match kind {
        StateKind::Connected | StateKind::Limited => active_name().unwrap_or_default(),
        _ => String::new(),
    };
    Some(NetState { kind, name })
}

/// The state one `nmcli -t -f STATE general` answer names, or `None` for no
/// answer. `connected (site only)` and `connected (local only)` are a network
/// with no way out to the internet, which is what a hotel portal looks like
/// before its login page.
fn nm_kind(answer: &str) -> Option<StateKind> {
    Some(match answer {
        "" => return None,
        "connected" => StateKind::Connected,
        limited if limited.starts_with("connected") => StateKind::Limited,
        connecting if connecting.starts_with("connecting") => StateKind::Connecting,
        "disconnecting" => StateKind::Disconnecting,
        "disconnected" => StateKind::Disconnected,
        _ => StateKind::Unknown,
    })
}

/// The Wi-Fi network in use, or the name of the active connection (Ethernet,
/// VPN) when there is none.
///
/// `--rescan no` matters: without it `nmcli device wifi list` starts a Wi-Fi
/// scan whenever its list is older than 30 seconds, and the pill asks every
/// few seconds for the whole session.
fn active_name() -> Option<String> {
    nmcli(&["-t", "-f", "ACTIVE,SSID", "device", "wifi", "list", "--rescan", "no"])
        .and_then(|list| active_value(&list))
        .or_else(|| nmcli(&["-t", "-f", "ACTIVE,NAME", "connection", "show", "--active"]).and_then(|list| active_value(&list)))
}

/// The standard output of one `nmcli` call that started.
fn nmcli(args: &[&str]) -> Option<String> {
    let out = Command::new("nmcli").args(args).env("LC_ALL", "C").output().ok()?;
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The value of the first `yes:<value>` line of a terse two-field `nmcli`
/// list, with nmcli's escapes resolved.
fn active_value(list: &str) -> Option<String> {
    list.lines()
        .filter_map(|line| line.trim().strip_prefix("yes:"))
        .map(unescape)
        .find(|value| !value.trim().is_empty())
}

/// Undo terse mode's escaping: nmcli writes `\:` for a colon and `\\` for a
/// backslash inside a value.
fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        match character {
            '\\' => out.extend(characters.next()),
            other => out.push(other),
        }
    }
    out
}

fn netctl_state() -> Option<NetState> {
    let out = Command::new("networkctl")
        .args(["status", "--json=short"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    parse_networkctl_status(&text)
}

fn parse_networkctl_status(text: &str) -> Option<NetState> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let machine = value.get("Machine")?.as_array()?;
    for iface in machine {
        let ifname = iface.get("Interface")?.as_str()?;
        if ifname == "lo" {
            continue;
        }
        let substate = iface.get("State")?.as_str()?.to_ascii_lowercase();
        let kind = match substate.as_str() {
            "routable" | "configured" => StateKind::Connected,
            "configuring" => StateKind::Connecting,
            "degraded" => StateKind::Connected,
            "off" | "failed" | "absent" => StateKind::Disconnected,
            _ => StateKind::Unknown,
        };
        let name = iface.get("Interface")?.as_str()?.to_string();
        return Some(NetState { kind, name });
    }
    None
}

#[cfg(test)]
mod tests;
