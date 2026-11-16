//! Reading the server's lists into the snapshot. One list query per kind of object runs at a
//! time; a change that lands while it runs queues exactly one more.

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};

use libpulse_binding as pa;
use pa::callbacks::ListResult;
use pa::context::introspect::{CardInfo, Introspector, SinkInfo, SinkInputInfo, SourceInfo};
use pa::context::Context;
use pa::def::PortAvailable;
use pa::proplist::Proplist;
use pa::volume::ChannelVolumes;

use super::{streams, Shared, SERVER, SINKS, SOURCES, STREAMS};
use crate::model::{Availability, Card, Device, Port, Profile, Stream};

/// Reload one kind of object (`SERVER`, `SINKS`, ...) and send a snapshot when it lands.
pub(super) fn refresh(bit: u8, weak: &Weak<RefCell<Context>>, shared: &Rc<RefCell<Shared>>) {
    {
        let mut state = shared.borrow_mut();
        if state.inflight & bit != 0 {
            state.dirty |= bit;
            return;
        }
        state.inflight |= bit;
    }
    let context = weak.upgrade();
    let Some(context) = context
        .as_ref()
        .and_then(|context| context.try_borrow().ok())
    else {
        shared.borrow_mut().inflight &= !bit;
        return;
    };
    let introspect = context.introspect();
    let finish = Finish {
        bit,
        weak: weak.clone(),
        shared: Rc::clone(shared),
    };
    match bit {
        SERVER => load_server(&introspect, finish),
        SINKS => load_sinks(&introspect, finish),
        SOURCES => load_sources(&introspect, finish),
        STREAMS => load_streams(&introspect, finish),
        _ => load_cards(&introspect, finish),
    }
}

fn load_server(introspect: &Introspector, finish: Finish) {
    introspect.get_server_info(move |info| {
        let sink = info.default_sink_name.as_ref().map(|name| name.to_string());
        let source = info
            .default_source_name
            .as_ref()
            .map(|name| name.to_string());
        finish.apply(|shared| {
            shared.snapshot.default_sink = sink.clone();
            shared.snapshot.default_source = source.clone();
        });
    });
}

fn load_sinks(introspect: &Introspector, finish: Finish) {
    let mut list = Vec::new();
    introspect.get_sink_info_list(move |result| match result {
        ListResult::Item(info) => list.push((sink(info), info.volume)),
        ListResult::End => {
            let items = std::mem::take(&mut list);
            finish.apply(|shared| {
                shared.snapshot.sinks = store(shared, SINKS, items, |device| device.index)
            });
        }
        ListResult::Error => finish.fail(),
    });
}

fn load_sources(introspect: &Introspector, finish: Finish) {
    let mut list = Vec::new();
    introspect.get_source_info_list(move |result| match result {
        ListResult::Item(info) => list.push((source(info), info.volume)),
        ListResult::End => {
            let items = std::mem::take(&mut list);
            finish.apply(|shared| {
                shared.snapshot.sources = store(shared, SOURCES, items, |device| device.index)
            });
        }
        ListResult::Error => finish.fail(),
    });
}

fn load_streams(introspect: &Introspector, finish: Finish) {
    let mut list = Vec::new();
    introspect.get_sink_input_info_list(move |result| match result {
        ListResult::Item(info) if info.has_volume => list.push((stream(info), info.volume)),
        ListResult::Item(_) => {}
        ListResult::End => {
            let items = std::mem::take(&mut list);
            finish.apply(|shared| {
                shared.snapshot.streams = store(shared, STREAMS, items, |stream| stream.index)
            });
        }
        ListResult::Error => finish.fail(),
    });
}

fn load_cards(introspect: &Introspector, finish: Finish) {
    let mut list = Vec::new();
    introspect.get_card_info_list(move |result| match result {
        ListResult::Item(info) => list.push(card(info)),
        ListResult::End => {
            let items = std::mem::take(&mut list);
            finish.apply(|shared| shared.snapshot.cards = items.clone());
        }
        ListResult::Error => finish.fail(),
    });
}

