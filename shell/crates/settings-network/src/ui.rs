//! Local copies of the `shell_common::ui` helpers this page uses, with the same
//! names and classes, until settings-core ships them. The lead swaps this file
//! for the shared module at merge.

use gtk4::prelude::*;
use gtk4::{Align, Box, Label, Orientation, Switch, Widget};

/// Whether GTK animations are on; with them off every transition is instant.
pub fn animations() -> bool {
    gtk4::Settings::default().is_none_or(|settings| settings.is_gtk_enable_animations())
}

/// The reveal and collapse duration in milliseconds, zero without animations.
pub fn motion_ms() -> u32 {
    if animations() {
        160
    } else {
        0
    }
}

/// A label with `class`, aligned to the start.
pub fn label(text: &str, class: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class(class);
    label.set_xalign(0.0);
    label.set_wrap(true);
    label
}

/// A titled card that rows are appended to.
pub fn section(title: &str) -> Box {
    let section = Box::new(Orientation::Vertical, 0);
    section.add_css_class("fx-section");
    if !title.is_empty() {
        section.append(&label(title, "fx-section-title"));
    }
    section
}

/// Title and subtitle on the left, `suffix` on the right.
pub fn row(title: &str, subtitle: Option<&str>, suffix: Option<&Widget>) -> Box {
    let row = Box::new(Orientation::Horizontal, 12);
    row.add_css_class("fx-row");
    let text = Box::new(Orientation::Vertical, 2);
    text.set_hexpand(true);
    text.set_valign(Align::Center);
    text.append(&label(title, "fx-title"));
    if let Some(subtitle) = subtitle {
        text.append(&label(subtitle, "fx-subtitle"));
    }
    row.append(&text);
    if let Some(suffix) = suffix {
        suffix.set_valign(Align::Center);
        row.append(suffix);
    }
    row
}

/// A row with a switch, which the caller wires up.
pub fn switch_row(title: &str, subtitle: Option<&str>, active: bool) -> (Box, Switch) {
    let switch = Switch::new();
    switch.set_active(active);
    switch.update_property(&[gtk4::accessible::Property::Label(title)]);
    (row(title, subtitle, Some(switch.upcast_ref())), switch)
}

/// A tall, quiet message where a list would be.
pub fn empty_state(title: &str, body: &str) -> Box {
    let state = Box::new(Orientation::Vertical, 4);
    state.add_css_class("fx-empty");
    state.set_valign(Align::Center);
    let heading = label(title, "fx-title");
    heading.set_xalign(0.5);
    heading.set_justify(gtk4::Justification::Center);
    state.append(&heading);
    if !body.is_empty() {
        let text = label(body, "fx-subtitle");
        text.set_xalign(0.5);
        text.set_justify(gtk4::Justification::Center);
        text.set_max_width_chars(48);
        state.append(&text);
    }
    state
}

/// A spinner beside a muted line.
pub fn spinner_row(title: &str) -> Box {
    let spinner = gtk4::Spinner::new();
    spinner.start();
    let row = Box::new(Orientation::Horizontal, 8);
    row.add_css_class("fx-row");
    row.append(&spinner);
    row.append(&label(title, "fx-subtitle"));
    row
}
