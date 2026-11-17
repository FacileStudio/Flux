//! The BlueZ side: a worker thread that owns the system bus connection, mirrors
//! `org.bluez`'s objects, serves the pairing agent and runs commands. The GTK
//! side only talks to it through two channels.
//!
//! `FLUX_BLUETOOTH_BUS` replaces the system bus with another address. It exists
//! for the tests, which run BlueZ's python-dbusmock template on a private bus.

mod agent;
mod bluez;
mod rfkill;

use std::sync::Arc;
use std::time::Duration;

use async_channel::{Receiver, Sender};

use crate::model::{Failure, Prompt, Snapshot, Step};

pub use agent::AGENT_PATH;

/// What the page asks BlueZ to do. Device and adapter paths are D-Bus object paths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    SetPowered(bool),
    SetVisible(bool),
    Discover(bool),
    Connect(String),
    Disconnect(String),
    Pair(String),
    CancelPairing(String),
    SetTrusted(String, bool),
    Forget(String),
}

/// The person's answer to an agent request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    Accept,
    Reject,
    Pin(String),
    Passkey(u32),
}

/// An agent request waiting on the person. `reply` is `None` when BlueZ only
/// asks to show a code.
#[derive(Clone, Debug)]
pub struct Request {
    pub device: String,
    pub prompt: Prompt,
    pub reply: Option<Sender<Answer>>,
}

/// Which command a [`Event::Done`] reports on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Power,
    Connect,
    Disconnect,
    Trust,
    Forget,
}

/// What the worker tells the page.
#[derive(Clone, Debug)]
pub enum Event {
    Snapshot(Snapshot),
    Request(Request),
    RequestCancelled,
    Pairing(Step),
    Done { device: String, op: Op, failure: Option<Failure> },
}

/// Both ends the GTK side keeps.
pub struct Backend {
    pub commands: Sender<Command>,
    pub events: Receiver<Event>,
}

/// Start the worker on the system bus, or on `FLUX_BLUETOOTH_BUS` when set.
pub fn start() -> Backend {
    start_on(std::env::var("FLUX_BLUETOOTH_BUS").ok().filter(|address| !address.is_empty()))
}

/// Start the worker on the bus at `address`, or the system bus for `None`.
pub fn start_on(address: Option<String>) -> Backend {
    let (commands, command_queue) = async_channel::unbounded();
    let (event_sink, events) = async_channel::unbounded();
    let spawned = std::thread::Builder::new()
        .name("bluetooth".into())
        .spawn(move || work(address, command_queue, event_sink));
    if let Err(error) = spawned {
        eprintln!("settings-bluetooth: cannot start the worker: {error}");
    }
    Backend { commands, events }
}

fn work(address: Option<String>, commands: Receiver<Command>, events: Sender<Event>) {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build();
    let runtime = match runtime {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("settings-bluetooth: cannot start the runtime: {error}");
            let _ = events.send_blocking(Event::Snapshot(Snapshot::default()));
            return;
        }
    };
    runtime.block_on(run(address, commands, events));
}

async fn connect(address: &Option<String>) -> zbus::Result<zbus::Connection> {
    match address {
        Some(address) => zbus::connection::Builder::address(address.as_str())?.build().await,
        None => zbus::Connection::system().await,
    }
}

async fn run(address: Option<String>, commands: Receiver<Command>, events: Sender<Event>) {
    loop {
        match connect(&address).await {
            Ok(connection) => {
                let session = Arc::new(bluez::Session::new(connection, events.clone()));
                if let Err(error) = session.run(&commands).await {
                    eprintln!("settings-bluetooth: bus session ended: {error}");
                }
            }
            Err(error) => eprintln!("settings-bluetooth: cannot reach the bus: {error}"),
        }
        if events.send(Event::Snapshot(Snapshot::default())).await.is_err() {
            return;
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
