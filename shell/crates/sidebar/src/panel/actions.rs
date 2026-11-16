//! What the panel does: drop a notification, empty itself, keep its relative
//! times current, and show or hide with the shared slide and fade.

use std::rc::Rc;

use gtk4::prelude::*;
use shell_common::motion;

use crate::wallpaper::picker::Picker;
use crate::{dbus, model, text};

use super::App;

impl App {
    /// Drop one notification, from the history file, the list and any toast.
    pub(super) fn dismiss(app: &Rc<App>, id: u32) {
        app.items.borrow_mut().retain(|item| item.id != id);
        model::save(&app.items.borrow());
        App::rebuild(app);
        app.toasts.dismiss(id);
        dbus::emit_closed(app.handle.as_ref(), id, 2);
    }

    /// Empty the panel, telling each client the notification was closed.
    pub(super) fn clear_all(app: &Rc<App>) {
        let ids: Vec<u32> = app.items.borrow().iter().map(|item| item.id).collect();
        app.items.borrow_mut().clear();
        model::save(&app.items.borrow());
        App::rebuild(app);
        for id in ids {
            app.toasts.dismiss(id);
            dbus::emit_closed(app.handle.as_ref(), id, 2);
        }
    }

    pub(super) fn refresh_times(&self) {
        let now = model::now();
        for (label, ts) in self.time_labels.borrow().iter() {
            label.set_text(&model::relative_time(*ts, now, text::get().just_now));
        }
    }

    /// Whether the panel is on screen and not on its way out.
    fn shown(&self) -> bool {
        (self.window.is_visible() && self.root.has_css_class("fx-shown"))
            || self.root.has_css_class("fx-pending")
    }

    pub(crate) fn toggle(&self) {
        if self.shown() {
            self.hide();
        } else {
            self.show();
        }
    }

    pub(crate) fn show(&self) {
        Picker::reload(&self.picker);
        self.refresh_times();
        self.window.present();
        motion::reveal(&self.root);
    }

    pub(crate) fn hide(&self) {
        let window = self.window.clone();
        motion::conceal(&self.root, move || window.set_visible(false));
    }
}
