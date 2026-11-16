//! The Bluetooth page of Flux Settings, over BlueZ's D-Bus API: power, the
//! paired and nearby devices, pairing through our own agent, and a quick menu
//! for the statusbar.
//!
//! `model` is plain data and logic, `backend` owns the bus on a worker thread,
//! `client` carries events to the GTK thread, and `view` draws them.

pub mod backend;
mod client;
mod i18n;
pub mod model;
mod preview;
mod ui;
mod view;

/// The page's key in the Settings navigation and on the command line.
pub const ID: &str = "bluetooth";

/// The page's icon in the Settings navigation.
pub const ICON: &str = "bluetooth-active-symbolic";

/// The page's title in the session's language.
pub fn title() -> String {
    i18n::text().title.to_string()
}

/// The whole Bluetooth page.
pub fn page() -> gtk4::Widget {
    view::page()
}

/// The compact menu for a statusbar popover.
pub fn quick_menu() -> gtk4::Widget {
    view::quick::quick_menu()
}
