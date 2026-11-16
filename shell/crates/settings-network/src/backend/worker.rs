//! The worker loop: commands in, signals in, snapshots and outcomes out.

use std::time::Duration;

use async_channel::{Receiver, Sender};
use futures_util::StreamExt;
use tokio::time::{sleep_until, Instant};
use zbus::{Connection, MatchRule, MessageStream};

use super::actions::{self, Join};
use super::{bus, emit, read, Command, Event};
use crate::model::wifi::{self, Failure};
use crate::model::{Snapshot, Status};

const SETTLE: Duration = Duration::from_millis(150);
const SCAN_LIMIT: Duration = Duration::from_secs(15);
const JOIN_LIMIT: Duration = Duration::from_secs(60);
const FAR: Duration = Duration::from_secs(3600);

struct Scan {
    last_scan: i64,
    until: Instant,
}

struct Pending {
    ssid: String,
    created: Option<String>,
    until: Instant,
}

struct Worker {
    conn: Connection,
    events: Sender<Event>,
    last: Snapshot,
    scan: Option<Scan>,
    pending: Option<Pending>,
}

/// The thread body: build the runtime and serve until the page goes away.
pub fn run(commands: Receiver<Command>, events: Sender<Event>) {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            emit(
                &events,
                Event::Snapshot(Box::new(Snapshot::empty(Status::Error(error.to_string())))),
            );
            return;
        }
    };
    runtime.block_on(serve(commands, events));
}

async fn serve(commands: Receiver<Command>, events: Sender<Event>) {
    let conn = match bus::connect().await {
        Ok(conn) => conn,
        Err(error) => {
            emit(
                &events,
                Event::Snapshot(Box::new(Snapshot::empty(Status::Error(error.to_string())))),
            );
            while commands.recv().await.is_ok() {}
            return;
        }
    };
    let mut signals = match subscribe(&conn).await {
        Ok(stream) => stream,
        Err(error) => {
            emit(
                &events,
                Event::Snapshot(Box::new(Snapshot::empty(Status::Error(error.to_string())))),
            );
            return;
        }
    };
    let mut worker = Worker {
        conn,
        events,
        last: Snapshot::empty(Status::Loading),
        scan: None,
        pending: None,
    };
    if !worker.publish().await {
        return;
    }
    worker.handle(Command::Scan).await;
    let mut settle: Option<Instant> = None;
    loop {
        let wake = worker.next_deadline(settle);
        tokio::select! {
            command = commands.recv() => match command {
                Ok(command) => worker.handle(command).await,
                Err(_) => return,
            },
            message = signals.next() => {
                if let Some(Ok(message)) = message {
                    worker.signal(&message).await;
                }
                settle.get_or_insert_with(|| Instant::now() + SETTLE);
            },
            () = sleep_until(wake) => {
                settle = None;
                worker.expire().await;
                if !worker.publish().await {
                    return;
                }
            },
        }
    }
}

/// Every signal under NetworkManager's object tree, plus its name coming and going.
async fn subscribe(
    conn: &Connection,
) -> zbus::Result<futures_util::stream::Select<MessageStream, MessageStream>> {
    let tree = MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .path_namespace(bus::NM_PATH)?
        .build();
    let owner = MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .sender("org.freedesktop.DBus")?
        .member("NameOwnerChanged")?
        .arg(0, bus::NM)?
        .build();
    let tree = MessageStream::for_match_rule(tree, conn, Some(256)).await?;
    let owner = MessageStream::for_match_rule(owner, conn, Some(16)).await?;
    Ok(futures_util::stream::select(tree, owner))
}

impl Worker {
    fn next_deadline(&self, settle: Option<Instant>) -> Instant {
        let far = Instant::now() + FAR;
        [
            settle,
            self.scan.as_ref().map(|scan| scan.until),
            self.pending.as_ref().map(|job| job.until),
        ]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(far)
    }

    async fn publish(&mut self) -> bool {
        let mut snapshot = read::snapshot(&self.conn).await;
        let last_scan = snapshot.wifi.as_ref().map(|device| device.last_scan);
        if let Some(scan) = &self.scan {
            if last_scan != Some(scan.last_scan) || Instant::now() >= scan.until {
                self.scan = None;
            }
        }
        snapshot.scanning = self.scan.is_some() && snapshot.wifi_enabled;
        self.last = snapshot.clone();
        emit(&self.events, Event::Snapshot(Box::new(snapshot)))
    }

