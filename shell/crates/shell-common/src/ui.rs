//! The building blocks every Flux settings page is made of.
//!
//! These mirror muse's `Page`, `SettingsSection` and `SettingsRow`: a page is a
//! title, a description and a column of sections no wider than [`MAX_WIDTH`];
//! a section is a heading over a card; a row is a label and a muted line on the
//! left with its control on the right. Rows are separated by their own padding,
//! never by a line, and nothing here draws a border.
//!
//! The look lives in [`CSS`], which uses only `@fc_*` palette names and `.fx-*`
//! classes. An app loads it after the palette, before its own sheet:
//! `theme::init(&format!("{}{}", ui::CSS, OWN), ...)`.

use gtk4::prelude::*;
use gtk4::{Align, Orientation};

mod clamp;

pub use clamp::Clamp;

/// The widest a page's column of sections grows, in logical pixels.
pub const MAX_WIDTH: i32 = 720;

/// The stylesheet behind every `.fx-*` class, to load after the palette.
pub const CSS: &str = include_str!("ui/style.css");

/// A label with `class`, left-aligned and wrapping.
fn text(value: &str, class: &str) -> gtk4::Label {
    let label = gtk4::Label::new(Some(value));
    label.add_css_class(class);
    label.set_xalign(0.0);
    label.set_wrap(true);
    label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
    label
}

/// A page scaffold: returns the scrolling root to hand back as the page, and the column to append sections to.
pub fn page(title: &str, description: &str) -> (gtk4::ScrolledWindow, gtk4::Box) {
    let column = gtk4::Box::new(Orientation::Vertical, 32);
    column.add_css_class("fx-page");
    let header = gtk4::Box::new(Orientation::Vertical, 4);
    header.append(&text(title, "fx-page-title"));
    if !description.is_empty() {
        header.append(&text(description, "fx-page-description"));
    }
    column.append(&header);
    let clamp = Clamp::new(MAX_WIDTH);
    clamp.set_child(Some(&column));
    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);
    scroll.set_child(Some(&clamp));
    (scroll, column)
}

/// A titled section: returns the section to append to a page, and the card to append rows to. An empty title shows no heading.
pub fn section(title: &str) -> (gtk4::Box, gtk4::Box) {
    let root = gtk4::Box::new(Orientation::Vertical, 16);
    root.add_css_class("fx-section");
    if !title.is_empty() {
        root.append(&text(title, "fx-section-title"));
    }
    let card = gtk4::Box::new(Orientation::Vertical, 0);
    card.add_css_class("fx-card");
    root.append(&card);
    (root, card)
}

/// A settings row: `title` and an optional muted `subtitle` (empty hides it) on the left, `suffix` on the right.
pub fn row(title: &str, subtitle: &str, suffix: &impl IsA<gtk4::Widget>) -> gtk4::Box {
    let line = gtk4::Box::new(Orientation::Horizontal, 24);
    line.add_css_class("fx-row");
    let labels = gtk4::Box::new(Orientation::Vertical, 4);
    labels.set_hexpand(true);
    labels.set_valign(Align::Center);
    labels.append(&text(title, "fx-title"));
    if !subtitle.is_empty() {
        labels.append(&text(subtitle, "fx-subtitle"));
    }
    line.append(&labels);
    let suffix = suffix.as_ref();
    suffix.set_valign(Align::Center);
    suffix.set_halign(Align::End);
    line.append(suffix);
    line
}

/// A row with a switch set to `active`; `on_toggle` runs with the new state after every change.
pub fn switch_row(
    title: &str,
    subtitle: &str,
    active: bool,
    on_toggle: impl Fn(bool) + 'static,
) -> (gtk4::Box, gtk4::Switch) {
    let switch = gtk4::Switch::new();
    switch.set_active(active);
    switch.update_property(&[gtk4::accessible::Property::Label(title)]);
    switch.connect_active_notify(move |switch| on_toggle(switch.is_active()));
    (row(title, subtitle, &switch), switch)
}

