//! The PulseAudio protocol backend, spoken to `pipewire-pulse` (or a real PulseAudio).
//!
//! One thread owns the connection. It runs libpulse's threaded main loop, which calls back on
//! its own thread, and takes the lock to issue each command. Every libpulse object is touched
//! only with that lock held, which is the contract the threaded main loop documents.

mod load;
mod streams;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use libpulse_binding as pa;
use pa::callbacks::ListResult;
use pa::context::subscribe::{Facility, InterestMaskSet};
use pa::context::{Context, FlagSet as ContextFlags, State as ContextState};
use pa::mainloop::threaded::Mainloop;
use pa::proplist::{properties, Proplist};
use pa::stream::Stream as PaStream;
use pa::volume::{ChannelVolumes, Volume};

use super::{Backend, Command, Event, Target};
use crate::model::{self, Direction, Snapshot};

const RETRY: Duration = Duration::from_secs(3);
const SERVER: u8 = 1;
const SINKS: u8 = 2;
const SOURCES: u8 = 4;
const STREAMS: u8 = 8;
const CARDS: u8 = 16;
const ALL: u8 = SERVER | SINKS | SOURCES | STREAMS | CARDS;

enum Message {
    Command(Command),
    Lost,
}

/// The handle the page holds: sending never blocks.
pub struct Pulse {
    messages: mpsc::Sender<Message>,
}

impl Backend for Pulse {
    fn send(&self, command: Command) {
        let _ = self.messages.send(Message::Command(command));
    }
}

/// Start the connection thread. Events arrive on `events` until the handle is dropped.
pub fn spawn(events: async_channel::Sender<Event>) -> Pulse {
    let (messages, receiver) = mpsc::channel();
    let wake = messages.clone();
    let _ = std::thread::Builder::new()
        .name("flux-audio".to_string())
        .spawn(move || run(&receiver, &wake, &events));
    Pulse { messages }
}

fn run(
    messages: &mpsc::Receiver<Message>,
    wake: &mpsc::Sender<Message>,
    events: &async_channel::Sender<Event>,
) {
    let meters = Rc::new(Cell::new(false));
    loop {
        if let Some(session) = Session::open(events, wake, &meters) {
            let quit = session.serve(messages);
            session.close();
            if quit {
                return;
            }
        }
        if events.try_send(Event::Unavailable).is_err() || wait(messages, RETRY, &meters) {
            return;
        }
    }
}

fn wait(messages: &mpsc::Receiver<Message>, pause: Duration, meters: &Cell<bool>) -> bool {
    let deadline = Instant::now() + pause;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        match messages.recv_timeout(left) {
            Ok(Message::Command(command)) => refuse(command, meters),
            Ok(Message::Lost) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => return false,
            Err(mpsc::RecvTimeoutError::Disconnected) => return true,
        }
    }
}

fn refuse(command: Command, meters: &Cell<bool>) {
    match command {
        Command::SetVolume(_, _, Some(done)) => {
            let _ = done.send(false);
        }
        Command::Meters(on) => meters.set(on),
        _ => {}
    }
}

#[derive(Default)]
struct Meters {
    on: bool,
    sources: (Option<String>, Option<String>),
    streams: Vec<Rc<RefCell<PaStream>>>,
}

struct Shared {
    snapshot: Snapshot,
    volumes: BTreeMap<(u8, u32), ChannelVolumes>,
    loaded: u8,
    inflight: u8,
    dirty: u8,
    meters: Meters,
    test: Option<Rc<RefCell<PaStream>>>,
    events: async_channel::Sender<Event>,
}

impl Shared {
    fn emit(&self) {
        if self.loaded == ALL {
            let _ = self
                .events
                .try_send(Event::Snapshot(Box::new(self.snapshot.clone())));
        }
    }
}

struct Session {
    mainloop: Rc<RefCell<Mainloop>>,
    context: Rc<RefCell<Context>>,
    shared: Rc<RefCell<Shared>>,
    meters: Rc<Cell<bool>>,
}

type Handles = (Rc<RefCell<Mainloop>>, Rc<RefCell<Context>>);

