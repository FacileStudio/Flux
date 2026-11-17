//! Playing streams, grouped into one row per application.

use super::{Stream, OWN_APP};

/// The streams of one application, shown as one row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppGroup {
    pub key: String,
    pub name: String,
    pub icon: String,
    pub streams: Vec<u32>,
    pub volume: u32,
    pub muted: bool,
}

impl Stream {
    fn property(&self, key: &str) -> Option<&str> {
        self.properties
            .get(key)
            .map(String::as_str)
            .filter(|value| !value.trim().is_empty())
    }

    fn is_own(&self) -> bool {
        self.property("application.name") == Some(OWN_APP)
    }
}

/// Group playing streams by application, in the order apps first appear.
pub fn group_streams(streams: &[Stream]) -> Vec<AppGroup> {
    let mut groups: Vec<AppGroup> = Vec::new();
    for stream in streams.iter().filter(|stream| !stream.is_own()) {
        let binary = stream.property("application.process.binary");
        let app = stream.property("application.name");
        let key = binary
            .or(app)
            .map(str::to_ascii_lowercase)
            .unwrap_or_else(|| format!("stream-{}", stream.index));
        if let Some(group) = groups.iter_mut().find(|group| group.key == key) {
            group.streams.push(stream.index);
            group.volume = group.volume.max(stream.volume);
            group.muted &= stream.muted;
            continue;
        }
        let name = app
            .or(stream.property("media.name"))
            .or(binary)
            .map(str::to_string)
            .unwrap_or_else(|| key.clone());
        let icon = stream
            .property("application.icon_name")
            .map(str::to_string)
            .or_else(|| binary.map(str::to_ascii_lowercase))
            .unwrap_or_else(|| "application-x-executable-symbolic".to_string());
        groups.push(AppGroup {
            key,
            name: title_case(&name),
            icon,
            streams: vec![stream.index],
            volume: stream.volume,
            muted: stream.muted,
        });
    }
    groups
}

fn title_case(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_lowercase() => first.to_uppercase().chain(chars).collect(),
        _ => name.to_string(),
    }
}
