//! Picking one of several: a device list and a choice row.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, DropDown, Image, Label, ListBox, ListBoxRow, Orientation, Revealer,
    SelectionMode, StringList,
};

use crate::i18n::Text;
use crate::model::{self, Device, Direction};
use crate::{motion, ui};

/// A list of devices with the current one selected; picking one calls back with its name.
pub struct DeviceList {
    pub root: ListBox,
    names: RefCell<Vec<String>>,
    keys: RefCell<Vec<String>>,
    direction: Direction,
    text: &'static Text,
}

impl DeviceList {
    pub fn new(
        text: &'static Text,
        direction: Direction,
        on_pick: impl Fn(String) + 'static,
    ) -> Rc<Self> {
        let root = ListBox::new();
        root.add_css_class("fx-choices");
        root.set_selection_mode(SelectionMode::Single);
        let list = Rc::new(DeviceList {
            root,
            names: RefCell::new(Vec::new()),
            keys: RefCell::new(Vec::new()),
            direction,
            text,
        });
        let weak = Rc::downgrade(&list);
        list.root.connect_row_activated(move |_, row| {
            let Some(list) = weak.upgrade() else {
                return;
            };
            let name = usize::try_from(row.index())
                .ok()
                .and_then(|index| list.names.borrow().get(index).cloned());
            if let Some(name) = name {
                on_pick(name);
            }
        });
        list
    }

    pub fn update(&self, devices: &[&Device], selected: Option<&str>) {
        let labels: Vec<model::Label> = devices
            .iter()
            .map(|device| device.label(self.direction))
            .collect();
        let keys: Vec<String> = devices
            .iter()
            .zip(&labels)
            .map(|(device, label)| {
                format!(
                    "{}\u{1}{}\u{1}{:?}\u{1}{:?}",
                    device.name, label.title, label.detail, label.kind
                )
            })
            .collect();
        if *self.keys.borrow() != keys {
            while let Some(child) = self.root.first_child() {
                self.root.remove(&child);
            }
            for label in &labels {
                self.root.append(&self.row(label));
            }
            self.names
                .replace(devices.iter().map(|device| device.name.clone()).collect());
            self.keys.replace(keys);
        }
        let index =
            selected.and_then(|name| self.names.borrow().iter().position(|item| item == name));
        let row = index
            .and_then(|index| i32::try_from(index).ok())
            .and_then(|index| self.root.row_at_index(index));
        self.root.select_row(row.as_ref());
    }

    fn row(&self, label: &model::Label) -> ListBoxRow {
        let line = GtkBox::new(Orientation::Horizontal, 12);
        let icon = Image::from_icon_name(label.kind.icon());
        icon.set_pixel_size(16);
        let words = GtkBox::new(Orientation::Vertical, 0);
        words.set_hexpand(true);
        let title = Label::new(Some(&label.title));
        title.add_css_class("fx-row-title");
        title.set_xalign(0.0);
        title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        let detail = label
            .detail
            .clone()
            .unwrap_or_else(|| self.text.kind(label.kind).to_string());
        let detail = Label::new(Some(&detail));
        detail.add_css_class("dim");
        detail.set_xalign(0.0);
        detail.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        words.append(&title);
        words.append(&detail);
        let check = Image::from_icon_name("object-select-symbolic");
        check.add_css_class("fx-check");
        line.append(&icon);
        line.append(&words);
        line.append(&check);
        let row = ListBoxRow::new();
        row.set_child(Some(&line));
        row
    }
}

/// A drop-down row that slides in only when there are at least two choices.
pub struct Choice {
    pub root: Revealer,
    dropdown: DropDown,
    keys: RefCell<Vec<(String, String)>>,
    guard: Cell<bool>,
}

impl Choice {
    pub fn new(title: &str, on_pick: impl Fn(String) + 'static) -> Rc<Self> {
        let (row, dropdown) = ui::choice_row(title, None);
        dropdown.set_hexpand(false);
        let choice = Rc::new(Choice {
            root: motion::revealer(&row),
            dropdown,
            keys: RefCell::new(Vec::new()),
            guard: Cell::new(false),
        });
        let weak = Rc::downgrade(&choice);
        choice.dropdown.connect_selected_notify(move |dropdown| {
            let Some(choice) = weak.upgrade() else {
                return;
            };
            if choice.guard.get() {
                return;
            }
            let key = usize::try_from(dropdown.selected())
                .ok()
                .and_then(|index| choice.keys.borrow().get(index).map(|(key, _)| key.clone()));
            if let Some(key) = key {
                on_pick(key);
            }
        });
        choice
    }

    /// Offer `options` (key, label) with `active` selected.
    pub fn update(&self, options: Vec<(String, String)>, active: Option<&str>) {
        motion::reveal(&self.root, options.len() > 1);
        self.guard.set(true);
        if *self.keys.borrow() != options {
            let labels: Vec<&str> = options.iter().map(|(_, label)| label.as_str()).collect();
            self.dropdown.set_model(Some(&StringList::new(&labels)));
            self.keys.replace(options);
        }
        let index = active
            .and_then(|active| self.keys.borrow().iter().position(|(key, _)| key == active))
            .and_then(|index| u32::try_from(index).ok())
            .unwrap_or(gtk4::INVALID_LIST_POSITION);
        self.dropdown.set_selected(index);
        self.guard.set(false);
    }
}
