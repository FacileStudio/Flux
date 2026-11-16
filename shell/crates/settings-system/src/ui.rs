//! The Settings building blocks, plus the few pieces only these pages need.
//! `shared` mirrors `shell_common::ui`; at merge it becomes
//! `pub use shell_common::ui::{...}` and this file keeps only the extras.

mod shared;

pub use shared::{choice_row, danger_button, empty_state, page, primary_button, row, secondary_button, section, spinner_row, switch_row};

use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation, Revealer, RevealerTransitionType, Widget};

/// How long a reveal takes, in milliseconds.
pub const MOTION_MS: u32 = 180;

/// A label with `class`, aligned to the start and wrapping.
pub fn label(text: &str, class: &str) -> Label {
    let label = Label::new(Some(text));
    if !class.is_empty() {
        label.add_css_class(class);
    }
    label.set_xalign(0.0);
    label.set_wrap(true);
    label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
    label.set_max_width_chars(1);
    label
}

/// A quiet action with no fill until hovered.
pub fn ghost_button(text: &str) -> Button {
    let button = Button::with_label(text);
    button.add_css_class("fx-button");
    button.add_css_class("fx-ghost");
    button
}

/// The page column, tagged so these pages' own rules stay scoped to them.
pub fn system_page(title: &str, description: &str) -> (gtk4::ScrolledWindow, Box) {
    let (scroll, column) = page(title, description);
    column.add_css_class("fx-sys");
    (scroll, column)
}

/// An empty suffix for a row that has no control.
pub fn nothing() -> Box {
    Box::new(Orientation::Horizontal, 0)
}

/// A revealer that slides `child` in and out.
pub fn reveal(child: &impl IsA<Widget>) -> Revealer {
    let revealer = Revealer::new();
    revealer.set_transition_type(RevealerTransitionType::SlideDown);
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

/// Slide `revealer` open or closed; a closed one takes no space, not even the gap around it.
pub fn show(revealer: &Revealer, open: bool) {
    if open {
        revealer.set_visible(true);
    }
    revealer.set_reveal_child(open);
}

/// A row of buttons, pushed to the end.
pub fn actions(buttons: &[&Button]) -> Box {
    let line = Box::new(Orientation::Horizontal, 8);
    line.add_css_class("fx-actions");
    line.set_halign(Align::End);
    for button in buttons {
        line.append(*button);
    }
    line
}

/// A labelled input with an error line under it.
pub fn field(title: &str, input: &impl IsA<Widget>) -> (Box, Label) {
    let column = Box::new(Orientation::Vertical, 6);
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
pub fn set_error(error: &Label, message: &str) {
    error.set_text(message);
    error.set_visible(!message.is_empty());
}

/// Remove every child of `container`.
pub fn clear(container: &Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}
