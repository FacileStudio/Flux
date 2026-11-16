//! The Sound page of Flux Settings: output and input devices, volumes, per-app volume, and the
//! statusbar quick menu, over the PulseAudio protocol (served by `pipewire-pulse`).

mod backend;
mod hub;
mod i18n;
pub mod model;
mod motion;
mod options;
mod ui;
mod view;

use std::sync::mpsc;
use std::time::Duration;

use backend::{Command, Target};
use model::Direction;

pub use hub::PREVIEW;

/// The page's key in the settings navigation and on the `flux-settings` command line.
pub const ID: &str = "audio";
/// The page's navigation icon.
pub const ICON: &str = "audio-volume-high-symbolic";

/// The page title in the session language.
pub fn title() -> String {
    i18n::text().title.to_string()
}

/// The whole Sound page.
pub fn page() -> gtk4::Widget {
    view::page()
}

/// The statusbar popover: output slider and mute, output device, microphone mute, and a link
/// to the full page.
pub fn quick_menu() -> gtk4::Widget {
    view::quick_menu()
}

/// Move the default output's volume by `delta_percent`, unmuting on the way up, within the
/// "Allow volume above 100%" cap. Returns the new percentage, or `None` when there is no sound
/// server or no output. Blocks for up to a few seconds: call it off the GTK thread.
pub fn set_volume_step(delta_percent: i32) -> Option<u32> {
    let boost = options::load().allow_boost;
    let (backend, snapshot) = backend::connect_blocking(Duration::from_secs(3))?;
    let device = snapshot.default_device(Direction::Output)?;
    let target = Target::Device(Direction::Output, device.index);
    let raw = model::step(device.volume, delta_percent, boost);
    if device.muted && delta_percent > 0 {
        backend.send(Command::SetMute(target, false));
    }
    let (done, heard) = mpsc::channel();
    backend.send(Command::SetVolume(target, raw, Some(done)));
    heard
        .recv_timeout(Duration::from_secs(2))
        .ok()
        .filter(|ok| *ok)
        .map(|_| model::percent(raw))
}
