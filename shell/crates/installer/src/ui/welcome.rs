use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Orientation, Widget};

use super::{hero, label};
use crate::app::{App, Screen};

fn choice(glyph: &str, title: &str, body: &str, class: &str) -> Button {
    let content = Box::new(Orientation::Vertical, 8);
    content.set_valign(Align::Start);
    let image = shell_common::ui::icon(glyph, 32);
    image.add_css_class("choice-icon");
    image.set_halign(Align::Start);
    content.append(&image);
    content.append(&label(title, "choice-title"));
    content.append(&label(body, "choice-body"));
    let button = Button::new();
    button.add_css_class("choice");
    button.add_css_class(class);
    button.set_hexpand(true);
    button.set_child(Some(&content));
    button
}

pub fn build(app: &Rc<App>) -> Widget {
    let text = app.text();
    let choices = Box::new(Orientation::Horizontal, 16);
    choices.set_homogeneous(true);
    let try_flux = choice("media-removable-symbolic", text.try_title, text.try_body, "secondary");
    let install = choice("drive-harddisk-symbolic", text.install_title, text.install_body, "primary");
    let window = app.window.clone();
    try_flux.connect_clicked(move |_| window.close());
    let weak = Rc::downgrade(app);
    install.connect_clicked(move |_| {
        if let Some(app) = weak.upgrade() {
            app.go(Screen::Language);
        }
    });
    choices.append(&try_flux);
    choices.append(&install);
    super::default(app, &install);
    let focus = install.clone();
    gtk4::glib::idle_add_local_once(move || {
        focus.grab_focus();
    });
    hero(text.welcome_title, text.welcome_subtitle, &choices)
}
