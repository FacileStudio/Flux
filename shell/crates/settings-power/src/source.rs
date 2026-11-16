//! Where the page's battery and power-mode state comes from: UPower and
//! power-profiles-daemon on the system bus, followed through their
//! `PropertiesChanged` signals, or fixed sample data for a preview.
//!
//! Proxies are created asynchronously, so nothing here blocks the main
//! thread. A service that is not running leaves its part of the snapshot
//! empty, and the page shows its empty state instead.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::gio::{self, prelude::*};
use gtk4::glib::{self, Variant};

use crate::upower::{self, Battery, State};

const UPOWER: &str = "org.freedesktop.UPower";
const UPOWER_PATH: &str = "/org/freedesktop/UPower";
const DISPLAY_PATH: &str = "/org/freedesktop/UPower/devices/DisplayDevice";
const DEVICE: &str = "org.freedesktop.UPower.Device";
const PROFILES: &str = "net.hadess.PowerProfiles";
const PROFILES_PATH: &str = "/net/hadess/PowerProfiles";

/// Everything the page draws from, at one moment.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub battery: Option<Battery>,
    pub lid: bool,
    pub profiles: Vec<&'static str>,
    pub active: Option<String>,
}

type Listener = Box<dyn Fn(&Source)>;

#[derive(Default)]
struct Inner {
    sample: Option<RefCell<Snapshot>>,
    root: RefCell<Option<gio::DBusProxy>>,
    display: RefCell<Option<gio::DBusProxy>>,
    device: RefCell<Option<gio::DBusProxy>>,
    profiles: RefCell<Option<gio::DBusProxy>>,
    listeners: RefCell<Vec<Listener>>,
}

/// A shared handle on the state; clones see the same proxies.
#[derive(Clone, Default)]
pub struct Source {
    inner: Rc<Inner>,
}

fn proxy(path: &str, name: &str, interface: &str, done: impl FnOnce(gio::DBusProxy) + 'static) {
    gio::DBusProxy::for_bus(
        gio::BusType::System,
        gio::DBusProxyFlags::NONE,
        None,
        name,
        path,
        interface,
        None::<&gio::Cancellable>,
        move |result| {
            if let Ok(proxy) = result {
                done(proxy);
            }
        },
    );
}

impl Source {
    /// Follow the real services.
    pub fn live() -> Source {
        let source = Source::default();
        source.connect(UPOWER_PATH, UPOWER, UPOWER, |inner| &inner.root);
        source.connect(DISPLAY_PATH, UPOWER, DEVICE, |inner| &inner.display);
        source.connect(PROFILES_PATH, PROFILES, PROFILES, |inner| &inner.profiles);
        source.find_battery();
        source
    }

    /// Fixed sample data: a laptop on battery, or a desktop without one.
    pub fn sample(laptop: bool) -> Source {
        let battery = laptop.then_some(Battery {
            percentage: 76.0,
            state: State::Discharging,
            time_to_empty: 11_520,
            time_to_full: 0,
            health: Some(91.0),
            cycles: Some(214),
        });
        let snapshot = Snapshot {
            battery,
            lid: laptop,
            profiles: upower::PROFILES.to_vec(),
            active: Some("balanced".into()),
        };
        Source {
            inner: Rc::new(Inner { sample: Some(RefCell::new(snapshot)), ..Inner::default() }),
        }
    }

    fn connect(&self, path: &str, name: &str, interface: &str, slot: fn(&Inner) -> &RefCell<Option<gio::DBusProxy>>) {
        let weak = Rc::downgrade(&self.inner);
        proxy(path, name, interface, move |proxy| {
            let Some(inner) = weak.upgrade() else {
                return;
            };
            let changed = Rc::downgrade(&inner);
            proxy.connect_local("g-properties-changed", false, move |_| {
                if let Some(inner) = changed.upgrade() {
                    Source { inner }.notify();
                }
                None
            });
            slot(&inner).replace(Some(proxy));
            Source { inner }.notify();
        });
    }

    fn find_battery(&self) {
        let weak = Rc::downgrade(&self.inner);
        proxy(UPOWER_PATH, UPOWER, UPOWER, move |root| {
            root.call(
                "EnumerateDevices",
                None,
                gio::DBusCallFlags::NONE,
                2000,
                None::<&gio::Cancellable>,
                move |result| {
                    let Some(paths) = result.ok().and_then(|reply| reply.child_value(0).get::<Vec<glib::variant::ObjectPath>>()) else {
                        return;
                    };
                    for path in paths {
                        let weak = weak.clone();
                        proxy(path.as_str(), UPOWER, DEVICE, move |device| {
                            let Some(inner) = weak.upgrade() else {
                                return;
                            };
                            let is_battery = device.cached_property("Type").and_then(|kind| kind.get::<u32>()) == Some(2)
                                && device.cached_property("PowerSupply").and_then(|supply| supply.get::<bool>()) == Some(true);
                            if is_battery && inner.device.borrow().is_none() {
                                inner.device.replace(Some(device));
                                Source { inner }.notify();
                            }
                        });
                    }
                },
            );
        });
    }

    fn notify(&self) {
        for listener in self.inner.listeners.borrow().iter() {
            listener(self);
        }
    }

    /// Run `listener` whenever the state changes.
    pub fn on_change(&self, listener: impl Fn(&Source) + 'static) {
        self.inner.listeners.borrow_mut().push(Box::new(listener));
    }

    /// Whether this is sample data rather than the real services.
    pub fn is_sample(&self) -> bool {
        self.inner.sample.is_some()
    }

    /// The state now.
    pub fn snapshot(&self) -> Snapshot {
        if let Some(sample) = &self.inner.sample {
            return sample.borrow().clone();
        }
        let cached = |slot: &RefCell<Option<gio::DBusProxy>>, name: &str| -> Option<Variant> {
            slot.borrow().as_ref()?.cached_property(name)
        };
        let mut battery = upower::battery(|name| cached(&self.inner.display, name));
        if let Some(battery) = battery.as_mut() {
            (battery.health, battery.cycles) = upower::health(|name| cached(&self.inner.device, name));
        }
        Snapshot {
            battery,
            lid: cached(&self.inner.root, "LidIsPresent").and_then(|value| value.get::<bool>()).unwrap_or(false),
            profiles: cached(&self.inner.profiles, "Profiles").map(|value| upower::profiles(&value)).unwrap_or_default(),
            active: cached(&self.inner.profiles, "ActiveProfile").and_then(|value| value.get::<String>()),
        }
    }

    /// Ask power-profiles-daemon for `profile`. The page redraws from the
    /// daemon's own change signal, so a refused request leaves it as it was.
    pub fn set_profile(&self, profile: &str) {
        if let Some(sample) = &self.inner.sample {
            sample.borrow_mut().active = Some(profile.to_string());
            self.notify();
            return;
        }
        let Some(proxy) = self.inner.profiles.borrow().clone() else {
            return;
        };
        let arguments = (PROFILES, "ActiveProfile", Variant::from_variant(&profile.into())).into();
        let source = self.clone();
        proxy.call(
            "org.freedesktop.DBus.Properties.Set",
            Some(&arguments),
            gio::DBusCallFlags::NONE,
            2000,
            None::<&gio::Cancellable>,
            move |result| {
                if let Err(error) = result {
                    eprintln!("settings-power: power mode not changed: {error}");
                }
                source.notify();
            },
        );
    }
}
