//! The line that says which choices download from the internet, and the stop shown when a
//! choice needs the internet and the machine has none.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{glib, Align, Box, Orientation};

use super::{button, label};
use crate::app::App;
use crate::i18n::Text;
use crate::model::Download;
use crate::system;

fn names(text: &Text, downloads: &[Download]) -> String {
    downloads
        .iter()
        .map(|download| match download {
            Download::WithoutFacile => text.without_facile,
            Download::WithoutApps => text.without_apps,
            Download::Gaming => text.software_gaming,
            Download::Nvidia => text.software_nvidia,
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn watch(app: &Rc<App>, root: &Box, refresh: Rc<dyn Fn()>) {
    if app.state.borrow().preview {
        return;
    }
    let (app, root) = (Rc::downgrade(app), root.downgrade());
    glib::timeout_add_seconds_local(3, move || {
        let (Some(app), Some(_root)) = (app.upgrade(), root.upgrade()) else {
            return glib::ControlFlow::Break;
        };
        let online = system::online();
        let changed = app.state.borrow().online != online;
        if changed {
            app.state.borrow_mut().online = online;
            refresh();
        }
        glib::ControlFlow::Continue
    });
}

/// The notice for the choices made so far, and the function that redraws it after a choice
/// changes. `explain` adds the sentence on what the install medium carries. `blocked` hears
/// whether the install has to wait for the internet, now and every time the answer changes;
/// the connection is checked again every three seconds.
pub fn build(app: &Rc<App>, explain: bool, blocked: impl Fn(bool) + 'static) -> (Box, Rc<dyn Fn()>) {
    let text = app.text();
    app.state.borrow_mut().check_network();
    let root = Box::new(Orientation::Vertical, 8);
    let carried = label(text.network_carried, "note");
    let line = label("", "note");
    let stop = Box::new(Orientation::Horizontal, 16);
    stop.add_css_class("danger-panel");
    let warning = label("", "danger-body");
    warning.set_hexpand(true);
    let open = button("network-wireless-symbolic", text.network_settings, "secondary");
    open.set_valign(Align::Center);
    open.connect_clicked(|_| system::network_settings());
    stop.append(&warning);
    stop.append(&open);
    root.append(&carried);
    root.append(&line);
    root.append(&stop);
    let weak = Rc::downgrade(app);
    let refresh: Rc<dyn Fn()> = Rc::new(move || {
        let Some(app) = weak.upgrade() else {
            return;
        };
        let state = app.state.borrow();
        let list = names(text, &state.choices().software.downloads());
        let waits = state.waits_for_network();
        drop(state);
        let quiet = if list.is_empty() { text.network_none.to_string() } else { text.network_needed.replace("{list}", &list) };
        line.set_text(&quiet);
        line.set_visible(!waits);
        carried.set_visible(explain && !waits);
        warning.set_text(&text.network_offline.replace("{list}", &list));
        stop.set_visible(waits);
        blocked(waits);
    });
    refresh();
    watch(app, &root, refresh.clone());
    (root, refresh)
}
