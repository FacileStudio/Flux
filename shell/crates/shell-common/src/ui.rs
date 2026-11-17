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
//! `theme::init(&format!("{}{}", ui::CSS, OWN), ...)`. A page crate adds only
//! rules scoped under its own root class, through [`page_style`].

use std::cell::RefCell;
use std::collections::HashSet;

use gtk4::prelude::*;
use gtk4::{Align, Orientation};

mod clamp;

pub use clamp::Clamp;

/// The widest a page's column of sections grows, in logical pixels.
pub const MAX_WIDTH: i32 = 720;

/// How long a reveal, a collapse or a page change takes, in milliseconds.
pub const MOTION_MS: u32 = 160;

/// The stylesheet behind every `.fx-*` class, to load after the palette.
pub const CSS: &str = include_str!("ui/style.css");

thread_local! {
    static STYLED: RefCell<HashSet<usize>> = RefCell::new(HashSet::new());
}

/// Load a page crate's own `css` once per process, above the shared sheet. Every rule in it must be scoped under the page's root class.
pub fn page_style(css: &'static str) {
    let key = css.as_ptr() as usize;
    if !STYLED.with(|styled| styled.borrow_mut().insert(key)) {
        return;
    }
    let Some(display) = gtk4::gdk::Display::default() else {
        return;
    };
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(css);
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
    );
}

/// Whether GTK's "enable animations" setting is on.
pub fn animations() -> bool {
    gtk4::Settings::default().is_none_or(|settings| settings.is_gtk_enable_animations())
}

/// `ms`, or zero when animations are off, for durations set from code.
pub fn motion(ms: u32) -> u32 {
    if animations() {
        ms
    } else {
        0
    }
}

