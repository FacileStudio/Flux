//! The NetworkManager side of the page, on its own thread.
//!
//! [`Backend::start`] spawns a worker with a single-threaded tokio runtime and
//! one D-Bus connection. The page sends [`Command`]s; the worker answers with
//! [`Event`]s. It subscribes to every signal NetworkManager emits under its
//! object tree and to the bus owner changes of its name, and on any of them
//! rereads the state after a short settle delay, so the list follows scans and
//! state changes without polling. Nothing here knows about GTK.

mod actions;
mod bus;
mod read;
mod worker;

use async_channel::{Receiver, Sender};

use crate::model::{wifi::Failure, Snapshot, Status};

pub use actions::Join;

/// What the page asks the worker to do.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Refresh,
    SetWifi(bool),
    SetAirplane(bool),
    Scan,
    Connect(Join),
    Disconnect,
    Forget(String),
    SetAutoconnect(String, bool),
    SetVpn(String, bool),
}

/// What the worker tells the page.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Snapshot(Box<Snapshot>),
    Connecting(String),
    Connected(String),
    Failed(String, Failure),
    ActionFailed(String),
}

/// The page's handle on the worker. Dropping every clone stops it.
#[derive(Clone)]
pub struct Backend {
    commands: Sender<Command>,
}

impl Backend {
    /// Start the worker and hand back the handle and the event stream.
    pub fn start() -> (Backend, Receiver<Event>) {
        let (commands, inbox) = async_channel::unbounded();
        let (events, outbox) = async_channel::unbounded();
        let spawned = std::thread::Builder::new()
            .name("settings-network".to_string())
            .spawn({
                let events = events.clone();
                move || worker::run(inbox, events)
            });
        if let Err(error) = spawned {
            let _ = events.try_send(Event::Snapshot(Box::new(Snapshot::empty(Status::Error(
                error.to_string(),
            )))));
        }
        (Backend { commands }, outbox)
    }

    /// A handle whose commands go nowhere, for previews.
    pub fn detached() -> Backend {
        let (commands, _) = async_channel::unbounded();
        Backend { commands }
    }

    /// Queue `command`. A stopped worker drops it.
    pub fn send(&self, command: Command) {
        let _ = self.commands.try_send(command);
    }
}

/// Deliver `event`, reporting whether the page is still listening.
fn emit(events: &Sender<Event>, event: Event) -> bool {
    events.try_send(event).is_ok()
}
