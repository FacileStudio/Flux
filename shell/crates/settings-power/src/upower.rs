//! UPower and power-profiles-daemon properties, parsed into plain values.
//!
//! The parsers take a property lookup instead of a proxy, so the tests feed
//! them fixtures and the page feeds them `DBusProxy::cached_property`.

use gtk4::glib::{Variant, VariantDict};

/// UPower's `Type` for a battery.
const TYPE_BATTERY: u32 = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum State {
    Charging,
    #[default]
    Discharging,
    Full,
    NotCharging,
}

impl State {
    /// The state for UPower's `State` number.
    pub fn from_upower(value: u32) -> State {
        match value {
            1 | 5 => State::Charging,
            4 => State::Full,
            6 => State::NotCharging,
            _ => State::Discharging,
        }
    }
}

/// What the page shows about the battery.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Battery {
    pub percentage: f64,
    pub state: State,
    pub time_to_empty: i64,
    pub time_to_full: i64,
    pub health: Option<f64>,
    pub cycles: Option<u32>,
}

/// The battery UPower's display device describes, or `None` when the machine
/// has none (a desktop: `IsPresent` false or `Type` not a battery).
pub fn battery(get: impl Fn(&str) -> Option<Variant>) -> Option<Battery> {
    let present = get("IsPresent")?.get::<bool>()?;
    let kind = get("Type")?.get::<u32>()?;
    if !present || kind != TYPE_BATTERY {
        return None;
    }
    Some(Battery {
        percentage: get("Percentage")?.get::<f64>()?,
        state: State::from_upower(get("State").and_then(|value| value.get::<u32>()).unwrap_or(0)),
        time_to_empty: get("TimeToEmpty").and_then(|value| value.get::<i64>()).unwrap_or(0),
        time_to_full: get("TimeToFull").and_then(|value| value.get::<i64>()).unwrap_or(0),
        health: None,
        cycles: None,
    })
}

/// The health (`Capacity`, in percent) and charge cycles of one real battery
/// device; each is `None` when the hardware does not report it.
pub fn health(get: impl Fn(&str) -> Option<Variant>) -> (Option<f64>, Option<u32>) {
    let capacity = get("Capacity")
        .and_then(|value| value.get::<f64>())
        .filter(|capacity| *capacity > 0.0);
    let cycles = get("ChargeCycles")
        .and_then(|value| value.get::<i32>())
        .and_then(|cycles| u32::try_from(cycles).ok())
        .filter(|cycles| *cycles > 0);
    (capacity, cycles)
}

/// The three profiles power-profiles-daemon knows, in the order the page
/// shows them.
pub const PROFILES: [&str; 3] = ["power-saver", "balanced", "performance"];

/// The profiles this machine offers, from the daemon's `Profiles` property
/// (`aa{sv}`, each with a `Profile` string), in [`PROFILES`] order.
pub fn profiles(value: &Variant) -> Vec<&'static str> {
    let offered: Vec<String> = (0..value.n_children())
        .filter_map(|index| {
            let entry = value.child_value(index);
            VariantDict::new(Some(&entry)).lookup_value("Profile", None)?.get::<String>()
        })
        .collect();
    PROFILES
        .into_iter()
        .filter(|profile| offered.iter().any(|offered| offered == profile))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use gtk4::glib::prelude::*;

    use super::*;

    fn fixture(pairs: Vec<(&'static str, Variant)>) -> impl Fn(&str) -> Option<Variant> {
        let map: HashMap<&str, Variant> = pairs.into_iter().collect();
        move |name| map.get(name).cloned()
    }

    #[test]
    fn a_laptop_display_device_parses() {
        let get = fixture(vec![
            ("IsPresent", true.to_variant()),
            ("Type", 2u32.to_variant()),
            ("Percentage", 76.0f64.to_variant()),
            ("State", 2u32.to_variant()),
            ("TimeToEmpty", 11_520i64.to_variant()),
        ]);
        let battery = battery(get).expect("battery");
        assert_eq!(battery.percentage, 76.0);
        assert_eq!(battery.state, State::Discharging);
        assert_eq!(battery.time_to_empty, 11_520);
        assert_eq!(battery.time_to_full, 0);
    }

    #[test]
    fn a_desktop_has_no_battery() {
        let absent = fixture(vec![("IsPresent", false.to_variant()), ("Type", 2u32.to_variant())]);
        assert_eq!(battery(absent), None);
        let ups = fixture(vec![("IsPresent", true.to_variant()), ("Type", 3u32.to_variant())]);
        assert_eq!(battery(ups), None);
        assert_eq!(battery(fixture(vec![])), None);
        let wrong_type = fixture(vec![("IsPresent", "yes".to_variant()), ("Type", 2u32.to_variant())]);
        assert_eq!(battery(wrong_type), None);
    }

    #[test]
    fn health_is_optional() {
        let get = fixture(vec![("Capacity", 91.5f64.to_variant()), ("ChargeCycles", 214i32.to_variant())]);
        assert_eq!(health(get), (Some(91.5), Some(214)));
        let unknown = fixture(vec![("Capacity", 0.0f64.to_variant()), ("ChargeCycles", (-1i32).to_variant())]);
        assert_eq!(health(unknown), (None, None));
    }

    #[test]
    fn profiles_follow_the_hardware() {
        let entry = |name: &str| {
            let dict = VariantDict::new(None);
            dict.insert("Profile", name);
            dict.insert("Driver", "placeholder");
            dict.end()
        };
        let value = Variant::array_from_iter_with_type(
            gtk4::glib::VariantTy::VARDICT,
            [entry("balanced"), entry("power-saver")],
        );
        assert_eq!(profiles(&value), vec!["power-saver", "balanced"]);
    }

    #[test]
    fn upower_states_map() {
        assert_eq!(State::from_upower(1), State::Charging);
        assert_eq!(State::from_upower(4), State::Full);
        assert_eq!(State::from_upower(6), State::NotCharging);
        assert_eq!(State::from_upower(0), State::Discharging);
    }
}
