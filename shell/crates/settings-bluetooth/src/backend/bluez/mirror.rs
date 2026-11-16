//! The mirror of BlueZ's object tree and the snapshot made from it.

use std::collections::HashMap;

use zbus::message::Body;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

use super::{ADAPTER, BATTERY, DEVICE};
use crate::model::{self, Adapter, Device, Radio, Snapshot};

pub type Props = HashMap<String, OwnedValue>;
pub type Interfaces = HashMap<String, Props>;
pub type ManagedObjects = HashMap<OwnedObjectPath, Interfaces>;

#[derive(Default)]
pub struct Cache {
    pub service: bool,
    pub owner: Option<String>,
    pub objects: HashMap<String, Interfaces>,
    pub radio: Radio,
}

pub fn value<T: TryFrom<OwnedValue>>(props: &Props, key: &str) -> Option<T> {
    T::try_from(props.get(key)?.try_clone().ok()?).ok()
}

fn flag(props: &Props, key: &str) -> bool {
    value::<bool>(props, key).unwrap_or(false)
}

fn adapter(path: &str, props: &Props) -> Adapter {
    Adapter {
        path: path.to_string(),
        name: value(props, "Alias").or_else(|| value(props, "Name")).unwrap_or_default(),
        address: value(props, "Address").unwrap_or_default(),
        powered: flag(props, "Powered"),
        discoverable: flag(props, "Discoverable"),
        discovering: flag(props, "Discovering"),
    }
}

pub fn parent(path: &str) -> String {
    path.rsplit_once('/').map(|(head, _)| head.to_string()).unwrap_or_default()
}

fn device(path: &str, interfaces: &Interfaces, props: &Props) -> Device {
    Device {
        path: path.to_string(),
        adapter: value::<OwnedObjectPath>(props, "Adapter")
            .map(|adapter| adapter.to_string())
            .unwrap_or_else(|| parent(path)),
        address: value(props, "Address").unwrap_or_default(),
        address_type: value(props, "AddressType").unwrap_or_default(),
        name: value(props, "Name"),
        alias: value(props, "Alias"),
        icon: value(props, "Icon"),
        class: value(props, "Class"),
        appearance: value(props, "Appearance"),
        paired: flag(props, "Paired"),
        trusted: flag(props, "Trusted"),
        connected: flag(props, "Connected"),
        rssi: value(props, "RSSI"),
        battery: interfaces.get(BATTERY).and_then(|battery| value(battery, "Percentage")),
    }
}

pub fn snapshot(cache: &Cache) -> Snapshot {
    let mut adapters = Vec::new();
    let mut devices = Vec::new();
    for (path, interfaces) in &cache.objects {
        if let Some(props) = interfaces.get(ADAPTER) {
            adapters.push(adapter(path, props));
        }
        if let Some(props) = interfaces.get(DEVICE) {
            devices.push(device(path, interfaces, props));
        }
    }
    devices.sort_by(|a, b| a.path.cmp(&b.path));
    Snapshot {
        service: cache.service,
        adapter: model::pick_adapter(adapters).filter(|_| cache.service),
        devices,
        radio: cache.radio,
    }
}

impl Cache {
    fn add(&mut self, body: &Body) -> bool {
        let Ok((object, interfaces)) = body.deserialize::<(OwnedObjectPath, Interfaces)>() else {
            return false;
        };
        self.objects.entry(object.to_string()).or_default().extend(interfaces);
        true
    }

    fn remove(&mut self, body: &Body) -> bool {
        let Ok((object, names)) = body.deserialize::<(OwnedObjectPath, Vec<String>)>() else {
            return false;
        };
        let object = object.to_string();
        let Some(interfaces) = self.objects.get_mut(&object) else {
            return true;
        };
        names.iter().for_each(|name| {
            interfaces.remove(name);
        });
        if interfaces.is_empty() {
            self.objects.remove(&object);
        }
        true
    }

    fn change(&mut self, path: &str, body: &Body) -> bool {
        let Ok((interface, changed, invalidated)) = body.deserialize::<(String, Props, Vec<String>)>() else {
            return false;
        };
        let Some(props) = self.objects.get_mut(path).and_then(|interfaces| interfaces.get_mut(&interface)) else {
            return false;
        };
        props.extend(changed);
        invalidated.iter().for_each(|name| {
            props.remove(name);
        });
        true
    }

    /// Fold one ObjectManager or Properties signal in; true when it changed something.
    pub fn apply(&mut self, member: &str, path: &str, body: &Body) -> bool {
        match member {
            "InterfacesAdded" => self.add(body),
            "InterfacesRemoved" => self.remove(body),
            "PropertiesChanged" => self.change(path, body),
            _ => false,
        }
    }
}
