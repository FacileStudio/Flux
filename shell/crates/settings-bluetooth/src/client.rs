//! The GTK side of the worker, one per process: the page and the quick menu
//! share it, read the last snapshot from it and get every event it receives.
//!
//! With `FLUX_BLUETOOTH_PREVIEW=<scene>` no worker starts. The client serves a
//! scene from [`crate::preview`] and plays commands out locally, so the page
//! can be shown and screenshot without BlueZ.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use async_channel::Sender;
use gtk4::glib;
use gtk4::prelude::*;

use crate::backend::{self, Answer, Command, Event, Op, Request};
use crate::model::{Failure, Prompt, Snapshot, Step};
use crate::preview;

type Callback = Rc<dyn Fn(&Event)>;
type Listener = (glib::WeakRef<gtk4::Widget>, bool, Callback);

pub struct Client {
    commands: Option<Sender<Command>>,
    snapshot: RefCell<Snapshot>,
    listeners: RefCell<Vec<Listener>>,
    expanded: Option<String>,
}

thread_local! {
    static CLIENT: RefCell<Option<Rc<Client>>> = const { RefCell::new(None) };
}

/// The process's client, started on first use.
pub fn client() -> Rc<Client> {
    CLIENT.with(|slot| {
        if let Some(client) = slot.borrow().as_ref() {
            return Rc::clone(client);
        }
        let client = match std::env::var("FLUX_BLUETOOTH_PREVIEW") {
            Ok(scene) => Client::preview(&scene),
            Err(_) => Client::live(),
        };
        *slot.borrow_mut() = Some(Rc::clone(&client));
        client
    })
}

impl Client {
    fn live() -> Rc<Client> {
        let backend = backend::start();
        let client = Rc::new(Client {
            commands: Some(backend.commands),
            snapshot: RefCell::new(Snapshot::default()),
            listeners: RefCell::new(Vec::new()),
            expanded: None,
        });
        let pump = Rc::clone(&client);
        glib::spawn_future_local(async move {
            while let Ok(event) = backend.events.recv().await {
                pump.dispatch(event);
            }
        });
        client
    }

    fn preview(name: &str) -> Rc<Client> {
        let scene = preview::scene(name);
        let client = Rc::new(Client {
            commands: None,
            snapshot: RefCell::new(scene.snapshot),
            listeners: RefCell::new(Vec::new()),
            expanded: scene.expanded,
        });
        let target = client.snapshot().nearby(false).first().map(|device| device.path.clone());
        let opening = Rc::clone(&client);
        glib::idle_add_local_once(move || {
            if let (true, Some(device)) = (scene.pairing, target) {
                opening.dispatch(Event::Pairing(Step::Start(device)));
            }
            if let Some((device, prompt)) = scene.prompt {
                opening.dispatch(Event::Pairing(Step::Start(device.clone())));
                opening.ask(device, prompt);
            }
        });
        client
    }

    /// The last state the worker reported.
    pub fn snapshot(&self) -> Snapshot {
        self.snapshot.borrow().clone()
    }

    /// The device the preview opens expanded.
    pub fn expanded(&self) -> Option<&str> {
        self.expanded.as_deref()
    }

    /// Call `callback` with every event for as long as `owner` lives. A
    /// listener with `prompts` set answers agent requests; with none alive,
    /// requests are rejected at once.
    pub fn subscribe(&self, owner: &impl IsA<gtk4::Widget>, prompts: bool, callback: impl Fn(&Event) + 'static) {
        self.listeners
            .borrow_mut()
            .push((owner.upcast_ref::<gtk4::Widget>().downgrade(), prompts, Rc::new(callback)));
    }

