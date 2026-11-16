//! How the network state reads: the pill's one-liner and the popup's detail.

use super::status::{NetState, StateKind};
use crate::text::Text;

/// The pill's label: the SSID or connection name when connected, the state
/// otherwise.
pub(super) fn label(state: &NetState, strings: &Text) -> String {
    match state.kind {
        StateKind::Connected if !state.name.trim().is_empty() => short_name(&state.name),
        kind => words(kind, strings).to_string(),
    }
}

/// The state in words.
pub(super) fn words(kind: StateKind, strings: &Text) -> &'static str {
    match kind {
        StateKind::Connected => strings.online,
        StateKind::Connecting => strings.connecting,
        StateKind::Disconnecting => strings.disconnecting,
        StateKind::Disconnected => strings.offline,
        StateKind::Unknown => strings.unknown,
    }
}

/// The popup's detail line: the state in words, plus the name when there is one.
pub(super) fn detail(state: &NetState, strings: &Text) -> String {
    let state_words = words(state.kind, strings);
    if state.name.trim().is_empty() {
        state_words.to_string()
    } else {
        format!("{state_words}\n{}: {}", strings.interface, short_name(&state.name))
    }
}

/// `name` trimmed, cut to 13 characters plus an ellipsis past 16. The cut counts
/// characters, never bytes, so a multibyte SSID is never split inside a code point.
fn short_name(name: &str) -> String {
    let name = name.trim();
    if name.chars().count() <= 16 {
        return name.to_string();
    }
    let mut out: String = name.chars().take(13).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests;
