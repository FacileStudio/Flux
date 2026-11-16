//! One connection's worth of BlueZ: a mirror of its ObjectManager tree kept
//! current from signals, turned into a [`Snapshot`] after each burst of
//! changes. The commands the page sends live in [`commands`].

mod commands;
mod mirror;

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use async_channel::{Receiver, Sender};
use futures_util::{Stream, StreamExt};
use zbus::message::Type;
use zbus::zvariant::ObjectPath;
use zbus::{Connection, MatchRule, Message, MessageStream};

use super::agent::{Agent, AGENT_PATH};
use super::{rfkill, Command, Event};
use crate::model;
use mirror::{snapshot, Cache, ManagedObjects};
use commands::already_exists;

const BLUEZ: &str = "org.bluez";
const ADAPTER: &str = "org.bluez.Adapter1";
const DEVICE: &str = "org.bluez.Device1";
const BATTERY: &str = "org.bluez.Battery1";
const AGENT_MANAGER: &str = "org.bluez.AgentManager1";
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";
const OBJECT_MANAGER: &str = "org.freedesktop.DBus.ObjectManager";
const DBUS: &str = "org.freedesktop.DBus";

const SETTLE: Duration = Duration::from_millis(80);
const MAX_DELAY: Duration = Duration::from_millis(400);
const IDLE: Duration = Duration::from_secs(2);
const DISCOVERABLE_SECONDS: u32 = 180;

/// A live connection to the bus and what it knows of BlueZ.
pub struct Session {
    connection: Connection,
    events: Sender<Event>,
    cache: Mutex<Cache>,
}

async fn stream(connection: &Connection, rule: zbus::Result<MatchRule<'static>>) -> zbus::Result<MessageStream> {
    MessageStream::for_match_rule(rule?, connection, None).await
}

impl Session {
    pub fn new(connection: Connection, events: Sender<Event>) -> Self {
        Session {
            connection,
            events,
            cache: Mutex::new(Cache::default()),
        }
    }

    fn cache(&self) -> MutexGuard<'_, Cache> {
        self.cache.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Serve the agent, mirror BlueZ and run commands until the bus goes away.
    pub async fn run(self: &Arc<Self>, commands: &Receiver<Command>) -> zbus::Result<()> {
        self.connection
            .object_server()
            .at(AGENT_PATH, Agent::new(self.events.clone()))
            .await?;
        let objects = stream(
            &self.connection,
            MatchRule::builder()
                .msg_type(Type::Signal)
                .interface(OBJECT_MANAGER)
                .and_then(|rule| rule.path("/"))
                .map(|rule| rule.build()),
        )
        .await?;
        let properties = stream(
            &self.connection,
            MatchRule::builder()
                .msg_type(Type::Signal)
                .interface(PROPERTIES)
                .and_then(|rule| rule.member("PropertiesChanged"))
                .and_then(|rule| rule.path_namespace("/org/bluez"))
                .map(|rule| rule.build()),
        )
        .await?;
        let owner = stream(
            &self.connection,
            MatchRule::builder()
                .msg_type(Type::Signal)
                .sender(DBUS)
                .and_then(|rule| rule.interface(DBUS))
                .and_then(|rule| rule.member("NameOwnerChanged"))
                .and_then(|rule| rule.arg(0, BLUEZ))
                .map(|rule| rule.build()),
        )
        .await?;
        let signals = futures_util::stream::select(objects, futures_util::stream::select(properties, owner));
        self.reload().await;
        self.read_radio();
        self.emit().await;
        let worker = Arc::clone(self);
        let queue = commands.clone();
        let commander = tokio::spawn(async move {
            while let Ok(command) = queue.recv().await {
                let session = Arc::clone(&worker);
                tokio::spawn(async move { session.execute(command).await });
            }
        });
        self.watch(signals).await;
        commander.abort();
        Ok(())
    }