fn store<T: Clone>(
    shared: &mut Shared,
    bit: u8,
    items: Vec<(T, ChannelVolumes)>,
    index: fn(&T) -> u32,
) -> Vec<T> {
    shared.volumes.retain(|(kind, _), _| *kind != bit);
    items
        .into_iter()
        .map(|(item, volumes)| {
            shared.volumes.insert((bit, index(&item)), volumes);
            item
        })
        .collect()
}

struct Finish {
    bit: u8,
    weak: Weak<RefCell<Context>>,
    shared: Rc<RefCell<Shared>>,
}

impl Finish {
    fn apply(&self, change: impl FnOnce(&mut Shared)) {
        let again = {
            let mut shared = self.shared.borrow_mut();
            change(&mut shared);
            shared.loaded |= self.bit;
            shared.inflight &= !self.bit;
            let again = shared.dirty & self.bit != 0;
            shared.dirty &= !self.bit;
            again
        };
        if again {
            refresh(self.bit, &self.weak, &self.shared);
            return;
        }
        if let (true, Some(context)) = (
            self.bit & (SERVER | SINKS | SOURCES) != 0,
            self.weak.upgrade(),
        ) {
            streams::update_meters(&context, &self.shared);
        }
        self.shared.borrow().emit();
    }

    fn fail(&self) {
        let mut shared = self.shared.borrow_mut();
        shared.inflight &= !self.bit;
        shared.dirty &= !self.bit;
    }
}

fn text(value: &Option<Cow<'_, str>>) -> String {
    value
        .as_ref()
        .map(|value| value.to_string())
        .unwrap_or_default()
}

fn port(
    name: &Option<Cow<'_, str>>,
    description: &Option<Cow<'_, str>>,
    available: PortAvailable,
) -> Port {
    Port {
        name: text(name),
        description: text(description),
        available: match available {
            PortAvailable::Yes => Availability::Yes,
            PortAvailable::No => Availability::No,
            _ => Availability::Unknown,
        },
    }
}

fn properties_of(list: &Proplist) -> BTreeMap<String, String> {
    list.iter()
        .filter_map(|key| list.get_str(&key).map(|value| (key, value)))
        .collect()
}

fn sink(info: &SinkInfo) -> Device {
    Device {
        index: info.index,
        name: text(&info.name),
        description: text(&info.description),
        properties: properties_of(&info.proplist),
        ports: info
            .ports
            .iter()
            .map(|p| port(&p.name, &p.description, p.available))
            .collect(),
        active_port: info.active_port.as_ref().map(|p| text(&p.name)),
        volume: info.volume.max().0,
        muted: info.mute,
        card: info.card,
        monitor: info
            .monitor_source_name
            .as_ref()
            .map(|name| name.to_string()),
        is_monitor: false,
    }
}

fn source(info: &SourceInfo) -> Device {
    Device {
        index: info.index,
        name: text(&info.name),
        description: text(&info.description),
        properties: properties_of(&info.proplist),
        ports: info
            .ports
            .iter()
            .map(|p| port(&p.name, &p.description, p.available))
            .collect(),
        active_port: info.active_port.as_ref().map(|p| text(&p.name)),
        volume: info.volume.max().0,
        muted: info.mute,
        card: info.card,
        monitor: None,
        is_monitor: info.monitor_of_sink.is_some(),
    }
}

fn stream(info: &SinkInputInfo) -> Stream {
    Stream {
        index: info.index,
        sink: info.sink,
        volume: info.volume.max().0,
        muted: info.mute,
        properties: properties_of(&info.proplist),
    }
}

fn card(info: &CardInfo) -> Card {
    Card {
        index: info.index,
        name: text(&info.name),
        profiles: info
            .profiles
            .iter()
            .map(|profile| Profile {
                name: text(&profile.name),
                description: text(&profile.description),
                priority: profile.priority,
                available: profile.available,
            })
            .collect(),
        active_profile: info
            .active_profile
            .as_ref()
            .map(|profile| text(&profile.name)),
    }
}
