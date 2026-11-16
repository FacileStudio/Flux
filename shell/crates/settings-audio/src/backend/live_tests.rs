//! Against a real server, never this machine's: run `cargo test -p settings-audio -- --ignored`
//! with `PULSE_SERVER` pointing at a private daemon that has null sinks `a` and `b` and a null
//! source `mic` (see the crate README).

use std::process::{Child, Command as Process, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::{connect, Backend, Command, Event, Target};
use crate::model::{self, Direction, Snapshot};

const WAIT: Duration = Duration::from_secs(5);

fn until(events: &async_channel::Receiver<Event>, check: impl Fn(&Event) -> bool) -> Option<Event> {
    let deadline = Instant::now() + WAIT;
    while Instant::now() < deadline {
        match events.try_recv() {
            Ok(event) if check(&event) => return Some(event),
            Ok(_) => {}
            Err(async_channel::TryRecvError::Empty) => {
                std::thread::sleep(Duration::from_millis(10))
            }
            Err(async_channel::TryRecvError::Closed) => return None,
        }
    }
    None
}

fn snapshot(
    events: &async_channel::Receiver<Event>,
    check: impl Fn(&Snapshot) -> bool,
) -> Snapshot {
    match until(
        events,
        |event| matches!(event, Event::Snapshot(s) if check(s)),
    ) {
        Some(Event::Snapshot(snapshot)) => *snapshot,
        _ => panic!("no matching snapshot within {WAIT:?}"),
    }
}

fn index(snapshot: &Snapshot, direction: Direction, name: &str) -> u32 {
    let device = snapshot
        .devices(direction)
        .into_iter()
        .find(|device| device.name == name);
    device.map(|device| device.index).unwrap_or(u32::MAX)
}

fn sink_of_player(snapshot: &Snapshot) -> Option<u32> {
    snapshot
        .streams
        .iter()
        .find(|stream| !is_own(stream))
        .map(|stream| stream.sink)
}

struct Player(Child);

impl Drop for Player {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
#[ignore]
fn private_server_round_trip() {
    assert!(
        std::env::var_os("PULSE_SERVER").is_some(),
        "set PULSE_SERVER to a private daemon"
    );
    let (backend, events) = connect();
    let first = snapshot(&events, |s| s.devices(Direction::Output).len() >= 2);
    let a = index(&first, Direction::Output, "a");
    let b = index(&first, Direction::Output, "b");
    assert!(first
        .devices(Direction::Input)
        .iter()
        .any(|device| device.name == "mic"));
    assert!(first.sources.iter().any(|source| source.is_monitor));
    println!(
        "listed {} sinks, {} sources",
        first.sinks.len(),
        first.sources.len()
    );
    let _player = switch_moves_streams(backend.as_ref(), &events, a, b);
    volumes_and_mute(backend.as_ref(), &events, b);
    meters_and_test(backend.as_ref(), &events);
    drop(backend);
    assert_eq!(crate::set_volume_step(5), Some(47));
    println!("volume step raised b to 47% and unmuted it");
}

fn switch_moves_streams(
    backend: &dyn Backend,
    events: &async_channel::Receiver<Event>,
    a: u32,
    b: u32,
) -> Player {
    backend.send(Command::SetDefault(Direction::Output, "a".to_string()));
    snapshot(events, |s| s.default_sink.as_deref() == Some("a"));
    let player = Player(
        Process::new("pacat")
            .args(["--playback", "--device=a", "/dev/zero"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|error| panic!("pacat: {error}")),
    );
    snapshot(events, |s| sink_of_player(s) == Some(a));
    backend.send(Command::SetDefault(Direction::Output, "b".to_string()));
    snapshot(events, |s| {
        s.default_sink.as_deref() == Some("b") && sink_of_player(s) == Some(b)
    });
    println!("switched default to b and moved the running stream");
    player
}

fn volumes_and_mute(backend: &dyn Backend, events: &async_channel::Receiver<Event>, b: u32) {
    let (done, heard) = mpsc::channel();
    backend.send(Command::SetVolume(
        Target::Device(Direction::Output, b),
        model::raw(42),
        Some(done),
    ));
    assert_eq!(heard.recv_timeout(WAIT).ok(), Some(true));
    snapshot(events, |s| {
        s.sinks.iter().any(|d| d.index == b && d.percent() == 42)
    });
    println!("set volume to 42%");
    backend.send(Command::SetMute(Target::Device(Direction::Output, b), true));
    let muted = snapshot(events, |s| s.sinks.iter().any(|d| d.index == b && d.muted));
    println!("muted");
    let player = muted
        .streams
        .iter()
        .find(|stream| !is_own(stream))
        .map(|stream| stream.index);
    let player = player.unwrap_or(u32::MAX);
    backend.send(Command::SetVolume(
        Target::Stream(player),
        model::raw(30),
        None,
    ));
    snapshot(events, |s| {
        s.streams
            .iter()
            .any(|x| x.index == player && model::percent(x.volume) == 30)
    });
    println!("set the app stream to 30%");
}

fn meters_and_test(backend: &dyn Backend, events: &async_channel::Receiver<Event>) {
    backend.send(Command::Meters(true));
    assert!(until(events, |event| matches!(
        event,
        Event::Level(Direction::Input, _)
    ))
    .is_some());
    assert!(until(events, |event| matches!(
        event,
        Event::Level(Direction::Output, _)
    ))
    .is_some());
    backend.send(Command::Meters(false));
    println!("meters report levels");
    backend.send(Command::Test("b".to_string()));
    snapshot(events, |s| s.streams.iter().any(is_own));
    println!("test sound plays");
}

fn is_own(stream: &model::Stream) -> bool {
    stream
        .properties
        .get("application.name")
        .map(String::as_str)
        == Some(model::OWN_APP)
}
