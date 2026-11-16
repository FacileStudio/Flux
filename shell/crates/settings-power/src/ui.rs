//! Local copies of the `shell_common::ui` helpers the settings pages share,
//! with the same names and classes, until the settings core lands them.

use std::cell::Cell;

use gtk4::prelude::*;
use gtk4::{Align, Orientation};

/// The look of the helpers below, in muse: fills instead of borders, 12px on
/// surfaces and 8px on controls, 150 ms ease-out on state changes.
pub const CSS: &str = r#"
.fx-page { padding: 32px 24px; }
.fx-page-title { font-size: 28px; font-weight: 600; letter-spacing: -0.02em; color: @fc_fg; }
.fx-section { background-color: @fc_component; border-radius: 12px; padding: 4px 20px; }
.fx-section-title { font-size: 18px; font-weight: 600; color: @fc_fg; letter-spacing: -0.02em; }
.fx-section-body { font-size: 14px; color: @fc_fg_muted; }
.fx-row { padding: 16px 0; min-height: 32px; }
.fx-title { font-size: 14px; font-weight: 500; color: @fc_fg; }
.fx-subtitle { font-size: 12px; color: @fc_fg_muted; }
.fx-danger { color: @fc_danger; }
.fx-empty { padding: 48px 20px; }
.fx-empty image { color: @fc_fg_muted; }
.fx-segmented { background-color: @fc_surface; border-radius: 8px; padding: 4px; }
.fx-segmented button { background-color: transparent; padding: 6px 14px; border-radius: 6px; transition: background-color 150ms ease-out, color 150ms ease-out; }
.fx-segmented button:hover { background-color: alpha(@fc_fg, 0.06); }
.fx-segmented button:checked { background-color: @fc_accent; color: @fc_accent_fg; }
dropdown > button { background-color: @fc_surface; padding: 6px 12px; border-radius: 8px; transition: background-color 150ms ease-out; }
dropdown > button:hover { background-color: mix(@fc_surface, @fc_fg, 0.08); }
dropdown popover > contents { background-color: @fc_component; border-radius: 12px; padding: 4px; }
dropdown popover row { border-radius: 8px; padding: 6px 10px; }
dropdown popover row:hover, dropdown popover row:selected { background-color: @fc_surface; }
switch { background-color: @fc_surface; border-radius: 999px; transition: background-color 150ms ease-out; }
switch:checked { background-color: @fc_accent; }
switch slider { background-color: @fc_page; border-radius: 999px; min-width: 20px; min-height: 20px; }
levelbar trough { background-color: @fc_surface; border-radius: 999px; min-height: 6px; }
levelbar block.filled { background-color: @fc_fg; border-radius: 999px; }
levelbar block.empty { background-color: transparent; }
"#;

thread_local! {
    static STYLED: Cell<bool> = const { Cell::new(false) };
}

/// Register [`CSS`] once per display.
pub fn style() {
    if STYLED.with(|styled| styled.replace(true)) {
        return;
    }
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(CSS);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(&display, &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1);
    }
}

fn label(text: &str, class: &str) -> gtk4::Label {
    let label = gtk4::Label::new(Some(text));
    label.add_css_class(class);
    label.set_xalign(0.0);
    label.set_wrap(true);
    label
}

/// A section: a heading, an optional line under it, and a filled card the
/// rows go in. Returns the whole block and the card.
pub fn section(title: &str, body: Option<&str>) -> (gtk4::Box, gtk4::Box) {
    let block = gtk4::Box::new(Orientation::Vertical, 12);
    let heading = gtk4::Box::new(Orientation::Vertical, 4);
    heading.append(&label(title, "fx-section-title"));
    if let Some(body) = body {
        heading.append(&label(body, "fx-section-body"));
    }
    block.append(&heading);
    let card = gtk4::Box::new(Orientation::Vertical, 0);
    card.add_css_class("fx-section");
    block.append(&card);
    (block, card)
}

/// A row: title and optional subtitle on the left, `suffix` on the right.
/// Returns the row and its subtitle label, so callers can update it.
pub fn row(title: &str, subtitle: Option<&str>, suffix: Option<&gtk4::Widget>) -> (gtk4::Box, gtk4::Label) {
    let row = gtk4::Box::new(Orientation::Horizontal, 16);
    row.add_css_class("fx-row");
    let text = gtk4::Box::new(Orientation::Vertical, 2);
    text.set_hexpand(true);
    text.set_valign(Align::Center);
    let title = label(title, "fx-title");
    text.append(&title);
    let subtitle_label = label(subtitle.unwrap_or(""), "fx-subtitle");
    subtitle_label.set_visible(subtitle.is_some());
    text.append(&subtitle_label);
    row.append(&text);
    if let Some(suffix) = suffix {
        suffix.set_valign(Align::Center);
        row.append(suffix);
    }
    (row, subtitle_label)
}

/// A row with a switch.
pub fn switch_row(title: &str, subtitle: Option<&str>, active: bool) -> (gtk4::Box, gtk4::Switch) {
    let switch = gtk4::Switch::new();
    switch.set_active(active);
    let (row, _) = row(title, subtitle, Some(switch.upcast_ref()));
    (row, switch)
}

/// A row with a drop-down of `choices`, `selected` first shown.
pub fn choice_row(title: &str, subtitle: Option<&str>, choices: &[String], selected: u32) -> (gtk4::Box, gtk4::DropDown) {
    let names: Vec<&str> = choices.iter().map(String::as_str).collect();
    let dropdown = gtk4::DropDown::from_strings(&names);
    dropdown.set_selected(selected);
    let (row, _) = row(title, subtitle, Some(dropdown.upcast_ref()));
    (row, dropdown)
}

/// A tall, quiet card for a list with nothing in it.
pub fn empty_state(icon: &str, title: &str, body: &str) -> gtk4::Box {
    let empty = gtk4::Box::new(Orientation::Vertical, 4);
    empty.add_css_class("fx-empty");
    empty.set_halign(Align::Center);
    let image = gtk4::Image::from_icon_name(icon);
    image.set_pixel_size(32);
    image.set_margin_bottom(12);
    empty.append(&image);
    let title = label(title, "fx-title");
    title.set_xalign(0.5);
    empty.append(&title);
    let body = label(body, "fx-subtitle");
    body.set_xalign(0.5);
    empty.append(&body);
    empty
}
