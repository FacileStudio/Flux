//! Local copies of the shared settings helpers (`shell_common::ui`), same names and classes,
//! until the settings core lands and the lead swaps them in.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, DropDown, Image, Label, Orientation, Spinner, Switch, Widget};

/// A titled section: returns the whole section and the card that holds its rows.
pub fn section(title: &str) -> (GtkBox, GtkBox) {
    let root = GtkBox::new(Orientation::Vertical, 8);
    root.add_css_class("fx-section");
    let heading = Label::new(Some(title));
    heading.add_css_class("fx-section-title");
    heading.set_halign(Align::Start);
    heading.set_xalign(0.0);
    let card = GtkBox::new(Orientation::Vertical, 0);
    card.add_css_class("fx-card");
    root.append(&heading);
    root.append(&card);
    (root, card)
}

/// A row: title and optional subtitle on the left, an optional control on the right.
pub fn row(title: &str, subtitle: Option<&str>, suffix: Option<&Widget>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 16);
    row.add_css_class("fx-row");
    let text = GtkBox::new(Orientation::Vertical, 2);
    text.set_hexpand(true);
    text.set_valign(Align::Center);
    let label = Label::new(Some(title));
    label.add_css_class("fx-title");
    label.set_xalign(0.0);
    label.set_wrap(true);
    text.append(&label);
    if let Some(subtitle) = subtitle {
        let label = Label::new(Some(subtitle));
        label.add_css_class("fx-subtitle");
        label.set_xalign(0.0);
        label.set_wrap(true);
        text.append(&label);
    }
    row.append(&text);
    if let Some(suffix) = suffix {
        suffix.set_valign(Align::Center);
        row.append(suffix);
    }
    row
}

/// A row with a switch.
pub fn switch_row(title: &str, subtitle: Option<&str>) -> (GtkBox, Switch) {
    let switch = Switch::new();
    switch.update_property(&[gtk4::accessible::Property::Label(title)]);
    (row(title, subtitle, Some(switch.upcast_ref())), switch)
}

/// A row with a drop-down of choices.
pub fn choice_row(title: &str, subtitle: Option<&str>) -> (GtkBox, DropDown) {
    let choice = DropDown::from_strings(&[]);
    choice.update_property(&[gtk4::accessible::Property::Label(title)]);
    (row(title, subtitle, Some(choice.upcast_ref())), choice)
}

/// A tall, quiet block for a list with nothing in it or a service that is missing.
pub fn empty_state(icon: &str, title: &str, body: &str) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 16);
    root.add_css_class("fx-empty");
    root.set_valign(Align::Center);
    let image = Image::from_icon_name(icon);
    image.set_pixel_size(32);
    image.add_css_class("fx-subtitle");
    let words = GtkBox::new(Orientation::Vertical, 4);
    let heading = Label::new(Some(title));
    heading.add_css_class("fx-title");
    let text = Label::new(Some(body));
    text.add_css_class("fx-subtitle");
    text.set_wrap(true);
    text.set_justify(gtk4::Justification::Center);
    text.set_max_width_chars(48);
    words.append(&heading);
    words.append(&text);
    root.append(&image);
    root.append(&words);
    root
}

/// A row that says something is loading.
pub fn spinner_row(title: &str) -> GtkBox {
    let spinner = Spinner::new();
    spinner.start();
    row(title, None, Some(spinner.upcast_ref()))
}