/// A row with a slider from `min` to `max` in `step`s at `value`; `on_change` runs with each new value.
pub fn slider_row(
    title: &str,
    subtitle: &str,
    range: (f64, f64, f64),
    value: f64,
    on_change: impl Fn(f64) + 'static,
) -> (gtk4::Box, gtk4::Scale) {
    let (min, max, step) = range;
    let scale = gtk4::Scale::with_range(Orientation::Horizontal, min, max, step);
    scale.set_value(value);
    scale.set_draw_value(false);
    scale.set_width_request(220);
    scale.update_property(&[gtk4::accessible::Property::Label(title)]);
    scale.connect_value_changed(move |scale| on_change(scale.value()));
    (row(title, subtitle, &scale), scale)
}

/// A row with a drop-down of `options` showing `selected`; `on_change` runs with the chosen index.
pub fn choice_row(
    title: &str,
    subtitle: &str,
    options: &[&str],
    selected: usize,
    on_change: impl Fn(usize) + 'static,
) -> (gtk4::Box, gtk4::DropDown) {
    let choice = gtk4::DropDown::from_strings(options);
    choice.add_css_class("fx-choice");
    choice.set_selected(u32::try_from(selected).unwrap_or(0));
    choice.update_property(&[gtk4::accessible::Property::Label(title)]);
    choice.connect_selected_notify(move |choice| {
        if let Ok(index) = usize::try_from(choice.selected()) {
            on_change(index);
        }
    });
    (row(title, subtitle, &choice), choice)
}

/// A tall, quiet placeholder for a list with nothing in it: a muted icon, a title and a description (empty hides it).
pub fn empty_state(icon_name: &str, title: &str, description: &str) -> gtk4::Box {
    let column = gtk4::Box::new(Orientation::Vertical, 16);
    column.add_css_class("fx-empty");
    column.set_halign(Align::Fill);
    let glyph = icon(icon_name, 32);
    glyph.add_css_class("fx-empty-icon");
    column.append(&glyph);
    let labels = gtk4::Box::new(Orientation::Vertical, 4);
    let heading = text(title, "fx-title");
    heading.set_xalign(0.5);
    heading.set_justify(gtk4::Justification::Center);
    labels.append(&heading);
    if !description.is_empty() {
        let detail = text(description, "fx-subtitle");
        detail.set_xalign(0.5);
        detail.set_justify(gtk4::Justification::Center);
        detail.set_max_width_chars(48);
        labels.append(&detail);
    }
    column.append(&labels);
    column
}

/// A row that shows work in progress: a spinning indicator beside `title`.
pub fn spinner_row(title: &str) -> gtk4::Box {
    let line = gtk4::Box::new(Orientation::Horizontal, 12);
    line.add_css_class("fx-row");
    let spinner = gtk4::Spinner::new();
    spinner.start();
    line.append(&spinner);
    let label = text(title, "fx-subtitle");
    label.set_valign(Align::Center);
    line.append(&label);
    line
}

/// A button with `label` and the `.fx-button` plus `class` look.
fn button(label: &str, class: &str) -> gtk4::Button {
    let button = gtk4::Button::with_label(label);
    button.add_css_class("fx-button");
    button.add_css_class(class);
    button
}

/// The one main action of a view: inverted fill.
pub fn primary_button(label: &str) -> gtk4::Button {
    button(label, "fx-primary")
}

/// Any other action: a muted fill.
pub fn secondary_button(label: &str) -> gtk4::Button {
    button(label, "fx-secondary")
}

/// A destructive action: danger text on a faint danger fill.
pub fn danger_button(label: &str) -> gtk4::Button {
    button(label, "fx-danger")
}

/// A themed icon `name` drawn at `size` logical pixels.
pub fn icon(name: &str, size: i32) -> gtk4::Image {
    let image = gtk4::Image::from_icon_name(name);
    image.set_pixel_size(size);
    image
}