fn connect() -> Option<Handles> {
    let mut props = Proplist::new()?;
    props
        .set_str(properties::APPLICATION_NAME, model::OWN_APP)
        .ok()?;
    props
        .set_str(properties::APPLICATION_ID, "studio.facile.FluxSettings")
        .ok()?;
    props
        .set_str(
            properties::APPLICATION_ICON_NAME,
            "audio-volume-high-symbolic",
        )
        .ok()?;
    let mainloop = Rc::new(RefCell::new(Mainloop::new()?));
    let context = Context::new_with_proplist(&*mainloop.borrow(), model::OWN_APP, &props)?;
    let context = Rc::new(RefCell::new(context));
    let signal = Rc::clone(&mainloop);
    context
        .borrow_mut()
        .set_state_callback(Some(Box::new(move || unsafe {
            (*signal.as_ptr()).signal(false)
        })));
    mainloop.borrow_mut().lock();
    let ready = mainloop.borrow_mut().start().is_ok()
        && context
            .borrow_mut()
            .connect(None, ContextFlags::NOAUTOSPAWN, None)
            .is_ok()
        && wait_ready(&mainloop, &context);
    if !ready {
        context.borrow_mut().set_state_callback(None);
        context.borrow_mut().disconnect();
        mainloop.borrow_mut().unlock();
        mainloop.borrow_mut().stop();
        return None;
    }
    Some((mainloop, context))
}

fn wait_ready(mainloop: &Rc<RefCell<Mainloop>>, context: &Rc<RefCell<Context>>) -> bool {
    loop {
        match context.borrow().get_state() {
            ContextState::Ready => return true,
            ContextState::Failed | ContextState::Terminated => return false,
            _ => {}
        }
        mainloop.borrow_mut().wait();
    }
}

impl Session {
    fn open(
        events: &async_channel::Sender<Event>,
        wake: &mpsc::Sender<Message>,
        meters: &Rc<Cell<bool>>,
    ) -> Option<Session> {
        let (mainloop, context) = connect()?;
        let shared = Rc::new(RefCell::new(Shared {
            snapshot: Snapshot::default(),
            volumes: BTreeMap::new(),
            loaded: 0,
            inflight: 0,
            dirty: 0,
            meters: Meters {
                on: meters.get(),
                ..Meters::default()
            },
            test: None,
            events: events.clone(),
        }));
        let session = Session {
            mainloop,
            context,
            shared,
            meters: Rc::clone(meters),
        };
        session.watch(wake.clone());
        session.mainloop.borrow_mut().unlock();
        Some(session)
    }

    fn watch(&self, wake: mpsc::Sender<Message>) {
        let weak = Rc::downgrade(&self.context);
        let mut context = self.context.borrow_mut();
        context.set_state_callback(Some(Box::new(move || {
            let good = weak
                .upgrade()
                .and_then(|context| context.try_borrow().ok().map(|c| c.get_state().is_good()));
            if good != Some(true) {
                let _ = wake.send(Message::Lost);
            }
        })));
        let weak = Rc::downgrade(&self.context);
        let shared = Rc::clone(&self.shared);
        context.set_subscribe_callback(Some(Box::new(move |facility, _, _| {
            if let Some(bit) = facility.and_then(bit_of) {
                load::refresh(bit, &weak, &shared);
            }
        })));
        let mask = InterestMaskSet::SERVER
            | InterestMaskSet::SINK
            | InterestMaskSet::SOURCE
            | InterestMaskSet::SINK_INPUT
            | InterestMaskSet::CARD;
        context.subscribe(mask, |_| {});
        drop(context);
        let weak = Rc::downgrade(&self.context);
        for bit in [SERVER, SINKS, SOURCES, STREAMS, CARDS] {
            load::refresh(bit, &weak, &self.shared);
        }
    }

    fn serve(&self, messages: &mpsc::Receiver<Message>) -> bool {
        loop {
            match messages.recv() {
                Ok(Message::Command(command)) => {
                    self.mainloop.borrow_mut().lock();
                    self.apply(command);
                    self.mainloop.borrow_mut().unlock();
                }
                Ok(Message::Lost) => return false,
                Err(_) => return true,
            }
        }
    }

    fn close(self) {
        self.mainloop.borrow_mut().lock();
        {
            let mut shared = self.shared.borrow_mut();
            shared.meters.streams.clear();
            shared.test = None;
        }
        {
            let mut context = self.context.borrow_mut();
            context.set_state_callback(None);
            context.set_subscribe_callback(None);
            context.disconnect();
        }
        self.mainloop.borrow_mut().unlock();
        self.mainloop.borrow_mut().stop();
    }

