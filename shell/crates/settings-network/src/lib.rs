//! The Network page of Flux Settings, and a compact quick menu for the status bar.
//!
//! [`model`] is pure data and rules, [`backend`] talks to NetworkManager over
//! D-Bus on its own thread, and `view` is GTK only.

pub mod backend;
pub mod i18n;
pub mod model;
mod ui;
mod view;

use std::cell::{Cell, RefCell};

use gtk4::prelude::*;

/// The page's id in the Settings navigation.
pub const ID: &str = "network";
/// The page's icon in the Settings navigation.
pub const ICON: &str = "network-wireless-symbolic";
/// The page's stylesheet, loaded once per display by [`page`] and [`quick_menu`].
pub const STYLE: &str = include_str!("../style.css");

thread_local! {
    static STYLED: Cell<bool> = const { Cell::new(false) };
}

/// The page's name in the user's language.
pub fn title() -> String {
    i18n::text().title.to_string()
}

/// The whole Network page, in a scrolled window.
pub fn page() -> gtk4::Widget {
    let content = mount(view::Mode::Full);
    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scroll.set_vexpand(true);
    scroll.set_child(Some(&content));
    scroll.upcast()
}

/// The Wi-Fi switch, the top networks and a "Network settings" link, for a
/// popover. The link activates `network.open-settings` when the host defines
/// that action, and runs `flux-settings network` otherwise.
pub fn quick_menu() -> gtk4::Widget {
    mount(view::Mode::Quick)
}

fn mount(mode: view::Mode) -> gtk4::Widget {
    style();
    let page = view::Page::new(mode);
    let widget = page.widget();
    let holder = RefCell::new(Some(page));
    widget.connect_destroy(move |_| {
        holder.borrow_mut().take();
    });
    widget
}

fn style() {
    if STYLED.with(|styled| styled.replace(true)) {
        return;
    }
    let Some(display) = gtk4::gdk::Display::default() else {
        return;
    };
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(STYLE);
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
    );
}
