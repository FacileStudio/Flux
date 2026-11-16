//! Local copies of the `shell_common::ui` helpers the Settings pages share,
//! with the same names and CSS classes. They go away when the shared module
//! lands.

use gtk4::prelude::*;
use gtk4::{Align, Box, Image, Label, Orientation, Switch, Widget};

/// A label with `class`, aligned to the start and wrapping.
pub fn label(text: &str, class: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class(class);
    label.set_xalign(0.0);
    label.set_wrap(true);
    label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
    label
}

/// A titled group: the heading row, then a filled card the rows go into.
/// Returns the whole section and the card.
pub fn section(title: &str) -> (Box, Box, Box) {
    let section = Box::new(Orientation::Vertical, 8);
    section.add_css_class("fx-section");
    let header = Box::new(Orientation::Horizontal, 8);
    header.add_css_class("fx-section-header");
    let heading = label(title, "fx-section-title");
    heading.set_hexpand(true);
    header.append(&heading);
    let card = Box::new(Orientation::Vertical, 0);
    card.add_css_class("fx-card");
    section.append(&header);
    section.append(&card);
    (section, header, card)
}

/// Title and subtitle stacked, the subtitle hidden while it is empty.
pub fn texts(title: &str, subtitle: &str) -> (Box, Label, Label) {
    let column = Box::new(Orientation::Vertical, 2);
    column.set_valign(Align::Center);
    column.set_hexpand(true);
    let title = label(title, "fx-title");
    let subtitle = label(subtitle, "fx-subtitle");
    subtitle.set_visible(!subtitle.text().is_empty());
    column.append(&title);
    column.append(&subtitle);
    (column, title, subtitle)
}

/// A row: title, subtitle and a widget at the end.
pub fn row(title: &str, subtitle: &str, suffix: Option<&Widget>) -> (Box, Label, Label) {
    let row = Box::new(Orientation::Horizontal, 12);
    row.add_css_class("fx-row");
    let (column, title, subtitle) = texts(title, subtitle);
    row.append(&column);
    if let Some(suffix) = suffix {
        suffix.set_valign(Align::Center);
        row.append(suffix);
    }
    (row, title, subtitle)
}

/// A row ending in a switch.
pub fn switch_row(title: &str, subtitle: &str, active: bool) -> (Box, Switch, Label) {
    let switch = Switch::new();
    switch.set_active(active);
    let (row, _, subtitle) = row(title, subtitle, Some(switch.upcast_ref()));
    (row, switch, subtitle)
}

/// An icon, a title and a sentence, centred, for when there is nothing to list.
pub fn empty_state(icon: &str, title: &str, body: &str) -> (Box, Image, Label, Label) {
    let column = Box::new(Orientation::Vertical, 8);
    column.add_css_class("fx-empty");
    column.set_halign(Align::Center);
    column.set_valign(Align::Center);
    let image = Image::from_icon_name(icon);
    image.set_pixel_size(48);
    image.add_css_class("fx-empty-icon");
    let title = Label::new(Some(title));
    title.add_css_class("fx-empty-title");
    title.set_wrap(true);
    title.set_justify(gtk4::Justification::Center);
    let body = Label::new(Some(body));
    body.add_css_class("fx-subtitle");
    body.set_wrap(true);
    body.set_justify(gtk4::Justification::Center);
    body.set_max_width_chars(48);
    column.append(&image);
    column.append(&title);
    column.append(&body);
    (column, image, title, body)
}

/// Whether GTK's "enable animations" setting is on.
pub fn animations() -> bool {
    gtk4::Settings::default().is_none_or(|settings| settings.is_gtk_enable_animations())
}

/// `ms`, or zero when animations are off.
pub fn motion(ms: u32) -> u32 {
    if animations() {
        ms
    } else {
        0
    }
}
