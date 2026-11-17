//! The reveal and conceal motion every widget shares.
//!
//! A surface that appears carries the `fx-reveal` class (or `fx-reveal-side`
//! for one that slides in from the right edge) and starts transparent and
//! slightly offset. [`reveal`] adds `fx-shown` once the surface has been drawn
//! in that state, so the CSS transition in [`crate::theme::BASE_CSS`] runs:
//! a 160 ms ease-out slide and fade. [`conceal`] removes the class and waits
//! for the transition before it hands over, so a window is hidden only once its
//! content has faded.
//!
//! GTK skips CSS transitions when `gtk-enable-animations` is off, and
//! [`conceal`] then hands over at once, so the motion disappears entirely.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use gtk4::glib;
use gtk4::prelude::*;

/// The length of every shell transition, matching the CSS.
pub const DURATION_MS: u64 = 160;

/// The class that holds a revealed surface in its final place.
const SHOWN: &str = "fx-shown";

/// The class a surface carries between [`reveal`] and the frame that shows it.
const PENDING: &str = "fx-pending";

/// The frame on which the class is added: the first frame draws the hidden
/// state, so the transition has a starting point.
const START_FRAME: u32 = 2;

/// Whether the user lets GTK animate.
pub fn enabled() -> bool {
    gtk4::Settings::default().is_none_or(|settings| settings.is_gtk_enable_animations())
}

/// Bring `widget` to its shown state with the shared transition.
pub fn reveal(widget: &impl IsA<gtk4::Widget>) {
    let widget = widget.as_ref().clone();
    if widget.has_css_class(SHOWN) || widget.has_css_class(PENDING) {
        return;
    }
    if !enabled() {
        widget.add_css_class(SHOWN);
        return;
    }
    widget.add_css_class(PENDING);
    let frames = Rc::new(Cell::new(0u32));
    widget.add_tick_callback(move |widget, _| {
        frames.set(frames.get() + 1);
        if !widget.has_css_class(PENDING) {
            return glib::ControlFlow::Break;
        }
        if frames.get() < START_FRAME {
            return glib::ControlFlow::Continue;
        }
        widget.remove_css_class(PENDING);
        widget.add_css_class(SHOWN);
        glib::ControlFlow::Break
    });
}

/// Take `widget` back to its hidden state, then run `done`: at once when
/// animations are off, after the transition otherwise. A widget revealed again
/// before the transition ends keeps its place and `done` never runs.
pub fn conceal(widget: &impl IsA<gtk4::Widget>, done: impl FnOnce() + 'static) {
    let widget = widget.as_ref().clone();
    widget.remove_css_class(SHOWN);
    widget.remove_css_class(PENDING);
    if !enabled() || !widget.is_mapped() {
        done();
        return;
    }
    glib::timeout_add_local_once(Duration::from_millis(DURATION_MS), move || {
        if !widget.has_css_class(SHOWN) && !widget.has_css_class(PENDING) {
            done();
        }
    });
}
