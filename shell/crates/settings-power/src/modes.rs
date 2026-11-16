//! The power-mode segmented control, shared by the page and the quick menu.

use std::cell::Cell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n::Text;
use crate::source::{Snapshot, Source};
use crate::upower::PROFILES;

/// One toggle per known profile, hidden when the hardware lacks it.
#[derive(Clone)]
pub struct Modes {
    pub root: gtk4::Box,
    buttons: Vec<(&'static str, gtk4::ToggleButton)>,
    syncing: Rc<Cell<bool>>,
}

fn name(text: &Text, profile: &str) -> &'static str {
    match profile {
        "power-saver" => text.power_saver,
        "performance" => text.performance,
        _ => text.balanced,
    }
}

impl Modes {
    pub fn new(text: &Text, source: &Source) -> Modes {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        root.add_css_class("fx-segmented");
        root.set_homogeneous(true);
        let syncing = Rc::new(Cell::new(false));
        let mut buttons: Vec<(&'static str, gtk4::ToggleButton)> = Vec::new();
        for profile in PROFILES {
            let button = gtk4::ToggleButton::with_label(name(text, profile));
            if let Some((_, first)) = buttons.first() {
                button.set_group(Some(first));
            }
            let source = source.clone();
            let guard = syncing.clone();
            button.connect_toggled(move |button| {
                if button.is_active() && !guard.get() {
                    source.set_profile(profile);
                }
            });
            root.append(&button);
            buttons.push((profile, button));
        }
        Modes { root, buttons, syncing }
    }

    /// Show the profiles `snapshot` offers and press the active one.
    pub fn sync(&self, snapshot: &Snapshot) {
        self.syncing.set(true);
        for (profile, button) in &self.buttons {
            button.set_visible(snapshot.profiles.contains(profile));
            button.set_active(snapshot.active.as_deref() == Some(*profile));
        }
        self.root.set_visible(!snapshot.profiles.is_empty());
        self.syncing.set(false);
    }
}
