//! The Power page of Flux Settings: battery, power mode, and the idle, lock
//! and sleep rules that `flux-idle` turns into a swayidle command line.
//!
//! The page writes two files:
//!
//! - `~/.config/flux/power.json`, the idle rules ([`config::PowerConfig`]),
//!   then restarts `flux-idle.service` so they apply;
//! - the `battery_percentage` boolean in `~/.config/flux/shell.json`, which
//!   the statusbar reads to print the percentage next to its battery icon.

use std::cell::Cell;

pub mod config;
pub mod format;
pub mod i18n;
pub mod idle;
mod modes;
mod page;
mod quick;
mod source;
mod ui;
pub mod upower;

pub const ID: &str = "power";
pub const ICON: &str = "battery-level-80-symbolic";

thread_local! {
    static SAMPLE: Cell<Option<bool>> = const { Cell::new(None) };
}

/// The page's name in the navigation.
pub fn title() -> String {
    i18n::text().title.to_string()
}

/// Show sample data instead of the real services from now on, and write
/// nothing: `laptop` picks a machine with a battery or a desktop. For previews
/// and screenshots.
pub fn use_sample_data(language: &str, laptop: bool) {
    i18n::force(language);
    SAMPLE.with(|sample| sample.set(Some(laptop)));
}

fn source() -> source::Source {
    match SAMPLE.with(Cell::get) {
        Some(laptop) => source::Source::sample(laptop),
        None => source::Source::live(),
    }
}

/// The whole page.
pub fn page() -> gtk4::Widget {
    page::build(i18n::text(), source())
}

/// The power quick menu for the statusbar: battery line and mode control.
pub fn quick_menu() -> gtk4::Widget {
    quick::build(i18n::text(), source())
}
