//! The Applications section: one volume per playing app, shown only while something plays.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, Align, Box as GtkBox, IconTheme, Image, Label, Orientation, Revealer};

use super::controls::{Change, VolumeControl};
use crate::backend::{Command, Target};
use crate::hub;
use crate::i18n::Text;
use crate::model::{self, AppGroup, Snapshot};
use crate::{motion, ui};

const FALLBACK_ICON: &str = "application-x-executable-symbolic";

struct AppRow {
    volume: Rc<VolumeControl>,
    streams: Rc<RefCell<Vec<u32>>>,
}

pub struct AppsSection {
    pub root: Revealer,
    card: GtkBox,
    rows: RefCell<Vec<AppRow>>,
    keys: RefCell<Vec<String>>,
    text: &'static Text,
}

impl AppsSection {
    pub fn new(text: &'static Text) -> Rc<Self> {
        let (section, card) = ui::section(text.applications);
        Rc::new(AppsSection {
            root: motion::revealer(&section),
            card,
            rows: RefCell::new(Vec::new()),
            keys: RefCell::new(Vec::new()),
            text,
        })
    }

    pub fn update(&self, snapshot: &Snapshot, cap: u32) {
        let groups = model::group_streams(&snapshot.streams);
        motion::reveal(&self.root, !groups.is_empty());
        let keys: Vec<String> = groups
            .iter()
            .map(|group| format!("{}\u{1}{}", group.key, group.name))
            .collect();
        if *self.keys.borrow() != keys {
            while let Some(child) = self.card.first_child() {
                self.card.remove(&child);
            }
            let rows: Vec<AppRow> = groups.iter().map(|group| self.row(group)).collect();
            self.rows.replace(rows);
            self.keys.replace(keys);
        }
        for (row, group) in self.rows.borrow().iter().zip(&groups) {
            row.streams.replace(group.streams.clone());
            row.volume.set(group.volume, group.muted, cap);
        }
    }

    fn row(&self, group: &AppGroup) -> AppRow {
        let streams = Rc::new(RefCell::new(group.streams.clone()));
        let targets = Rc::clone(&streams);
        let volume = VolumeControl::new(self.text, &group.name, false, move |change| {
            let hub = hub::get();
            for index in targets.borrow().iter() {
                let target = Target::Stream(*index);
                hub.send(match change {
                    Change::Volume(raw) => Command::SetVolume(target, raw, None),
                    Change::Mute(mute) => Command::SetMute(target, mute),
                });
            }
        });
        let line = GtkBox::new(Orientation::Horizontal, 12);
        line.add_css_class("fx-row");
        let icon = Image::from_icon_name(&icon_for(&group.icon));
        icon.set_pixel_size(24);
        icon.set_valign(Align::Center);
        let words = GtkBox::new(Orientation::Vertical, 4);
        words.set_hexpand(true);
        let name = Label::new(Some(&group.name));
        name.add_css_class("fx-row-title");
        name.set_xalign(0.0);
        name.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        words.append(&name);
        words.append(&volume.root);
        line.append(&icon);
        line.append(&words);
        self.card.append(&line);
        AppRow { volume, streams }
    }
}

fn icon_for(name: &str) -> String {
    let theme = gdk::Display::default().map(|display| IconTheme::for_display(&display));
    let known = |candidate: &str| {
        theme
            .as_ref()
            .is_some_and(|theme| theme.has_icon(candidate))
    };
    [
        name.to_string(),
        name.to_ascii_lowercase(),
        format!("{}-symbolic", name.to_ascii_lowercase()),
    ]
    .into_iter()
    .find(|candidate| known(candidate))
    .unwrap_or_else(|| FALLBACK_ICON.to_string())
}
