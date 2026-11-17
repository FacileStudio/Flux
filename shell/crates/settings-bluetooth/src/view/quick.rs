//! The compact menu for a statusbar popover: the power switch, paired
//! devices with a connect switch each, and a link to the full page.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{glib, Align, Box, Button, Image, Label, Orientation, Spinner, Switch};

use crate::backend::{Command, Event};
use crate::client::{self, Client};
use crate::i18n::{self, Text};
use crate::model::{self, Availability, Device};
use crate::ui;

struct Item {
    root: Box,
    icon: Image,
    title: Label,
    subtitle: Label,
    spinner: Spinner,
    switch: Switch,
}

struct Menu {
    client: Rc<Client>,
    text: &'static Text,
    power: Switch,
    status: Label,
    list: Box,
    items: RefCell<HashMap<String, Item>>,
    busy: RefCell<HashSet<String>>,
    syncing: Rc<Cell<bool>>,
}

fn item(menu: &Rc<Menu>, device: &Device) -> Item {
    let root = Box::new(Orientation::Horizontal, 12);
    root.add_css_class("fx-row");
    root.add_css_class("bt-quick-item");
    let icon = Image::from_icon_name(device.kind().icon());
    icon.set_pixel_size(16);
    icon.add_css_class("bt-icon");
    let (texts, title, subtitle) = ui::texts(&device.display_name(), "");
    let spinner = Spinner::new();
    spinner.set_valign(Align::Center);
    let switch = Switch::new();
    switch.set_valign(Align::Center);
    root.append(&icon);
    root.append(&texts);
    root.append(&spinner);
    root.append(&switch);
    let weak = Rc::downgrade(menu);
    let path = device.path.clone();
    switch.connect_state_set(move |_, on| {
        let Some(menu) = weak.upgrade() else {
            return glib::Propagation::Proceed;
        };
        if menu.syncing.get() {
            return glib::Propagation::Proceed;
        }
        menu.busy.borrow_mut().insert(path.clone());
        let command = if on { Command::Connect(path.clone()) } else { Command::Disconnect(path.clone()) };
        menu.client.send(command);
        menu.render();
        glib::Propagation::Stop
    });
    Item {
        root,
        icon,
        title,
        subtitle,
        spinner,
        switch,
    }
}

fn open_settings(widget: &impl IsA<gtk4::Widget>) {
    let launched = gtk4::gio::Subprocess::newv(
        &[std::ffi::OsStr::new("flux-settings"), std::ffi::OsStr::new(crate::ID)],
        gtk4::gio::SubprocessFlags::NONE,
    );
    if let Err(error) = launched {
        eprintln!("settings-bluetooth: cannot open flux-settings: {error}");
    }
    if let Some(popover) = widget.ancestor(gtk4::Popover::static_type()).and_downcast::<gtk4::Popover>() {
        popover.popdown();
    }
}

impl Menu {
    fn render(self: &Rc<Self>) {
        let snapshot = self.client.snapshot();
        let availability = snapshot.availability();
        let on = availability == Availability::On;
        self.syncing.set(true);
        self.power.set_active(on);
        self.power.set_state(on);
        self.syncing.set(false);
        self.power
            .set_sensitive(matches!(availability, Availability::On | Availability::Off | Availability::SoftBlocked));
        let status = match availability {
            Availability::On => "",
            Availability::Off => self.text.off_title,
            Availability::SoftBlocked => self.text.soft_title,
            Availability::HardBlocked => self.text.hard_title,
            Availability::NoAdapter => self.text.no_adapter_title,
            Availability::NoService => self.text.no_service_title,
        };
        self.status.set_text(status);
        self.status.set_visible(!status.is_empty());
        let devices = if on { snapshot.paired() } else { Vec::new() };
        let mut seen = HashSet::new();
        let mut previous: Option<gtk4::Widget> = None;
        for device in devices {
            seen.insert(device.path.clone());
            let mut items = self.items.borrow_mut();
            let entry = items.entry(device.path.clone()).or_insert_with(|| item(self, device));
            if entry.root.parent().is_none() {
                self.list.append(&entry.root);
            }
            self.list.reorder_child_after(&entry.root, previous.as_ref());
            previous = Some(entry.root.clone().upcast());
            let busy = self.busy.borrow().contains(&device.path);
            entry.title.set_text(&device.display_name());
            entry.icon.set_icon_name(Some(device.kind().icon()));
            let line = match (busy, device.battery.filter(|_| device.connected)) {
                (true, _) => self.text.connecting.to_string(),
                (false, Some(percent)) => model::battery_label(percent, self.text.battery),
                (false, None) if device.connected => self.text.connected.to_string(),
                (false, None) => String::new(),
            };
            entry.subtitle.set_text(&line);
            entry.subtitle.set_visible(!line.is_empty());
            entry.spinner.set_visible(busy);
            entry.spinner.set_spinning(busy);
            self.syncing.set(true);
            if !busy {
                entry.switch.set_active(device.connected);
            }
            entry.switch.set_state(device.connected);
            self.syncing.set(false);
        }
        self.items.borrow_mut().retain(|path, item| {
            let keep = seen.contains(path);
            if !keep {
                self.list.remove(&item.root);
            }
            keep
        });
        self.list.set_visible(!seen.is_empty());
    }
}

/// The quick menu, connected to the process's client.
pub fn quick_menu() -> gtk4::Widget {
    super::install_style();
    let text = i18n::text();
    let root = Box::new(Orientation::Vertical, 8);
    root.add_css_class("bt-quick");
    let header = Box::new(Orientation::Horizontal, 12);
    header.add_css_class("bt-quick-header");
    let icon = Image::from_icon_name("bluetooth-active-symbolic");
    icon.set_pixel_size(16);
    let title = ui::label(text.title, "fx-title");
    title.set_hexpand(true);
    let power = Switch::new();
    power.set_valign(Align::Center);
    header.append(&icon);
    header.append(&title);
    header.append(&power);
    let status = ui::label("", "fx-subtitle");
    status.add_css_class("bt-quick-status");
    let list = Box::new(Orientation::Vertical, 0);
    list.add_css_class("fx-card");
    let link = Button::with_label(text.settings_link);
    link.add_css_class("fx-button");
    link.add_css_class("fx-ghost");
    link.set_halign(Align::Start);
    root.append(&header);
    root.append(&status);
    root.append(&list);
    root.append(&link);
    let menu = Rc::new(Menu {
        client: client::client(),
        text,
        power,
        status,
        list,
        items: RefCell::new(HashMap::new()),
        busy: RefCell::new(HashSet::new()),
        syncing: Rc::new(Cell::new(false)),
    });
    let weak = Rc::downgrade(&menu);
    menu.power.connect_state_set(move |_, on| {
        let Some(menu) = weak.upgrade() else {
            return glib::Propagation::Proceed;
        };
        if menu.syncing.get() {
            return glib::Propagation::Proceed;
        }
        menu.client.send(Command::SetPowered(on));
        glib::Propagation::Stop
    });
    link.connect_clicked(open_settings);
    let weak = Rc::downgrade(&menu);
    menu.client.subscribe(&root, false, move |event| {
        let Some(menu) = weak.upgrade() else {
            return;
        };
        if let Event::Done { device, .. } = event {
            menu.busy.borrow_mut().remove(device);
        }
        if matches!(event, Event::Snapshot(_) | Event::Done { .. }) {
            menu.render();
        }
    });
    menu.render();
    let keep = RefCell::new(Some(menu));
    root.connect_destroy(move |_| {
        keep.borrow_mut().take();
    });
    root.upcast()
}
