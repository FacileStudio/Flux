//! Motion that follows the GTK "enable animations" setting: with it off, everything is instant.

use gtk4::prelude::*;
use gtk4::{Revealer, RevealerTransitionType, Widget};

/// Reveal and slide duration, in milliseconds.
pub const DURATION: u32 = shell_common::ui::MOTION_MS;

/// Whether the session wants animations.
pub fn enabled() -> bool {
    shell_common::ui::animations()
}

/// A revealer that slides its child down over [`DURATION`] and starts hidden.
pub fn revealer(child: &impl IsA<Widget>) -> Revealer {
    Revealer::builder()
        .transition_type(RevealerTransitionType::SlideDown)
        .transition_duration(DURATION)
        .reveal_child(false)
        .child(child)
        .build()
}

/// Show or hide, instantly when animations are off.
pub fn reveal(revealer: &Revealer, show: bool) {
    if revealer.reveals_child() == show {
        return;
    }
    revealer.set_transition_duration(if enabled() { DURATION } else { 0 });
    revealer.set_reveal_child(show);
}

/// Ease-out cubic, for a progress between 0 and 1.
pub fn ease_out(progress: f64) -> f64 {
    let rest = 1.0 - progress.clamp(0.0, 1.0);
    1.0 - rest * rest * rest
}