/// A label with `class` (empty adds none), left-aligned and wrapping.
pub fn label(value: &str, class: &str) -> gtk4::Label {
    let label = gtk4::Label::new(Some(value));
    if !class.is_empty() {
        label.add_css_class(class);
    }
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
    header.append(&label(title, "fx-page-title"));
    if !description.is_empty() {
        header.append(&label(description, "fx-page-description"));
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

/// A section's parts: `root` goes on the page, `header` holds the title and any action beside it, rows go in `card`.
pub struct Section {
    pub root: gtk4::Box,
    pub header: gtk4::Box,
    pub card: gtk4::Box,
}

/// A titled section with an optional muted `description` (empty hides it) under its title, and room in the header for an action. An empty title hides the header.
pub fn section_with(title: &str, description: &str) -> Section {
    let root = gtk4::Box::new(Orientation::Vertical, 16);
    root.add_css_class("fx-section");
    let header = gtk4::Box::new(Orientation::Horizontal, 12);
    header.add_css_class("fx-section-header");
    let heading = gtk4::Box::new(Orientation::Vertical, 4);
    heading.set_hexpand(true);
    heading.set_valign(Align::Center);
    heading.append(&label(title, "fx-section-title"));
    if !description.is_empty() {
        heading.append(&label(description, "fx-section-description"));
    }
    header.append(&heading);
    header.set_visible(!title.is_empty());
    root.append(&header);
    let card = gtk4::Box::new(Orientation::Vertical, 0);
    card.add_css_class("fx-card");
    root.append(&card);
    Section { root, header, card }
}

/// A titled section: returns the section to append to a page, and the card to append rows to. An empty title shows no heading.
pub fn section(title: &str) -> (gtk4::Box, gtk4::Box) {
    let parts = section_with(title, "");
    (parts.root, parts.card)
}

/// A row's parts: the row itself and its two labels, so a page can update them in place.
pub struct Row {
    pub root: gtk4::Box,
    pub title: gtk4::Label,
    pub subtitle: gtk4::Label,
}

/// A settings row with an optional `suffix` on the right. The subtitle line is hidden while its text is empty.
pub fn row_with(title: &str, subtitle: &str, suffix: Option<&gtk4::Widget>) -> Row {
    let line = gtk4::Box::new(Orientation::Horizontal, 24);
    line.add_css_class("fx-row");
    let (labels, title, subtitle) = texts(title, subtitle);
    line.append(&labels);
    if let Some(suffix) = suffix {
        suffix.set_valign(Align::Center);
        suffix.set_halign(Align::End);
        line.append(suffix);
    }
    Row {
        root: line,
        title,
        subtitle,
    }
}

/// A row's text column: the title over a muted subtitle that is hidden while empty. It expands to push anything after it to the end.
pub fn texts(title: &str, subtitle: &str) -> (gtk4::Box, gtk4::Label, gtk4::Label) {
    let labels = gtk4::Box::new(Orientation::Vertical, 4);
    labels.set_hexpand(true);
    labels.set_valign(Align::Center);
    let title = label(title, "fx-title");
    labels.append(&title);
    let subtitle = label(subtitle, "fx-subtitle");
    subtitle.set_visible(!subtitle.text().is_empty());
    labels.append(&subtitle);
    (labels, title, subtitle)
}

/// A settings row: `title` and an optional muted `subtitle` (empty hides it) on the left, `suffix` on the right.
pub fn row(title: &str, subtitle: &str, suffix: &impl IsA<gtk4::Widget>) -> gtk4::Box {
    row_with(title, subtitle, Some(suffix.as_ref())).root
}

/// Set a row subtitle's text and hide the line while it is empty.
pub fn set_subtitle(subtitle: &gtk4::Label, value: &str) {
    subtitle.set_text(value);
    subtitle.set_visible(!value.is_empty());
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
pub fn choice_row<S: AsRef<str>>(
    title: &str,
    subtitle: &str,
    options: &[S],
    selected: usize,
    on_change: impl Fn(usize) + 'static,
) -> (gtk4::Box, gtk4::DropDown) {
    let names: Vec<&str> = options.iter().map(AsRef::as_ref).collect();
    let choice = gtk4::DropDown::from_strings(&names);
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

/// Replace the options of a [`choice_row`] drop-down and select `selected`.
pub fn set_choices<S: AsRef<str>>(choice: &gtk4::DropDown, options: &[S], selected: usize) {
    let names: Vec<&str> = options.iter().map(AsRef::as_ref).collect();
    choice.set_model(Some(&gtk4::StringList::new(&names)));
    choice.set_selected(u32::try_from(selected).unwrap_or(0));
}

/// A tall, quiet placeholder for a list with nothing in it: a muted icon, a title and a description (empty hides it).
pub fn empty_state(icon_name: &str, title: &str, description: &str) -> gtk4::Box {
    empty_state_with(icon_name, title, description).0
}

/// [`empty_state`] with its icon, title and description, so a page can change them in place.
pub fn empty_state_with(
    icon_name: &str,
    title: &str,
    description: &str,
) -> (gtk4::Box, gtk4::Image, gtk4::Label, gtk4::Label) {
    let column = gtk4::Box::new(Orientation::Vertical, 16);
    column.add_css_class("fx-empty");
    column.set_halign(Align::Fill);
    column.set_valign(Align::Center);
    let glyph = icon(icon_name, 32);
    glyph.add_css_class("fx-empty-icon");
    glyph.set_visible(!icon_name.is_empty());
    column.append(&glyph);
    let labels = gtk4::Box::new(Orientation::Vertical, 4);
    let heading = label(title, "fx-title");
    heading.set_xalign(0.5);
    heading.set_justify(gtk4::Justification::Center);
    labels.append(&heading);
    let detail = label(description, "fx-subtitle");
    detail.set_xalign(0.5);
    detail.set_justify(gtk4::Justification::Center);
    detail.set_max_width_chars(48);
    detail.set_visible(!description.is_empty());
    labels.append(&detail);
    column.append(&labels);
    (column, glyph, heading, detail)
}

/// A row that shows work in progress: a spinning indicator beside `title`.
pub fn spinner_row(title: &str) -> gtk4::Box {
    let line = gtk4::Box::new(Orientation::Horizontal, 12);
    line.add_css_class("fx-row");
    let spinner = gtk4::Spinner::new();
    spinner.start();
    line.append(&spinner);
    let text = label(title, "fx-subtitle");
    text.set_valign(Align::Center);
    text.set_hexpand(true);
    line.append(&text);
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

/// A quiet action with no fill until hovered.
pub fn ghost_button(label: &str) -> gtk4::Button {
    button(label, "fx-ghost")
}

/// A square, fill-less button showing the themed icon `name`, named `tooltip` on hover and for screen readers.
pub fn icon_button(name: &str, tooltip: &str) -> gtk4::Button {
    let button = gtk4::Button::from_icon_name(name);
    button.add_css_class("fx-icon-button");
    button.set_tooltip_text(Some(tooltip));
    button.update_property(&[gtk4::accessible::Property::Label(tooltip)]);
    button
}

/// A row of buttons pushed to the end of its line.
pub fn actions(buttons: &[&gtk4::Button]) -> gtk4::Box {
    let line = gtk4::Box::new(Orientation::Horizontal, 8);
    line.add_css_class("fx-actions");
    line.set_halign(Align::End);
    for button in buttons {
        line.append(*button);
    }
    line
}

/// A labelled input with an error line under it: returns the field and the error label for [`set_error`].
pub fn field(title: &str, input: &impl IsA<gtk4::Widget>) -> (gtk4::Box, gtk4::Label) {
    let column = gtk4::Box::new(Orientation::Vertical, 6);
    column.add_css_class("fx-field");
    column.set_hexpand(true);
    column.append(&label(title, "fx-field-label"));
    column.append(input);
    let error = label("", "fx-field-error");
    error.set_visible(false);
    column.append(&error);
    (column, error)
}

/// Show `message` under a field, or hide the line when it is empty.
pub fn set_error(error: &gtk4::Label, message: &str) {
    error.set_text(message);
    error.set_visible(!message.is_empty());
}

/// A revealer that slides `child` down in [`MOTION_MS`] (instantly with animations off) and takes no space while closed.
pub fn reveal(child: &impl IsA<gtk4::Widget>) -> gtk4::Revealer {
    let revealer = gtk4::Revealer::new();
    revealer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
    revealer.set_transition_duration(MOTION_MS);
    revealer.set_child(Some(child));
    revealer.set_visible(false);
    revealer.connect_child_revealed_notify(|revealer| {
        if !revealer.is_child_revealed() && !revealer.reveals_child() {
            revealer.set_visible(false);
        }
    });
    revealer
}

/// Slide a [`reveal`] revealer open or closed.
pub fn show(revealer: &gtk4::Revealer, open: bool) {
    if open {
        revealer.set_visible(true);
    }
    revealer.set_reveal_child(open);
}

/// Remove every child of `container`.
pub fn clear(container: &gtk4::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

/// A themed icon `name` drawn at `size` logical pixels.
pub fn icon(name: &str, size: i32) -> gtk4::Image {
    let image = gtk4::Image::from_icon_name(name);
    image.set_pixel_size(size);
    image
}
