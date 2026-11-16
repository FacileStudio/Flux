//! The quick controls above the notifications: today, the light/dark toggle.
//!
//! The toggle runs `flux-scheme toggle`, which flips the desktop colour
//! scheme. The portal then reports the change and every widget, this one
//! included, restyles itself; the button follows through the theme's reload
//! hook rather than guessing the outcome of its own click.

use gtk4::prelude::*;
use gtk4::{glib, Box, Button, Image, Label};
use shell_common::scheme::{self, Scheme};

/// The icon and label offered for switching away from `scheme`.
fn offer(scheme: Option<Scheme>) -> (&'static str, &'static str) {
    match scheme {
        Some(Scheme::Dark) => ("weather-clear-symbolic", "Light mode"),
        _ => ("weather-clear-night-symbolic", "Dark mode"),
    }
}

/// The quick-controls row.
pub(super) fn controls() -> Box {
    let row = Box::new(gtk4::Orientation::Horizontal, 8);
    row.add_css_class("controls");

    let icon = Image::new();
    let label = Label::new(None);
    let content = Box::new(gtk4::Orientation::Horizontal, 8);
    content.append(&icon);
    content.append(&label);

    let toggle = Button::new();
    toggle.set_child(Some(&content));
    toggle.add_css_class("control");
    toggle.add_css_class("scheme");
    toggle.connect_clicked(|_| {
        if let Err(error) = glib::spawn_command_line_async("flux-scheme toggle") {
            eprintln!("sidebar: flux-scheme: {error}");
        }
    });

    let show = move || {
        let (icon_name, text) = offer(scheme::current());
        icon.set_icon_name(Some(icon_name));
        label.set_text(text);
    };
    show();
    shell_common::theme::on_reload(show);

    row.append(&toggle);
    row
}