    fn apply(&self, command: Command) {
        if !self.context.borrow().get_state().is_good() {
            refuse(command, &self.meters);
            return;
        }
        match command {
            Command::SetDefault(direction, name) => self.set_default(direction, &name),
            Command::SetVolume(target, raw, done) => self.set_volume(target, raw, done),
            Command::SetMute(target, mute) => self.set_mute(target, mute),
            Command::SetPort(direction, index, port) => {
                let mut introspect = self.context.borrow().introspect();
                match direction {
                    Direction::Output => introspect.set_sink_port_by_index(index, &port, None),
                    Direction::Input => introspect.set_source_port_by_index(index, &port, None),
                };
            }
            Command::SetProfile(card, profile) => {
                self.context
                    .borrow()
                    .introspect()
                    .set_card_profile_by_index(card, &profile, None);
            }
            Command::Test(sink) => {
                let stream = streams::play_test(&mut self.context.borrow_mut(), &sink);
                self.shared.borrow_mut().test = stream;
            }
            Command::Meters(on) => {
                self.meters.set(on);
                self.shared.borrow_mut().meters.on = on;
                streams::update_meters(&self.context, &self.shared);
            }
        }
    }

    fn set_mute(&self, target: Target, mute: bool) {
        let mut introspect = self.context.borrow().introspect();
        match target {
            Target::Device(Direction::Output, index) => {
                introspect.set_sink_mute_by_index(index, mute, None)
            }
            Target::Device(Direction::Input, index) => {
                introspect.set_source_mute_by_index(index, mute, None)
            }
            Target::Stream(index) => introspect.set_sink_input_mute(index, mute, None),
        };
    }

    fn set_default(&self, direction: Direction, name: &str) {
        let mut context = self.context.borrow_mut();
        if direction == Direction::Output {
            context.set_default_sink(name, |_| {});
            let streams: Vec<u32> = self
                .shared
                .borrow()
                .snapshot
                .streams
                .iter()
                .map(|s| s.index)
                .collect();
            let mut introspect = context.introspect();
            for index in streams {
                introspect.move_sink_input_by_name(index, name, None);
            }
            return;
        }
        context.set_default_source(name, |_| {});
        let monitors: Vec<u32> = self
            .shared
            .borrow()
            .snapshot
            .sources
            .iter()
            .filter(|s| s.is_monitor)
            .map(|s| s.index)
            .collect();
        let weak = Rc::downgrade(&self.context);
        let target = name.to_string();
        context
            .introspect()
            .get_source_output_info_list(move |result| {
                let ListResult::Item(info) = result else {
                    return;
                };
                let own = info
                    .proplist
                    .get_str(properties::APPLICATION_NAME)
                    .as_deref()
                    == Some(model::OWN_APP);
                if own || monitors.contains(&info.source) {
                    return;
                }
                if let Some(context) = weak.upgrade() {
                    if let Ok(context) = context.try_borrow() {
                        context
                            .introspect()
                            .move_source_output_by_name(info.index, &target, None);
                    }
                }
            });
    }

    fn set_volume(&self, target: Target, raw: u32, done: Option<mpsc::Sender<bool>>) {
        let key = match target {
            Target::Device(Direction::Output, index) => (SINKS, index),
            Target::Device(Direction::Input, index) => (SOURCES, index),
            Target::Stream(index) => (STREAMS, index),
        };
        let volumes = self
            .shared
            .borrow_mut()
            .volumes
            .get_mut(&key)
            .map(|volumes| {
                if volumes.scale(Volume(raw.min(Volume::MAX.0))).is_none() {
                    volumes.set(volumes.len().max(1), Volume(raw));
                }
                *volumes
            });
        let Some(volumes) = volumes else {
            if let Some(done) = done {
                let _ = done.send(false);
            }
            return;
        };
        let callback = done.map(|done| {
            Box::new(move |ok| {
                let _ = done.send(ok);
            }) as Box<dyn FnMut(bool)>
        });
        let mut introspect = self.context.borrow().introspect();
        match target {
            Target::Device(Direction::Output, index) => {
                introspect.set_sink_volume_by_index(index, &volumes, callback)
            }
            Target::Device(Direction::Input, index) => {
                introspect.set_source_volume_by_index(index, &volumes, callback)
            }
            Target::Stream(index) => introspect.set_sink_input_volume(index, &volumes, callback),
        };
    }
}

fn bit_of(facility: Facility) -> Option<u8> {
    match facility {
        Facility::Server => Some(SERVER),
        Facility::Sink => Some(SINKS),
        Facility::Source => Some(SOURCES),
        Facility::SinkInput => Some(STREAMS),
        Facility::Card => Some(CARDS),
        _ => None,
    }
}
