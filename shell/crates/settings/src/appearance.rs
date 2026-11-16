//! The Appearance page: light or dark, the wallpaper, text size, cursor size and motion.

mod interface;
mod scheme;
mod wallpaper;

use std::process::Command;

use gtk4::prelude::*;
use shell_common::ui;

use crate::i18n;

pub const ID: &str = "appearance";
pub const ICON: &str = "preferences-desktop-appearance-symbolic";

/// The page's name in the navigation.
pub fn title() -> String {
    i18n::text().appearance_title.to_string()
}

/// The whole page.
pub fn page() -> gtk4::Widget {
    let text = i18n::text();
    let (root, column) = ui::page(text.appearance_title, text.appearance_description);
    column.append(&scheme::section());
    column.append(&wallpaper::section());
    column.append(&interface::section());
    root.upcast()
}

/// Run `program` with `args` and return what it printed, or why it failed. Blocking: call it off the main thread.
pub(crate) fn run(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("{program}: {error}"))?;
    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if message.is_empty() {
            format!("{program} exited with {}", output.status)
        } else {
            message
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