    fn device(&self) -> Option<String> {
        self.last.wifi.as_ref().map(|device| device.path.clone())
    }

    async fn handle(&mut self, command: Command) {
        let conn = self.conn.clone();
        let result = match command {
            Command::Refresh => Ok(()),
            Command::SetWifi(on) => actions::set_wifi(&conn, on).await,
            Command::SetAirplane(on) => actions::set_airplane(&conn, on).await,
            Command::Scan => self.start_scan().await,
            Command::Connect(join) => {
                self.join(join).await;
                Ok(())
            }
            Command::Disconnect => match self.device() {
                Some(device) => actions::disconnect(&conn, &device).await,
                None => Ok(()),
            },
            Command::Forget(ssid) => actions::forget(&conn, &self.last, &ssid).await,
            Command::SetAutoconnect(ssid, on) => {
                actions::set_autoconnect(&conn, &self.last, &ssid, on).await
            }
            Command::SetVpn(profile, on) => {
                let active = self
                    .last
                    .vpns
                    .iter()
                    .find(|vpn| vpn.path == profile)
                    .and_then(|vpn| vpn.active.clone());
                actions::set_vpn(&conn, &profile, active.as_deref(), on).await
            }
        };
        if let Err(error) = result {
            emit(&self.events, Event::ActionFailed(error.to_string()));
        }
        self.publish().await;
    }

    async fn start_scan(&mut self) -> zbus::Result<()> {
        let Some(device) = self.last.wifi.as_ref().filter(|_| self.last.wifi_enabled) else {
            return Ok(());
        };
        self.scan = Some(Scan {
            last_scan: device.last_scan,
            until: Instant::now() + SCAN_LIMIT,
        });
        let path = device.path.clone();
        match actions::scan(&self.conn, &path).await {
            Ok(()) => Ok(()),
            Err(error) if error.to_string().contains("NotAllowed") => Ok(()),
            Err(error) => {
                self.scan = None;
                Err(error)
            }
        }
    }

    async fn join(&mut self, join: Join) {
        let Some(device) = self.device() else {
            emit(&self.events, Event::Failed(join.ssid, Failure::Other(0)));
            return;
        };
        emit(&self.events, Event::Connecting(join.ssid.clone()));
        match actions::connect(&self.conn, &self.last, &device, &join).await {
            Ok(created) => {
                self.pending = Some(Pending {
                    ssid: join.ssid,
                    created,
                    until: Instant::now() + JOIN_LIMIT,
                });
            }
            Err(error) => {
                emit(&self.events, Event::ActionFailed(error.to_string()));
                emit(&self.events, Event::Failed(join.ssid, Failure::Other(0)));
            }
        }
    }

    async fn signal(&mut self, message: &zbus::Message) {
        let header = message.header();
        let is_state = header
            .member()
            .is_some_and(|member| member.as_str() == "StateChanged")
            && header
                .interface()
                .is_some_and(|iface| iface.as_str() == bus::DEVICE);
        let on_device = header.path().map(|path| path.as_str().to_string()) == self.device();
        if !is_state || !on_device || self.pending.is_none() {
            return;
        }
        let Ok((state, _old, reason)) = message.body().deserialize::<(u32, u32, u32)>() else {
            return;
        };
        match state {
            100 => self.finish(None).await,
            120 => self.finish(Some(wifi::failure(reason))).await,
            _ => {}
        }
    }

    async fn expire(&mut self) {
        if self
            .pending
            .as_ref()
            .is_some_and(|job| Instant::now() >= job.until)
        {
            self.finish(Some(Failure::Timeout)).await;
        }
    }

    async fn finish(&mut self, failure: Option<Failure>) {
        let Some(job) = self.pending.take() else {
            return;
        };
        let Some(failure) = failure else {
            emit(&self.events, Event::Connected(job.ssid));
            return;
        };
        if failure == Failure::WrongPassword {
            if let Some(profile) = &job.created {
                let _ = actions::delete(&self.conn, profile).await;
            }
        }
        emit(&self.events, Event::Failed(job.ssid, failure));
    }
}
