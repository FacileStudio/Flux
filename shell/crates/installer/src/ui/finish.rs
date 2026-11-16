use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box, Orientation, Widget};

use super::{button, hero, label, page};
use crate::app::App;
use crate::system;

pub fn error(app: &Rc<App>) -> Widget {
    let text = app.text();
    let panel = Box::new(Orientation::Vertical, 8);
    panel.add_css_class("danger-panel");
    panel.set_valign(gtk4::Align::Start);
    panel.append(&label(&app.state.borrow().error, "danger-body"));
    let copy = button(text.copy_log, "primary");
    let copied = text.copied;
    copy.connect_clicked(move |copy| {
        copy.clipboard().set_text(&system::log());
        copy.set_label(copied);
    });
    super::default(app, &copy);
    page(app, text.error_title, text.error_subtitle, &panel, &[&copy])
}

pub fn done(app: &Rc<App>) -> Widget {
    let text = app.text();
    let actions = Box::new(Orientation::Horizontal, 8);
    let restart = button(text.restart, "primary");
    restart.connect_clicked(|_| system::reboot());
    let keep = button(text.close, "ghost");
    let window = app.window.clone();
    keep.connect_clicked(move |_| window.close());
    actions.append(&restart);
    actions.append(&keep);
    super::default(app, &restart);
    let focus = restart.clone();
    gtk4::glib::idle_add_local_once(move || {
        focus.grab_focus();
    });
    hero(text.done_title, text.done_subtitle, &actions)
}
