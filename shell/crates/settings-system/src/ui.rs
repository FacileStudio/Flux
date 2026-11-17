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

/// The one banner a page shows when `/etc/flux/settings.json` cannot be read: what is wrong, what it means, and the parser's own words.
pub fn problem_banner(text: &crate::i18n::Text, problem: &str) -> Box {
    let banner = Box::new(Orientation::Vertical, 4);
    banner.add_css_class("fx-banner");
    banner.add_css_class("fx-danger");
    banner.append(&label(text.problem_title, "fx-title"));
    for line in [text.problem_body, problem] {
        let line = label(line, "fx-subtitle");
        line.set_selectable(true);
        banner.append(&line);
    }
    banner
}
