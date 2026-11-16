//! Local copies of the `shell_common::ui` helpers the Settings pages share,
//! with the same names, until the settings core lands and replaces them.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, DropDown, Label, Orientation, Scale, Switch, Widget};

const STYLE: &str = include_str!("../style.css");

/// Load this crate's stylesheet once per display.
pub fn install_style() {
    thread_local! {
        static DONE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }
    if DONE.with(|done| done.replace(true)) {
        return;
    }
    let Some(display) = gtk4::gdk::Display::default() else {
        return;
    };
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(STYLE);
    gtk4::style_context_add_provider_for_display(&display, &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
}

/// The scrolling page every Settings page is built in, and the column its
/// sections go into.
pub fn page() -> (gtk4::ScrolledWindow, GtkBox) {
    install_style();
    let column = GtkBox::new(Orientation::Vertical, 32);
    column.add_css_class("fx-page");
    let scroller = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .hexpand(true)
        .child(&column)
        .build();
    (scroller, column)
}

/// A titled group of rows: append rows to `rows`, place `root` on the page.
pub struct Section {
    pub root: GtkBox,
    pub rows: GtkBox,
}

pub fn section(title: &str) -> Section {
    let root = GtkBox::new(Orientation::Vertical, 12);
    root.add_css_class("fx-section");
    let heading = Label::builder().label(title).xalign(0.0).css_classes(["fx-heading"]).build();
    let rows = GtkBox::new(Orientation::Vertical, 0);
    rows.add_css_class("fx-card");
    root.append(&heading);
    root.append(&rows);
    Section { root, rows }
}

/// A row: title and optional subtitle on the left, `suffix` on the right.
pub fn row(title: &str, subtitle: Option<&str>, suffix: &impl IsA<Widget>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 16);
    row.add_css_class("fx-row");
    let text = GtkBox::new(Orientation::Vertical, 2);
    text.set_hexpand(true);
    text.set_valign(Align::Center);
    text.append(&Label::builder().label(title).xalign(0.0).wrap(true).css_classes(["fx-title"]).build());
    if let Some(subtitle) = subtitle {
        text.append(&Label::builder().label(subtitle).xalign(0.0).wrap(true).css_classes(["fx-subtitle"]).build());
    }
    suffix.set_valign(Align::Center);
    row.append(&text);
    row.append(suffix);
    row
}

pub fn switch_row(title: &str, subtitle: Option<&str>, active: bool) -> (GtkBox, Switch) {
    let switch = Switch::builder().active(active).build();
    switch.update_property(&[gtk4::accessible::Property::Label(title)]);
    (row(title, subtitle, &switch), switch)
}

pub fn slider_row(title: &str, subtitle: Option<&str>, range: (f64, f64, f64), value: f64) -> (GtkBox, Scale) {
    let (min, max, step) = range;
    let scale = Scale::with_range(Orientation::Horizontal, min, max, step);
    scale.set_value(value);
    scale.set_width_request(220);
    scale.set_draw_value(false);
    scale.update_property(&[gtk4::accessible::Property::Label(title)]);
    (row(title, subtitle, &scale), scale)
}

pub fn choice_row(title: &str, subtitle: Option<&str>, items: &[String], selected: usize) -> (GtkBox, DropDown) {
    let names: Vec<&str> = items.iter().map(String::as_str).collect();
    let dropdown = DropDown::from_strings(&names);
    if selected < items.len() {
        dropdown.set_selected(selected as u32);
    }
    dropdown.update_property(&[gtk4::accessible::Property::Label(title)]);
    (row(title, subtitle, &dropdown), dropdown)
}

/// A centred message for when there is nothing to show.
pub fn empty_state(icon: &str, title: &str, body: &str) -> GtkBox {
    let state = GtkBox::new(Orientation::Vertical, 8);
    state.add_css_class("fx-empty");
    state.set_valign(Align::Center);
    state.set_halign(Align::Center);
    let image = gtk4::Image::from_icon_name(icon);
    image.set_pixel_size(48);
    image.add_css_class("fx-subtitle");
    state.append(&image);
    state.append(&Label::builder().label(title).css_classes(["fx-heading"]).build());
    state.append(&Label::builder().label(body).wrap(true).justify(gtk4::Justification::Center).max_width_chars(48).css_classes(["fx-subtitle"]).build());
    state
}

/// A row that says something is loading.
pub fn spinner_row(title: &str) -> GtkBox {
    let spinner = gtk4::Spinner::new();
    spinner.start();
    row(title, None, &spinner)
}

/// A filled button in the muse style: `primary`, `ghost` or `danger`.
pub fn button(label: &str, kind: &str) -> gtk4::Button {
    gtk4::Button::builder().label(label).css_classes(["fx-button", kind]).build()
}

/// Remove every child of `container`.
pub fn clear(container: &GtkBox) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}
