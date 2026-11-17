//! The shared Settings building blocks, plus the two pieces only these pages need.

pub use shell_common::ui::*;

use gtk4::prelude::*;
use gtk4::{Box, Orientation};

/// The page column, tagged so these pages' own rules stay scoped to them.
pub fn system_page(title: &str, description: &str) -> (gtk4::ScrolledWindow, Box) {
    page_style(crate::CSS);
    let (scroll, column) = page(title, description);
    column.add_css_class("fx-sys");
    (scroll, column)
}

/// An empty suffix for a row that has no control.
pub fn nothing() -> Box {
    Box::new(Orientation::Horizontal, 0)
}
