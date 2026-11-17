//! A backend that plays the laptop fixture and applies commands to it in memory, so the page
//! can be looked at and clicked without a sound server.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk4::glib;

use super::{Backend, Command, Event, Target};
use crate::model::{Direction, Snapshot};

const SAMPLE: &str = include_str!("../../fixtures/laptop.json");

pub struct Preview {
    snapshot: Rc<RefCell<Snapshot>>,
    events: async_channel::Sender<Event>,
    meters: RefCell<Option<glib::SourceId>>,
}

impl Preview {
    /// `mode` is the preview variable: `unavailable` plays a missing server, `bluetooth` starts
    /// on the headset, anything else on the built-in speakers.
    pub fn new(events: async_channel::Sender<Event>, mode: &str) -> Preview {
        let mut snapshot: Snapshot = serde_json::from_str(SAMPLE).unwrap_or_default();
        if mode == "bluetooth" {
            snapshot.default_sink = Some("bluez_output.AC_80_0A_12_34_56.1".to_string());
        }
        let first = if mode == "unavailable" {
            Event::Unavailable
        } else {
            Event::Snapshot(Box::new(snapshot.clone()))
        };
        let _ = events.try_send(first);
        Preview {
            snapshot: Rc::new(RefCell::new(snapshot)),
            events,
            meters: RefCell::new(None),
        }
    }

    fn emit(&self) {
        let _ = self
            .events
            .try_send(Event::Snapshot(Box::new(self.snapshot.borrow().clone())));
    }
}

impl Backend for Preview {
    fn send(&self, command: Command) {
        match command {
            Command::Meters(on) => self.meters(on),
            Command::Test(_) => {}
            command => {
                apply(&mut self.snapshot.borrow_mut(), command);
                self.emit();
            }
        }
    }
}

fn apply(snapshot: &mut Snapshot, command: Command) {
    match command {
        Command::SetDefault(Direction::Output, name) => snapshot.default_sink = Some(name),
        Command::SetDefault(Direction::Input, name) => snapshot.default_source = Some(name),
        Command::SetVolume(target, raw, done) => {
            edit(snapshot, target, |volume, _| *volume = raw);
            if let Some(done) = done {
                let _ = done.send(true);
            }
        }
        Command::SetMute(target, mute) => edit(snapshot, target, |_, muted| *muted = mute),
        Command::SetPort(direction, index, port) => {
            let list = match direction {
                Direction::Output => &mut snapshot.sinks,
                Direction::Input => &mut snapshot.sources,
            };
            if let Some(device) = list.iter_mut().find(|device| device.index == index) {
                device.active_port = Some(port);
            }
        }
        Command::SetProfile(index, profile) => {
            if let Some(card) = snapshot.cards.iter_mut().find(|card| card.index == index) {
                card.active_profile = Some(profile);
            }
        }
        Command::Test(_) | Command::Meters(_) => {}
    }
}

impl Preview {
    fn meters(&self, on: bool) {
        if let Some(source) = self.meters.borrow_mut().take() {
            source.remove();
        }
        if !on {
            return;
        }
        let events = self.events.clone();
        let tick = Cell::new(0u32);
        let source = glib::timeout_add_local(Duration::from_millis(40), move || {
            let step = tick.get().wrapping_add(1);
            tick.set(step);
            let phase = step as f32 / 6.0;
            let output = 0.12 + 0.1 * phase.sin().abs() + if step % 17 < 3 { 0.15 } else { 0.0 };
            let input = 0.02 + 0.05 * (phase * 0.7).cos().abs();
            let _ = events.try_send(Event::Level(Direction::Output, output));
            let _ = events.try_send(Event::Level(Direction::Input, input));
            glib::ControlFlow::Continue
        });
        self.meters.replace(Some(source));
    }
}

fn edit(snapshot: &mut Snapshot, target: Target, change: impl FnOnce(&mut u32, &mut bool)) {
    match target {
        Target::Device(direction, index) => {
            let list = match direction {
                Direction::Output => &mut snapshot.sinks,
                Direction::Input => &mut snapshot.sources,
            };
            if let Some(device) = list.iter_mut().find(|device| device.index == index) {
                change(&mut device.volume, &mut device.muted);
            }
        }
        Target::Stream(index) => {
            if let Some(stream) = snapshot
                .streams
                .iter_mut()
                .find(|stream| stream.index == index)
            {
                change(&mut stream.volume, &mut stream.muted);
            }
        }
    }
}
