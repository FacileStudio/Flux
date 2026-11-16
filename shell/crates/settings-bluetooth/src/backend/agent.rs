//! The `org.bluez.Agent1` object BlueZ calls while pairing. Each request that
//! needs the person goes to the page as an [`Event::Request`] and waits for
//! the [`Answer`]; BlueZ's `Cancel` drops the wait.

use std::sync::{Arc, Mutex};

use async_channel::{Receiver, Sender};
use zbus::zvariant::OwnedObjectPath;

use super::{Answer, Event, Request};
use crate::model::{self, Prompt};

/// Where the agent is served on our connection.
pub const AGENT_PATH: &str = "/studio/facile/flux/bluetooth/agent";

#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.bluez.Error")]
pub enum AgentError {
    #[zbus(error)]
    ZBus(zbus::Error),
    Rejected(String),
    Canceled(String),
}

fn rejected() -> AgentError {
    AgentError::Rejected("Rejected by the user".into())
}

pub struct Agent {
    events: Sender<Event>,
    pending: Arc<Mutex<Option<Receiver<Answer>>>>,
}

impl Agent {
    pub fn new(events: Sender<Event>) -> Self {
        Agent {
            events,
            pending: Arc::new(Mutex::new(None)),
        }
    }

    fn set_pending(&self, receiver: Option<Receiver<Answer>>) {
        let mut pending = self.pending.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(previous) = pending.take() {
            previous.close();
        }
        *pending = receiver;
    }

    async fn ask(&self, device: OwnedObjectPath, prompt: Prompt) -> Result<Answer, AgentError> {
        let (reply, answer) = async_channel::bounded(1);
        self.set_pending(Some(answer.clone()));
        let request = Request {
            device: device.to_string(),
            prompt,
            reply: Some(reply),
        };
        if self.events.send(Event::Request(request)).await.is_err() {
            return Err(rejected());
        }
        let result = answer.recv().await;
        match result {
            Ok(Answer::Reject) => Err(rejected()),
            Ok(answer) => Ok(answer),
            Err(_) => Err(AgentError::Canceled("Canceled".into())),
        }
    }

    async fn agree(&self, device: OwnedObjectPath, prompt: Prompt) -> Result<(), AgentError> {
        match self.ask(device, prompt).await? {
            Answer::Accept => Ok(()),
            _ => Err(rejected()),
        }
    }

    async fn show(&self, device: OwnedObjectPath, prompt: Prompt) {
        let request = Request {
            device: device.to_string(),
            prompt,
            reply: None,
        };
        let _ = self.events.send(Event::Request(request)).await;
    }
}

#[zbus::interface(name = "org.bluez.Agent1")]
impl Agent {
    async fn release(&self) {
        self.set_pending(None);
    }

    async fn request_pin_code(&self, device: OwnedObjectPath) -> Result<String, AgentError> {
        match self.ask(device, Prompt::RequestPin).await? {
            Answer::Pin(pin) if model::valid_pin(&pin) => Ok(pin),
            _ => Err(rejected()),
        }
    }

    async fn display_pin_code(&self, device: OwnedObjectPath, pincode: String) {
        self.show(device, Prompt::DisplayPin(pincode)).await;
    }

    async fn request_passkey(&self, device: OwnedObjectPath) -> Result<u32, AgentError> {
        match self.ask(device, Prompt::RequestPasskey).await? {
            Answer::Passkey(passkey) if passkey <= 999_999 => Ok(passkey),
            _ => Err(rejected()),
        }
    }

    async fn display_passkey(&self, device: OwnedObjectPath, passkey: u32, entered: u16) {
        self.show(device, Prompt::DisplayPasskey { passkey, entered }).await;
    }

    async fn request_confirmation(&self, device: OwnedObjectPath, passkey: u32) -> Result<(), AgentError> {
        self.agree(device, Prompt::Confirm(passkey)).await
    }

    async fn request_authorization(&self, device: OwnedObjectPath) -> Result<(), AgentError> {
        self.agree(device, Prompt::Authorize).await
    }

    async fn authorize_service(&self, device: OwnedObjectPath, uuid: String) -> Result<(), AgentError> {
        self.agree(device, Prompt::AuthorizeService(model::service(&uuid))).await
    }

    async fn cancel(&self) {
        self.set_pending(None);
        let _ = self.events.send(Event::RequestCancelled).await;
    }
}
