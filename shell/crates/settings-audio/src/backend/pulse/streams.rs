//! The streams the page opens itself: peak meters on the default output's monitor and the
//! default input, and the left-then-right test sound.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use libpulse_binding as pa;
use pa::context::Context;
use pa::def::BufferAttr;
use pa::proplist::{properties, Proplist};
use pa::sample::{Format, Spec};
use pa::stream::{FlagSet as StreamFlags, PeekResult, SeekMode, Stream as PaStream};

use super::{Event, Shared, ALL};
use crate::model::{self, Direction};

const TEST_RATE: u32 = 48_000;
const FRAME: usize = 8;

/// Open, move or close the meters to match what the page wants and the current defaults.
pub(super) fn update_meters(context: &Rc<RefCell<Context>>, shared: &Rc<RefCell<Shared>>) {
    let (wanted, events) = {
        let state = shared.borrow();
        let wanted = if state.meters.on && state.loaded == ALL {
            let snapshot = &state.snapshot;
            let output = snapshot
                .default_device(Direction::Output)
                .and_then(|sink| sink.monitor.clone());
            let input = snapshot
                .default_device(Direction::Input)
                .map(|source| source.name.clone());
            (output, input)
        } else {
            (None, None)
        };
        (wanted, state.events.clone())
    };
    if shared.borrow().meters.sources == wanted {
        return;
    }
    let Ok(mut context) = context.try_borrow_mut() else {
        return;
    };
    let mut opened = Vec::new();
    if let Some(source) = &wanted.0 {
        opened.extend(open_meter(
            &mut context,
            source,
            Direction::Output,
            events.clone(),
        ));
    }
    if let Some(source) = &wanted.1 {
        opened.extend(open_meter(&mut context, source, Direction::Input, events));
    }
    let mut state = shared.borrow_mut();
    state.meters.streams = opened;
    state.meters.sources = wanted;
}

fn open_meter(
    context: &mut Context,
    source: &str,
    direction: Direction,
    events: async_channel::Sender<Event>,
) -> Option<Rc<RefCell<PaStream>>> {
    let spec = Spec {
        format: Format::F32le,
        rate: 25,
        channels: 1,
    };
    let stream = Rc::new(RefCell::new(PaStream::new(
        context,
        "Peak detect",
        &spec,
        None,
    )?));
    let weak = Rc::downgrade(&stream);
    stream
        .borrow_mut()
        .set_read_callback(Some(Box::new(move |_| {
            let peak = with_stream(&weak, drain_peaks).flatten();
            if let Some(peak) = peak {
                let _ = events.try_send(Event::Level(direction, peak));
            }
        })));
    let attr = BufferAttr {
        maxlength: u32::MAX,
        tlength: u32::MAX,
        prebuf: u32::MAX,
        minreq: u32::MAX,
        fragsize: 4,
    };
    let flags = StreamFlags::PEAK_DETECT | StreamFlags::ADJUST_LATENCY | StreamFlags::DONT_MOVE;
    stream
        .borrow_mut()
        .connect_record(Some(source), Some(&attr), flags)
        .ok()?;
    Some(stream)
}

fn with_stream<R>(
    weak: &Weak<RefCell<PaStream>>,
    action: impl FnOnce(&mut PaStream) -> R,
) -> Option<R> {
    let stream = weak.upgrade()?;
    let mut stream = stream.try_borrow_mut().ok()?;
    Some(action(&mut stream))
}

fn drain_peaks(stream: &mut PaStream) -> Option<f32> {
    let mut peak = None;
    loop {
        match stream.peek() {
            Ok(PeekResult::Data(data)) => {
                let tail = data
                    .len()
                    .checked_sub(4)
                    .and_then(|start| data.get(start..));
                if let Some(bytes) = tail.and_then(|bytes| <[u8; 4]>::try_from(bytes).ok()) {
                    peak = Some(f32::from_le_bytes(bytes).abs());
                }
                let _ = stream.discard();
            }
            Ok(PeekResult::Hole(_)) => {
                let _ = stream.discard();
            }
            Ok(PeekResult::Empty) | Err(_) => return peak,
        }
    }
}

/// Play the test tone on `sink`; the returned stream must be kept until it drains.
pub(super) fn play_test(context: &mut Context, sink: &str) -> Option<Rc<RefCell<PaStream>>> {
    let spec = Spec {
        format: Format::F32le,
        rate: TEST_RATE,
        channels: 2,
    };
    let mut props = Proplist::new()?;
    props.set_str(properties::MEDIA_ROLE, "test").ok()?;
    props.set_str(properties::MEDIA_NAME, "Speaker test").ok()?;
    let stream = PaStream::new_with_proplist(context, "Speaker test", &spec, None, &mut props)?;
    let stream = Rc::new(RefCell::new(stream));
    let bytes: Vec<u8> = model::test_tone(TEST_RATE)
        .iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect();
    let offset = Cell::new(0usize);
    let weak = Rc::downgrade(&stream);
    stream
        .borrow_mut()
        .set_write_callback(Some(Box::new(move |wanted| {
            let drained = weak.clone();
            with_stream(&weak, |stream| {
                feed(stream, &bytes, &offset, wanted, drained)
            });
        })));
    stream
        .borrow_mut()
        .connect_playback(Some(sink), None, StreamFlags::NOFLAGS, None, None)
        .ok()?;
    Some(stream)
}

fn feed(
    stream: &mut PaStream,
    bytes: &[u8],
    offset: &Cell<usize>,
    wanted: usize,
    weak: Weak<RefCell<PaStream>>,
) {
    let start = offset.get();
    if start >= bytes.len() {
        return;
    }
    let end = (start + wanted - wanted % FRAME).clamp(start, bytes.len());
    if let Some(chunk) = bytes.get(start..end) {
        let _ = stream.write_copy(chunk, 0, SeekMode::Relative);
    }
    offset.set(end);
    if end < bytes.len() {
        return;
    }
    stream.drain(Some(Box::new(move |_| {
        with_stream(&weak, |stream| {
            let _ = stream.disconnect();
        });
    })));
}