    async fn watch(&self, mut signals: impl Stream<Item = zbus::Result<Message>> + Unpin) {
        let mut dirty = false;
        let mut last = Instant::now();
        while !self.events.is_closed() {
            let wait = if dirty { SETTLE } else { IDLE };
            match tokio::time::timeout(wait, signals.next()).await {
                Ok(Some(Ok(message))) => {
                    dirty |= self.apply(&message).await;
                    if last.elapsed() < MAX_DELAY {
                        continue;
                    }
                }
                Ok(Some(Err(_))) => continue,
                Ok(None) => return,
                Err(_) => dirty |= self.read_radio(),
            }
            if dirty {
                self.emit().await;
                dirty = false;
                last = Instant::now();
            }
        }
    }

    async fn apply(&self, message: &Message) -> bool {
        let header = message.header();
        let member = header.member().map(|member| member.to_string()).unwrap_or_default();
        let sender = header.sender().map(|sender| sender.to_string());
        let path = header.path().map(|path| path.to_string()).unwrap_or_default();
        let body = message.body();
        if member == "NameOwnerChanged" {
            return self.owner_changed(&body).await;
        }
        let mut cache = self.cache();
        if sender.is_none() || sender != cache.owner {
            return false;
        }
        cache.apply(&member, &path, &body)
    }

    async fn owner_changed(&self, body: &zbus::message::Body) -> bool {
        let Ok((name, _, owner)) = body.deserialize::<(String, String, String)>() else {
            return false;
        };
        if name != BLUEZ {
            return false;
        }
        if owner.is_empty() {
            let mut cache = self.cache();
            cache.service = false;
            cache.owner = None;
            cache.objects.clear();
        } else {
            self.reload().await;
        }
        true
    }

    async fn reload(&self) {
        let objects = self
            .connection
            .call_method(Some(BLUEZ), "/", Some(OBJECT_MANAGER), "GetManagedObjects", &())
            .await
            .and_then(|reply| reply.body().deserialize::<ManagedObjects>());
        let owner = self
            .connection
            .call_method(Some(DBUS), "/org/freedesktop/DBus", Some(DBUS), "GetNameOwner", &(BLUEZ,))
            .await
            .and_then(|reply| reply.body().deserialize::<String>());
        let found = objects.is_ok() && owner.is_ok();
        {
            let mut cache = self.cache();
            cache.service = found;
            cache.owner = owner.ok();
            cache.objects = objects
                .map(|objects| objects.into_iter().map(|(path, interfaces)| (path.to_string(), interfaces)).collect())
                .unwrap_or_default();
        }
        if found {
            self.register_agent().await;
        }
    }

    async fn register_agent(&self) {
        let path = ObjectPath::from_static_str_unchecked(AGENT_PATH);
        let registered = self
            .connection
            .call_method(Some(BLUEZ), "/org/bluez", Some(AGENT_MANAGER), "RegisterAgent", &(&path, "KeyboardDisplay"))
            .await;
        if let Err(error) = registered.as_ref().map(|_| ()) {
            if !already_exists(error) {
                eprintln!("settings-bluetooth: cannot register the pairing agent: {error}");
                return;
            }
        }
        let default = self
            .connection
            .call_method(Some(BLUEZ), "/org/bluez", Some(AGENT_MANAGER), "RequestDefaultAgent", &(&path,))
            .await;
        if let Err(error) = default {
            eprintln!("settings-bluetooth: cannot make the agent the default: {error}");
        }
    }

    fn read_radio(&self) -> bool {
        let radio = model::radio(&rfkill::read());
        let mut cache = self.cache();
        let changed = cache.radio != radio;
        cache.radio = radio;
        changed
    }

    async fn emit(&self) {
        let snapshot = snapshot(&self.cache());
        self.send(Event::Snapshot(snapshot)).await;
    }

    async fn send(&self, event: Event) {
        let _ = self.events.send(event).await;
    }

}
