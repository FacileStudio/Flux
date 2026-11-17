//! Typed reads out of the `a{sv}` maps NetworkManager answers with.
//!
//! A value may arrive wrapped in one or more variants, depending on how the
//! map was decoded, so every read looks through them first. A missing key or a
//! value of the wrong type reads as `None`, never as a panic.

use std::collections::HashMap;

use zbus::zvariant::{OwnedValue, Value};

/// The properties of one D-Bus object, from `GetAll`.
pub type Props = HashMap<String, OwnedValue>;

/// A connection profile, from `GetSettings`: setting name to its properties.
pub type Settings = HashMap<String, HashMap<String, OwnedValue>>;

/// `value` with every variant wrapper removed.
pub fn plain<'a>(value: &'a Value<'a>) -> &'a Value<'a> {
    match value {
        Value::Value(inner) => plain(inner),
        other => other,
    }
}

fn get<'a>(props: &'a Props, key: &str) -> Option<&'a Value<'a>> {
    props.get(key).map(|value| plain(value))
}

/// A `u` property.
pub fn u32_of(props: &Props, key: &str) -> Option<u32> {
    match get(props, key)? {
        Value::U32(number) => Some(*number),
        _ => None,
    }
}

/// A `y` property.
pub fn u8_of(props: &Props, key: &str) -> Option<u8> {
    match get(props, key)? {
        Value::U8(number) => Some(*number),
        _ => None,
    }
}

/// An `x` property.
pub fn i64_of(props: &Props, key: &str) -> Option<i64> {
    match get(props, key)? {
        Value::I64(number) => Some(*number),
        _ => None,
    }
}

/// A `b` property.
pub fn bool_of(props: &Props, key: &str) -> Option<bool> {
    match get(props, key)? {
        Value::Bool(flag) => Some(*flag),
        _ => None,
    }
}

/// An `s` property.
pub fn str_of(props: &Props, key: &str) -> Option<String> {
    match get(props, key)? {
        Value::Str(text) => Some(text.as_str().to_string()),
        _ => None,
    }
}

/// An `o` property, with NetworkManager's `/` meaning none.
pub fn path_of(props: &Props, key: &str) -> Option<String> {
    match get(props, key)? {
        Value::ObjectPath(path) if path.as_str() != "/" => Some(path.as_str().to_string()),
        _ => None,
    }
}

/// An `ao` property.
pub fn paths_of(props: &Props, key: &str) -> Vec<String> {
    match get(props, key) {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| match plain(item) {
                Value::ObjectPath(path) => Some(path.as_str().to_string()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// An `ay` property.
pub fn bytes_of(props: &Props, key: &str) -> Option<Vec<u8>> {
    match get(props, key)? {
        Value::Array(items) => items
            .iter()
            .map(|item| match plain(item) {
                Value::U8(byte) => Some(*byte),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}

/// An `au` property.
pub fn u32s_of(props: &Props, key: &str) -> Vec<u32> {
    match get(props, key) {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| match plain(item) {
                Value::U32(number) => Some(*number),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// An `aa{sv}` property: one map per element.
pub fn maps_of(props: &Props, key: &str) -> Vec<Props> {
    match get(props, key) {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| {
                let owned = plain(item).try_clone().ok()?;
                Props::try_from(owned).ok()
            })
            .collect(),
        _ => Vec::new(),
    }
}
