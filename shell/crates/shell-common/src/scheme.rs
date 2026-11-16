//! The light or dark colour scheme, read from the XDG desktop portal.
//!
//! The portal's `org.freedesktop.appearance` `color-scheme` setting is `1` for
//! dark, `2` for light and `0` for no preference, which Flux treats as light.
//! [`follow`] reads it once and subscribes to `SettingChanged`, so a
//! `flux-scheme toggle` restyles every running widget. Without a portal the
//! scheme stays unknown and the palette falls back to `/etc/flux/palette.css`.

use std::cell::{Cell, RefCell};

use gtk4::gio;
use gtk4::glib::{self, Variant};
use gtk4::prelude::*;

const PORTAL: &str = "org.freedesktop.portal.Desktop";
const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
const SETTINGS: &str = "org.freedesktop.portal.Settings";
const NAMESPACE: &str = "org.freedesktop.appearance";
const KEY: &str = "color-scheme";

/// The time a portal read may take before the widget starts without it.
const READ_TIMEOUT_MS: i32 = 1000;

/// The two schemes Flux ships a palette for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scheme {
    Light,
    Dark,
}

impl Scheme {
    /// The scheme a portal `color-scheme` value stands for.
    pub fn from_portal(value: u32) -> Scheme {
        if value == 1 {
            Scheme::Dark
        } else {
            Scheme::Light
        }
    }

    /// The name used in `/etc/flux/palette-<name>.css`.
    pub fn name(self) -> &'static str {
        match self {
            Scheme::Light => "light",
            Scheme::Dark => "dark",
        }
    }
}

thread_local! {
    static CURRENT: Cell<Option<Scheme>> = const { Cell::new(None) };
    static PROXY: RefCell<Option<gio::DBusProxy>> = const { RefCell::new(None) };
}

/// The scheme the portal last reported, or `None` without a portal.
pub fn current() -> Option<Scheme> {
    CURRENT.with(Cell::get)
}

/// Strip the `v` wrappers the portal puts around a value and read the `u32`.
fn portal_value(value: &Variant) -> Option<u32> {
    let mut value = value.clone();
    while let Some(inner) = value.as_variant() {
        value = inner;
    }
    value.get::<u32>()
}

/// Ask the portal for the setting, through `ReadOne` and then the older `Read`.
fn read(proxy: &gio::DBusProxy) -> Option<Scheme> {
    ["ReadOne", "Read"].iter().find_map(|method| {
        let reply = proxy
            .call_sync(
                method,
                Some(&(NAMESPACE, KEY).to_variant()),
                gio::DBusCallFlags::NONE,
                READ_TIMEOUT_MS,
                gio::Cancellable::NONE,
            )
            .ok()?;
        portal_value(&reply.child_value(0)).map(Scheme::from_portal)
    })
}

/// Read the scheme now and run `on_change` whenever the portal reports a new
/// one. Call once per process, before the stylesheet is first built.
pub fn follow(on_change: impl Fn() + 'static) {
    let Ok(proxy) = gio::DBusProxy::for_bus_sync(
        gio::BusType::Session,
        gio::DBusProxyFlags::DO_NOT_LOAD_PROPERTIES,
        None,
        PORTAL,
        PORTAL_PATH,
        SETTINGS,
        gio::Cancellable::NONE,
    ) else {
        return;
    };
    CURRENT.with(|slot| slot.set(read(&proxy)));
    proxy.connect_local("g-signal", false, move |values| {
        let signal = values.get(2)?.get::<String>().ok()?;
        let parameters = values.get(3)?.get::<Variant>().ok()?;
        if signal != "SettingChanged" {
            return None;
        }
        let namespace = parameters.child_value(0).get::<String>()?;
        let key = parameters.child_value(1).get::<String>()?;
        if namespace != NAMESPACE || key != KEY {
            return None;
        }
        let scheme = portal_value(&parameters.child_value(2)).map(Scheme::from_portal);
        if scheme.is_some() && scheme != current() {
            CURRENT.with(|slot| slot.set(scheme));
            on_change();
        }
        None::<glib::Value>
    });
    PROXY.with(|slot| *slot.borrow_mut() = Some(proxy));
}
