//! The shared Settings building blocks, plus this crate's page frame.

pub use shell_common::ui::*;

/// The page scaffold with this crate's scoped sheet loaded and its root class set.
pub fn display_page(title: &str, description: &str) -> (gtk4::ScrolledWindow, gtk4::Box) {
    use gtk4::prelude::*;
    page_style(include_str!("../style.css"));
    let (scroll, column) = page(title, description);
    column.add_css_class("fx-display");
    (scroll, column)
}
