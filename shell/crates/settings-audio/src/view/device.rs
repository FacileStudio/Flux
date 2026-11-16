//! The Output and Input sections: which device, how loud, which connector and mode.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};

use super::controls::{Change, VolumeControl};
use super::lists::{Choice, DeviceList};
use crate::backend::{Command, Target};
use crate::hub;
use crate::i18n::Text;
use crate::model::{Direction, Snapshot};
use crate::ui;

pub struct DeviceSection {
    pub root: GtkBox,
    direction: Direction,
    list: Rc<DeviceList>,
    empty: Label,
    controls: GtkBox,
    volume: Rc<VolumeControl>,
    port: Rc<Choice>,
    profile: Rc<Choice>,
    current: Rc<Cell<Option<u32>>>,
    card: Rc<Cell<Option<u32>>>,
    sink: RefCell<Option<String>>,
}

impl DeviceSection {
    pub fn new(text: &'static Text, direction: Direction) -> Rc<Self> {
        let (title, empty_text) = match direction {
            Direction::Output => (text.output, text.no_output),
            Direction::Input => (text.input, text.no_input),
        };
        let (root, card) = ui::section(title);
        let list = DeviceList::new(text, direction, move |name| {
            hub::get().send(Command::SetDefault(direction, name))
        });
        let empty = Label::new(Some(empty_text));
        empty.add_css_class("fx-subtitle");
        empty.add_css_class("fx-row");
        empty.set_xalign(0.0);
        let current: Rc<Cell<Option<u32>>> = Rc::new(Cell::new(None));
        let target = Rc::clone(&current);
        let volume = VolumeControl::new(text, text.volume, true, move |change| {
            let Some(index) = target.get() else {
                return;
            };
            let target = Target::Device(direction, index);
            hub::get().send(match change {
                Change::Volume(raw) => Command::SetVolume(target, raw, None),
                Change::Mute(mute) => Command::SetMute(target, mute),
            });
        });
        let port_target = Rc::clone(&current);
        let port = Choice::new(text.port, move |port| {
            if let Some(index) = port_target.get() {
                hub::get().send(Command::SetPort(direction, index, port));
            }
        });
        let card_index: Rc<Cell<Option<u32>>> = Rc::new(Cell::new(None));
        let profile_card = Rc::clone(&card_index);
        let profile = Choice::new(text.profile, move |profile| {
            if let Some(card) = profile_card.get() {
                hub::get().send(Command::SetProfile(card, profile));
            }
        });
        let controls = GtkBox::new(Orientation::Vertical, 0);
        let volume_row = GtkBox::new(Orientation::Vertical, 0);
        volume_row.add_css_class("fx-row");
        volume_row.append(&volume.root);
        controls.append(&volume_row);
        controls.append(&port.root);
        controls.append(&profile.root);
        card.append(&list.root);
        card.append(&empty);
        card.append(&controls);
        let section = Rc::new(DeviceSection {
            root,
            direction,
            list,
            empty,
            controls,
            volume,
            port,
            profile,
            current,
            card: card_index,
            sink: RefCell::new(None),
        });
        if direction == Direction::Output {
            let button = ui::secondary_button(text.test);
            let weak = Rc::downgrade(&section);
            button.connect_clicked(move |_| {
                let sink = weak
                    .upgrade()
                    .and_then(|section| section.sink.borrow().clone());
                if let Some(sink) = sink {
                    hub::get().send(Command::Test(sink));
                }
            });
            let row = ui::row(text.test_title, text.test_body, &button);
            section.controls.append(&row);
        }
        section
    }

    pub fn update(&self, snapshot: &Snapshot, cap: u32) {
        let devices = snapshot.devices(self.direction);
        let current = snapshot.default_device(self.direction);
        self.list
            .update(&devices, current.map(|device| device.name.as_str()));
        self.list.root.set_visible(!devices.is_empty());
        self.empty.set_visible(devices.is_empty());
        self.controls.set_visible(current.is_some());
        self.current.set(current.map(|device| device.index));
        self.sink.replace(current.map(|device| device.name.clone()));
        let Some(device) = current else {
            self.volume.level(0.0);
            return;
        };
        self.volume.set(device.volume, device.muted, cap);
        let ports = device
            .port_choices()
            .iter()
            .map(|port| (port.name.clone(), port.description.clone()))
            .collect();
        self.port.update(ports, device.active_port.as_deref());
        let shared = self.direction == Direction::Input
            && snapshot
                .default_device(Direction::Output)
                .and_then(|output| output.card)
                == device.card;
        let card = snapshot.card_of(device).filter(|_| !shared);
        self.card.set(card.map(|card| card.index));
        let profiles = card
            .map(|card| {
                card.profile_choices()
                    .iter()
                    .map(|profile| (profile.name.clone(), profile.description.clone()))
                    .collect()
            })
            .unwrap_or_default();
        self.profile.update(
            profiles,
            card.and_then(|card| card.active_profile.as_deref()),
        );
    }

    pub fn level(&self, peak: f32) {
        self.volume.level(peak);
    }
}
