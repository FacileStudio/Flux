use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box, Orientation, Widget};

use super::{button, label, network, page};
use crate::app::{App, Screen};
use crate::model::{keymap_name, Disk, LANGUAGES};

fn line(name: &str, value: &str, class: &str) -> Box {
    let row = Box::new(Orientation::Horizontal, 16);
    row.add_css_class("summary-row");
    let key = label(name, "dim");
    key.set_width_chars(18);
    key.set_xalign(0.0);
    row.append(&key);
    let value = label(value, class);
    value.set_hexpand(true);
    row.append(&value);
    row
}

pub fn build(app: &Rc<App>) -> Widget {
    let text = app.text();
    let state = app.state.borrow();
    let panel = Box::new(Orientation::Vertical, 0);
    panel.add_css_class("panel");
    panel.add_css_class("summary");
    panel.set_valign(gtk4::Align::Start);
    let disk = state.disk.as_ref().map(Disk::label).unwrap_or_default();
    let account = format!("{} ({})", state.full_name.trim(), state.username);
    panel.append(&line(text.summary_language, LANGUAGES[state.language].name, "value"));
    panel.append(&line(text.summary_keyboard, keymap_name(&state.keymap), "value"));
    panel.append(&line(text.summary_disk, &disk, "value-danger"));
    panel.append(&line(text.account_title, &account, "value"));
    panel.append(&line(text.hostname, &state.hostname, "value"));
    panel.append(&line(text.summary_timezone, &super::timezone::readable(&state.timezone), "value"));
    let choices = state.choices().software;
    let picked: Vec<&str> = [
        (choices.facile, text.software_facile),
        (choices.apps, text.software_apps),
        (choices.gaming, text.software_gaming),
        (choices.nvidia, text.software_nvidia),
    ]
    .into_iter()
    .filter_map(|(on, name)| on.then_some(name))
    .collect();
    let software = if picked.is_empty() { text.summary_minimal.to_string() } else { picked.join(", ") };
    panel.append(&line(text.summary_software, &software, "value"));
    drop(state);
    let install = button("drive-harddisk-symbolic", text.install, "primary");
    let ready = app.state.borrow().ready();
    let gate = install.clone();
    let (notice, _) = network::build(app, false, move |waits| gate.set_sensitive(ready && !waits));
    let body = Box::new(Orientation::Vertical, 12);
    body.append(&panel);
    body.append(&notice);
    let weak = Rc::downgrade(app);
    install.connect_clicked(move |_| {
        if let Some(app) = weak.upgrade() {
            app.go(Screen::Progress);
        }
    });
    super::default(app, &install);
    page(app, text.summary_title, text.summary_subtitle, &body, &[&install])
}
