//! The light or dark colour scheme.
//!
//! `flux-scheme` writes `org.gnome.desktop.interface color-scheme` to dconf,
//! so that key is the source of truth: [`follow`] reads it through
//! `gio::Settings`, a local dconf read that never waits on another process, and
//! restyles on its `changed` signal. `prefer-dark` is dark; anything else is
//! light.
//!
//! Without the GNOME schema the XDG desktop portal answers instead
//! (`org.freedesktop.appearance color-scheme`: `1` dark, `2` light, `0` no
//! preference). That read is asynchronous, so a portal that is still starting
//! never holds up the first paint: the widget draws with `/etc/flux/palette.css`
//! and restyles when the answer lands.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::gio;
use gtk4::glib::{self, Variant};
use gtk4::prelude::*;

const SCHEMA: &str = "org.gnome.desktop.interface";
const SCHEMA_KEY: &str = "color-scheme";

const PORTAL: &str = "org.freedesktop.portal.Desktop";
const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
const SETTINGS: &str = "org.freedesktop.portal.Settings";
const NAMESPACE: &str = "org.freedesktop.appearance";
const KEY: &str = "color-scheme";

/// The time a portal read may take before the widget gives up on it.
const READ_TIMEOUT_MS: i32 = 5000;

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

    /// The scheme a GSettings `color-scheme` value stands for.
    pub fn from_gsettings(value: &str) -> Scheme {
        if value == "prefer-dark" {
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
    static SOURCE: RefCell<Option<glib::Object>> = const { RefCell::new(None) };
}

/// The scheme last read, or `None` before the first answer.
pub fn current() -> Option<Scheme> {
    CURRENT.with(Cell::get)
}

/// Record `scheme` and run `on_change` when it differs from what was known.
fn update(scheme: Option<Scheme>, on_change: &dyn Fn()) {
    if scheme.is_some() && scheme != current() {
        CURRENT.with(|slot| slot.set(scheme));
        on_change();
    }
}

/// Read the scheme and run `on_change` whenever it moves. Call once per
/// process, before the stylesheet is first built: with GSettings the first
/// value is known before that build, so it is drawn in the right colours.
pub fn follow(on_change: impl Fn() + 'static) {
    let on_change: Rc<dyn Fn()> = Rc::new(on_change);
    if let Some(settings) = interface_settings() {
        CURRENT.with(|slot| slot.set(Some(Scheme::from_gsettings(&settings.string(SCHEMA_KEY)))));
        settings.connect_changed(Some(SCHEMA_KEY), move |settings, _| {
            update(Some(Scheme::from_gsettings(&settings.string(SCHEMA_KEY))), &*on_change);
        });
        SOURCE.with(|slot| *slot.borrow_mut() = Some(settings.upcast()));
        return;
    }
    follow_portal(on_change);
}

/// The GNOME interface settings, when their schema is installed. Without the
/// check `gio::Settings::new` would abort the process.
fn interface_settings() -> Option<gio::Settings> {
    let schema = gio::SettingsSchemaSource::default()?.lookup(SCHEMA, true)?;
    schema
        .has_key(SCHEMA_KEY)
        .then(|| gio::Settings::new(SCHEMA))
}

/// Ask the portal, without waiting, and subscribe to its changes.
fn follow_portal(on_change: Rc<dyn Fn()>) {
    gio::DBusProxy::for_bus(
        gio::BusType::Session,
        gio::DBusProxyFlags::DO_NOT_LOAD_PROPERTIES,
        None,
        PORTAL,
        PORTAL_PATH,
        SETTINGS,
        gio::Cancellable::NONE,
        move |proxy| {
            let Ok(proxy) = proxy else {
                return;
            };
            let signal_change = on_change.clone();
            proxy.connect_local("g-signal", false, move |values| {
                update(changed_scheme(values), &*signal_change);
                None
            });
            read_portal(&proxy, "ReadOne", on_change);
            SOURCE.with(|slot| *slot.borrow_mut() = Some(proxy.upcast()));
        },
    );
}

/// One portal read, falling back from `ReadOne` to the older `Read`.
fn read_portal(proxy: &gio::DBusProxy, method: &'static str, on_change: Rc<dyn Fn()>) {
    let retry = proxy.clone();
    proxy.call(
        method,
        Some(&(NAMESPACE, KEY).to_variant()),
        gio::DBusCallFlags::NONE,
        READ_TIMEOUT_MS,
        gio::Cancellable::NONE,
        move |reply| match reply {
            Ok(reply) => update(portal_value(&reply.child_value(0)).map(Scheme::from_portal), &*on_change),
            Err(_) if method == "ReadOne" => read_portal(&retry, "Read", on_change),
            Err(_) => {}
        },
    );
}

/// Strip the `v` wrappers the portal puts around a value and read the `u32`.
fn portal_value(value: &Variant) -> Option<u32> {
    let mut value = value.clone();
    while let Some(inner) = value.as_variant() {
        value = inner;
    }
    value.get::<u32>()
}

/// The scheme a proxy `g-signal` emission carries, when it is the portal's
/// `SettingChanged` for `color-scheme`.
fn changed_scheme(values: &[glib::Value]) -> Option<Scheme> {
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
    portal_value(&parameters.child_value(2)).map(Scheme::from_portal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_sources_name_the_same_schemes() {
        assert_eq!(Scheme::from_gsettings("prefer-dark"), Scheme::Dark);
        assert_eq!(Scheme::from_gsettings("prefer-light"), Scheme::Light);
        assert_eq!(Scheme::from_gsettings("default"), Scheme::Light);
        assert_eq!(Scheme::from_portal(1), Scheme::Dark);
        assert_eq!(Scheme::from_portal(2), Scheme::Light);
        assert_eq!(Scheme::from_portal(0), Scheme::Light);
    }

    #[test]
    fn portal_values_lose_their_wrappers() {
        let wrapped = Variant::from_variant(&Variant::from_variant(&1u32.to_variant()));
        assert_eq!(portal_value(&wrapped), Some(1));
        assert_eq!(portal_value(&"x".to_variant()), None);
    }
}