    fn dispatch(&self, event: Event) {
        if let Event::Snapshot(snapshot) = &event {
            *self.snapshot.borrow_mut() = snapshot.clone();
        }
        let callbacks: Vec<(bool, Callback)> = {
            let mut listeners = self.listeners.borrow_mut();
            listeners.retain(|(owner, _, _)| owner.upgrade().is_some());
            listeners.iter().map(|(_, prompts, callback)| (*prompts, Rc::clone(callback))).collect()
        };
        if let Event::Request(Request { reply: Some(reply), .. }) = &event {
            if !callbacks.iter().any(|(prompts, _)| *prompts) {
                let _ = reply.try_send(Answer::Reject);
                return;
            }
        }
        for (_, callback) in callbacks {
            callback(&event);
        }
    }

    /// Hand `command` to the worker.
    pub fn send(self: &Rc<Self>, command: Command) {
        match &self.commands {
            Some(commands) => {
                let _ = commands.try_send(command);
            }
            None => self.play(command),
        }
    }

    fn ask(self: &Rc<Self>, device: String, prompt: Prompt) {
        let (reply, answer) = async_channel::bounded(1);
        let needs_answer = prompt.needs_answer();
        self.dispatch(Event::Request(Request {
            device: device.clone(),
            prompt,
            reply: needs_answer.then_some(reply),
        }));
        if !needs_answer {
            return;
        }
        let client = Rc::clone(self);
        glib::spawn_future_local(async move {
            match answer.recv().await {
                Ok(Answer::Reject) | Err(_) => {
                    client.dispatch(Event::Pairing(Step::Failed(device, Failure::Canceled)));
                }
                Ok(_) => client.finish_pairing(device),
            }
        });
    }

    fn finish_pairing(self: &Rc<Self>, device: String) {
        self.dispatch(Event::Pairing(Step::Paired(device.clone())));
        self.change(|snapshot| {
            if let Some(found) = snapshot.devices.iter_mut().find(|found| found.path == device) {
                found.paired = true;
                found.trusted = true;
                found.connected = true;
            }
        });
        self.dispatch(Event::Pairing(Step::Finished(device)));
    }

    fn change(&self, edit: impl FnOnce(&mut Snapshot)) {
        let mut snapshot = self.snapshot();
        edit(&mut snapshot);
        self.dispatch(Event::Snapshot(snapshot));
    }

    fn later(self: &Rc<Self>, action: impl FnOnce(&Rc<Client>) + 'static) {
        let client = Rc::clone(self);
        glib::timeout_add_local_once(Duration::from_millis(900), move || action(&client));
    }

    fn play(self: &Rc<Self>, command: Command) {
        match command {
            Command::SetPowered(on) => self.change(|snapshot| {
                if let Some(adapter) = snapshot.adapter.as_mut() {
                    adapter.powered = on;
                }
                if on {
                    snapshot.radio = crate::model::Radio::Free;
                }
            }),
            Command::Connect(device) | Command::Disconnect(device) => self.later(move |client| {
                let mut op = Op::Connect;
                client.change(|snapshot| {
                    if let Some(found) = snapshot.devices.iter_mut().find(|found| found.path == device) {
                        found.connected = !found.connected;
                        if !found.connected {
                            op = Op::Disconnect;
                        }
                    }
                });
                client.dispatch(Event::Done { device, op, failure: None });
            }),
            Command::SetTrusted(device, on) => self.change(|snapshot| {
                if let Some(found) = snapshot.devices.iter_mut().find(|found| found.path == device) {
                    found.trusted = on;
                }
            }),
            Command::Forget(device) => self.change(|snapshot| {
                snapshot.devices.retain(|found| found.path != device);
            }),
            Command::Pair(device) => {
                self.dispatch(Event::Pairing(Step::Start(device.clone())));
                self.later(move |client| client.ask(device, Prompt::Confirm(492_017)));
            }
            Command::CancelPairing(device) => {
                self.dispatch(Event::Pairing(Step::Failed(device, Failure::Canceled)));
            }
            Command::SetVisible(_) | Command::Discover(_) => {}
        }
    }
}
