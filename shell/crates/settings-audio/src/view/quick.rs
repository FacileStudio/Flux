//! The statusbar popover: output volume and mute, output device, microphone mute, a link to
//! the full page.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gio, Box as GtkBox, Button, Orientation, Popover, Switch, Widget};

use super::controls::{Change, VolumeControl};
use super::lists::DeviceList;
use crate::backend::{Command, Event, Target};
use crate::i18n::Text;
use crate::model::{self, Direction, Snapshot};
use crate::{hub, ui};

struct Quick {
    volume: Rc<VolumeControl>,
    devices: Rc<DeviceList>,
    mic_row: GtkBox,
    mic: Switch,
    output: Rc<Cell<Option<u32>>>,
    input: Rc<Cell<Option<u32>>>,
    guard: Rc<Cell<bool>>,
    cap: Cell<u32>,
    last: RefCell<Option<Snapshot>>,
}

impl Quick {
    fn show(&self, snapshot: &Snapshot) {
        let outputs = snapshot.devices(Direction::Output);
        let sink = snapshot.default_device(Direction::Output);
        self.devices
            .update(&outputs, sink.map(|device| device.name.as_str()));
        self.output.set(sink.map(|device| device.index));
        self.volume.set_sensitive(sink.is_some());
        if let Some(sink) = sink {
            self.volume.set(sink.volume, sink.muted, self.cap.get());
        }
        let source = snapshot.default_device(Direction::Input);
        self.input.set(source.map(|device| device.index));
        self.mic_row.set_visible(source.is_some());
        self.guard.set(true);
        self.mic
            .set_active(source.is_some_and(|device| !device.muted));
        self.guard.set(false);
    }
}

pub fn build(text: &'static Text) -> Widget {
    let root = GtkBox::new(Orientation::Vertical, 8);
    root.add_css_class("fx-quick");
    root.add_css_class("fx-sound");
    let output: Rc<Cell<Option<u32>>> = Rc::new(Cell::new(None));
    let target = Rc::clone(&output);
    let volume = VolumeControl::new(text, text.volume, false, move |change| {
        let Some(index) = target.get() else {
            return;
        };
        let target = Target::Device(Direction::Output, index);
        hub::get().send(match change {
            Change::Volume(raw) => Command::SetVolume(target, raw, None),
            Change::Mute(mute) => Command::SetMute(target, mute),
        });
    });
    let devices = DeviceList::new(text, Direction::Output, |name| {
        hub::get().send(Command::SetDefault(Direction::Output, name));
    });
    let (mic_row, mic) = ui::switch_row(text.microphone, "", false, |_| {});
    let input: Rc<Cell<Option<u32>>> = Rc::new(Cell::new(None));
    let guard = Rc::new(Cell::new(false));
    {
        let input = Rc::clone(&input);
        let guard = Rc::clone(&guard);
        mic.connect_active_notify(move |switch| {
            if guard.get() {
                return;
            }
            if let Some(index) = input.get() {
                hub::get().send(Command::SetMute(
                    Target::Device(Direction::Input, index),
                    !switch.is_active(),
                ));
            }
        });
    }
    let link = Button::with_label(text.settings);
    link.add_css_class("fx-link");
    link.connect_clicked(|button| {
        if let Some(popover) = button
            .ancestor(Popover::static_type())
            .and_downcast::<Popover>()
        {
            popover.popdown();
        }
        let _ = gio::Subprocess::newv(
            &["flux-settings".as_ref(), crate::ID.as_ref()],
            gio::SubprocessFlags::NONE,
        );
    });
    root.append(&volume.root);
    root.append(&devices.root);
    root.append(&mic_row);
    root.append(&link);
    let state = Rc::new(Quick {
        volume,
        devices,
        mic_row,
        mic,
        output,
        input,
        guard,
        cap: Cell::new(100),
        last: RefCell::new(None),
    });
    let weak = Rc::downgrade(&state);
    let surface = root.downgrade();
    let id = hub::get().listen(move |event| {
        let (Some(state), Some(root)) = (weak.upgrade(), surface.upgrade()) else {
            return;
        };
        match event {
            Event::Snapshot(snapshot) => {
                root.set_sensitive(true);
                state.show(snapshot);
                state.last.replace(Some((**snapshot).clone()));
            }
            Event::Unavailable => root.set_sensitive(false),
            Event::Options(options) => {
                state.cap.set(model::max_percent(options.allow_boost));
                let last = state.last.borrow().clone();
                if let Some(snapshot) = last {
                    state.show(&snapshot);
                }
            }
            Event::Level(..) => {}
        }
    });
    root.connect_destroy(move |_| {
        let _ = &state;
        hub::get().forget(id);
    });
    root.upcast()
}
