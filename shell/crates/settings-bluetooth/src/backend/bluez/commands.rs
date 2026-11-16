//! The commands the page sends, run against BlueZ.

use std::time::Duration;

use zbus::zvariant::{ObjectPath, OwnedObjectPath, Value};

use super::mirror::{parent, snapshot, value};
use super::{Session, ADAPTER, BLUEZ, DEVICE, DISCOVERABLE_SECONDS, PROPERTIES};
use crate::backend::{rfkill, Command, Event, Op};
use crate::model::{self, Failure, Radio, Step};

/// The failure a zbus error stands for.
fn failure_of(error: &zbus::Error) -> Failure {
    match error {
        zbus::Error::MethodError(name, message, _) => model::failure(name.as_str(), message.as_deref().unwrap_or("")),
        other => model::failure("", &other.to_string()),
    }
}

fn is_error(error: &zbus::Error, suffix: &str) -> bool {
    matches!(error, zbus::Error::MethodError(name, _, _) if name.as_str().ends_with(suffix))
}

pub fn already_exists(error: &zbus::Error) -> bool {
    is_error(error, ".AlreadyExists")
}

fn settled(result: zbus::Result<()>, already: &str) -> zbus::Result<()> {
    match result {
        Err(error) if is_error(&error, already) => Ok(()),
        other => other,
    }
}

impl Session {
    fn adapter_path(&self) -> Option<String> {
        snapshot(&self.cache()).adapter.map(|adapter| adapter.path)
    }

    fn adapter_of(&self, device: &str) -> String {
        self.cache()
            .objects
            .get(device)
            .and_then(|interfaces| interfaces.get(DEVICE))
            .and_then(|props| value::<OwnedObjectPath>(props, "Adapter"))
            .map(|adapter| adapter.to_string())
            .unwrap_or_else(|| parent(device))
    }

    async fn call(&self, path: &str, interface: &str, method: &str) -> zbus::Result<()> {
        self.connection
            .call_method(Some(BLUEZ), path, Some(interface), method, &())
            .await
            .map(|_| ())
    }

    async fn set(&self, path: &str, interface: &str, name: &str, value: Value<'_>) -> zbus::Result<()> {
        self.connection
            .call_method(Some(BLUEZ), path, Some(PROPERTIES), "Set", &(interface, name, value))
            .await
            .map(|_| ())
    }

    async fn done(&self, device: &str, op: Op, result: zbus::Result<()>) {
        if let Err(error) = &result {
            eprintln!("settings-bluetooth: {op:?} {device}: {error}");
        }
        self.send(Event::Done {
            device: device.to_string(),
            op,
            failure: result.err().map(|error| failure_of(&error)),
        })
        .await;
    }

    pub async fn execute(&self, command: Command) {
        match command {
            Command::SetPowered(on) => self.power(on).await,
            Command::SetVisible(on) => {
                let Some(adapter) = self.adapter_path() else {
                    return;
                };
                if on {
                    let _ = self.set(&adapter, ADAPTER, "DiscoverableTimeout", Value::from(DISCOVERABLE_SECONDS)).await;
                }
                if let Err(error) = self.set(&adapter, ADAPTER, "Discoverable", Value::from(on)).await {
                    eprintln!("settings-bluetooth: cannot change visibility: {error}");
                }
            }
            Command::Discover(on) => {
                let Some(adapter) = self.adapter_path() else {
                    return;
                };
                let method = if on { "StartDiscovery" } else { "StopDiscovery" };
                if let Err(error) = self.call(&adapter, ADAPTER, method).await {
                    eprintln!("settings-bluetooth: {method}: {error}");
                }
            }
            Command::Connect(device) => {
                let result = settled(self.call(&device, DEVICE, "Connect").await, ".AlreadyConnected");
                self.done(&device, Op::Connect, result).await;
            }
            Command::Disconnect(device) => {
                let result = settled(self.call(&device, DEVICE, "Disconnect").await, ".NotConnected");
                self.done(&device, Op::Disconnect, result).await;
            }
            Command::Pair(device) => self.pair(device).await,
            Command::CancelPairing(device) => {
                let _ = self.call(&device, DEVICE, "CancelPairing").await;
            }
            Command::SetTrusted(device, on) => {
                let result = self.set(&device, DEVICE, "Trusted", Value::from(on)).await;
                self.done(&device, Op::Trust, result).await;
            }
            Command::Forget(device) => {
                let adapter = self.adapter_of(&device);
                let result = match ObjectPath::try_from(device.as_str()) {
                    Ok(path) => self
                        .connection
                        .call_method(Some(BLUEZ), adapter.as_str(), Some(ADAPTER), "RemoveDevice", &(path,))
                        .await
                        .map(|_| ()),
                    Err(error) => Err(error.into()),
                };
                self.done(&device, Op::Forget, result).await;
            }
        }
    }

    async fn power(&self, on: bool) {
        let Some(adapter) = self.adapter_path() else {
            return;
        };
        if on && self.cache().radio == Radio::SoftBlocked {
            if let Err(error) = rfkill::unblock() {
                eprintln!("settings-bluetooth: cannot unblock the radio: {error}");
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
        let result = self.set(&adapter, ADAPTER, "Powered", Value::from(on)).await;
        self.done(&adapter, Op::Power, result).await;
        tokio::time::sleep(Duration::from_millis(200)).await;
        self.read_radio();
        self.emit().await;
    }

    async fn pair(&self, device: String) {
        self.send(Event::Pairing(Step::Start(device.clone()))).await;
        match self.call(&device, DEVICE, "Pair").await {
            Err(error) if !already_exists(&error) => {
                eprintln!("settings-bluetooth: pairing {device}: {error}");
                self.send(Event::Pairing(Step::Failed(device, failure_of(&error)))).await;
                return;
            }
            _ => {}
        }
        self.send(Event::Pairing(Step::Paired(device.clone()))).await;
        if let Err(error) = self.set(&device, DEVICE, "Trusted", Value::from(true)).await {
            eprintln!("settings-bluetooth: cannot trust {device}: {error}");
        }
        let result = settled(self.call(&device, DEVICE, "Connect").await, ".AlreadyConnected");
        self.done(&device, Op::Connect, result).await;
        self.send(Event::Pairing(Step::Finished(device))).await;
    }
}
