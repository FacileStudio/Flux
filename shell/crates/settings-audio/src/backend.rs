//! The seam between the page and the sound server. The page sends [`Command`]s and reads
//! [`Event`]s; whatever sits behind [`Backend`] talks to the server off the GTK thread.

#[cfg(test)]
mod live_tests;
mod preview;
mod pulse;

use std::sync::mpsc;

use crate::model::{Direction, Options, Snapshot};

/// What a slider or a mute button acts on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Device(Direction, u32),
    Stream(u32),
}

/// A change the page asks for.
#[derive(Debug)]
pub enum Command {
    /// Make a device the default and move every running stream to it.
    SetDefault(Direction, String),
    /// Set a raw volume, keeping the balance between channels. `done` hears whether it worked.
    SetVolume(Target, u32, Option<mpsc::Sender<bool>>),
    SetMute(Target, bool),
    SetPort(Direction, u32, String),
    SetProfile(u32, String),
    /// Play the left-then-right test sound on this sink.
    Test(String),
    /// Start or stop the live level meters of the default output and input.
    Meters(bool),
}

/// What the page hears back.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Snapshot(Box<Snapshot>),
    Level(Direction, f32),
    Unavailable,
    Options(Options),
}

/// Something that carries commands to a sound server.
pub trait Backend {
    fn send(&self, command: Command);
}

/// Connect to the session's sound server through the PulseAudio protocol.
pub fn connect() -> (Box<dyn Backend>, async_channel::Receiver<Event>) {
    let (events, receiver) = async_channel::unbounded();
    (Box::new(pulse::spawn(events)), receiver)
}

/// A backend that plays back sample data and touches no service, for design checks.
pub fn preview(mode: &str) -> (Box<dyn Backend>, async_channel::Receiver<Event>) {
    let (events, receiver) = async_channel::unbounded();
    (Box::new(preview::Preview::new(events, mode)), receiver)
}

/// Connect, wait for the first snapshot, and return it with the live connection.
pub fn connect_blocking(timeout: std::time::Duration) -> Option<(Box<dyn Backend>, Snapshot)> {
    let (backend, receiver) = connect();
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        match receiver.recv_blocking() {
            Ok(Event::Snapshot(snapshot)) => return Some((backend, *snapshot)),
            Ok(Event::Unavailable) | Err(_) => return None,
            Ok(_) => {}
        }
    }
    None
}
