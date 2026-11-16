//! One popup card per bar, shared by that bar's control pills.
//!
//! The card is an overlay layer surface on the bar's monitor, paired with a
//! transparent full-screen backdrop that dismisses it on a click outside. It
//! slides and fades in under the pill that opened it.

use std::cell::Cell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, Application, ApplicationWindow, Box, Label, Orientation};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use shell_common::layer::LayerWindow;
use shell_common::motion;

/// The width of a fill bar's track, in pixels.
const TRACK: f64 = 150.0;

#[derive(Clone)]
pub struct Popup {
    window: ApplicationWindow,
    backdrop: ApplicationWindow,
    root: Box,
    active_anchor: Rc<Cell<Option<usize>>>,
}

impl Popup {
    pub fn new(application: &Application, monitor: Option<&gdk::Monitor>) -> Self {
        let backdrop_layer = LayerWindow::new(application).layer(Layer::Top);
        backdrop_layer.anchors(true, true, true, true);
        backdrop_layer.exclusive_zone(false);
        backdrop_layer.keyboard_mode(KeyboardMode::None);
        backdrop_layer.monitor(monitor);
        let backdrop = backdrop_layer.window;
        backdrop.set_css_classes(&["backdrop"]);

        let layer = LayerWindow::new(application).layer(Layer::Overlay);
        layer.anchors(true, false, true, false);
        layer.exclusive_zone(false);
        layer.keyboard_mode(KeyboardMode::OnDemand);
        layer.monitor(monitor);
        layer.window.set_margin(Edge::Top, shell_common::theme::WINDOW_TOP_GAP);
        layer.window.set_margin(Edge::Right, shell_common::theme::GAP);

        let window = layer.window;
        let root = Box::new(Orientation::Vertical, 0);
        root.add_css_class("popup");
        root.add_css_class("fx-reveal");
        window.set_child(Some(&root));

        let popup = Popup {
            window,
            backdrop,
            root,
            active_anchor: Rc::new(Cell::new(None)),
        };
        popup.dismiss_on_outside_click();
        popup.dismiss_on_escape();
        popup.window.set_visible(false);
        popup.backdrop.set_visible(false);
        popup
    }

    fn dismiss_on_outside_click(&self) {
        let click = gtk4::GestureClick::new();
        let popup = self.clone();
        click.connect_pressed(move |_, _, _, _| popup.hide());
        self.backdrop.add_controller(click);
    }

    fn dismiss_on_escape(&self) {
        let key = gtk4::EventControllerKey::new();
        let popup = self.clone();
        key.connect_key_pressed(move |_, keyval, _, _| {
            if keyval == gdk::Key::Escape {
                popup.hide();
                return gtk4::glib::Propagation::Stop;
            }
            gtk4::glib::Propagation::Proceed
        });
        self.window.add_controller(key);
    }

    pub fn position_under(&self, anchor: &impl IsA<gtk4::Widget>) {
        self.active_anchor.set(Some(anchor.as_ptr() as usize));
        let Some(toplevel) = anchor.root() else {
            return;
        };
        let Some(bounds) = anchor.compute_bounds(&toplevel) else {
            return;
        };
        let right_edge = f64::from(bounds.x() + bounds.width());
        let screen_width = f64::from(toplevel.width());
        let margin_right = (screen_width - right_edge).max(f64::from(shell_common::theme::GAP)) as i32;
        self.window.set_margin(Edge::Right, margin_right);
    }

    pub fn show<F: FnOnce(&Box)>(&self, build: F) {
        while let Some(child) = self.root.first_child() {
            self.root.remove(&child);
        }
        build(&self.root);
        self.backdrop.set_visible(true);
        self.backdrop.present();
        self.window.set_visible(true);
        self.window.present();
        motion::reveal(&self.root);
    }

    pub fn hide(&self) {
        self.active_anchor.set(None);
        self.backdrop.set_visible(false);
        let window = self.window.clone();
        motion::conceal(&self.root, move || window.set_visible(false));
    }

    pub fn is_visible(&self) -> bool {
        self.window.is_visible() && self.active_anchor.get().is_some()
    }

    pub fn is_open_for(&self, anchor: &impl IsA<gtk4::Widget>) -> bool {
        self.is_visible() && self.active_anchor.get() == Some(anchor.as_ptr() as usize)
    }

    /// Close both surfaces for good, when the bar's monitor goes away.
    pub fn destroy(&self) {
        self.window.destroy();
        self.backdrop.destroy();
    }
}

/// The header line of a popup.
pub fn header(text: &str) -> Label {
    let header = Label::new(Some(text));
    header.add_css_class("label");
    header.set_halign(gtk4::Align::Start);
    header.set_xalign(0.0);
    header
}

/// The large value line of a popup.
pub fn value(text: &str) -> Label {
    let value = Label::new(Some(text));
    value.add_css_class("value");
    value.set_halign(gtk4::Align::Start);
    value.set_xalign(0.0);
    value
}

/// A muted detail line of a popup.
pub fn detail(text: &str) -> Label {
    let detail = Label::new(Some(text));
    detail.add_css_class("dim");
    detail.set_halign(gtk4::Align::Start);
    detail.set_xalign(0.0);
    detail
}

/// A popup holding a single title-and-value pair: what a control shows when its
/// backend cannot be read.
pub fn message(popup: &Popup, title: &str, text: &str) {
    let (title, text) = (title.to_string(), text.to_string());
    popup.show(move |pop| {
        pop.append(&header(&title));
        pop.append(&value(&text));
    });
}

/// A fill bar whose thumb covers a share of a fixed track.
#[derive(Clone)]
pub struct FillBar {
    pub root: Box,
    thumb: Box,
}

impl FillBar {
    pub fn new(percent: u16) -> Self {
        let root = Box::new(Orientation::Horizontal, 0);
        root.add_css_class("fill");
        root.set_size_request(TRACK as i32, -1);
        root.set_valign(gtk4::Align::Center);
        let thumb = Box::new(Orientation::Vertical, 0);
        thumb.add_css_class("thumb");
        thumb.set_halign(gtk4::Align::Start);
        root.append(&thumb);
        let bar = Self { root, thumb };
        bar.set(percent);
        bar
    }

    /// Move the thumb to `percent`, clamped to the track.
    pub fn set(&self, percent: u16) {
        self.thumb.set_width_request(thumb_width(percent));
        self.thumb.set_visible(percent > 0);
    }
}

/// The thumb width for `percent` of the track.
pub fn thumb_width(percent: u16) -> i32 {
    (f64::from(percent.min(100)) / 100.0 * TRACK).round() as i32
}

#[cfg(test)]
mod tests {
    use super::thumb_width;

    #[test]
    fn the_thumb_covers_its_share_of_the_track() {
        assert_eq!(thumb_width(0), 0);
        assert_eq!(thumb_width(50), 75);
        assert_eq!(thumb_width(100), 150);
        assert_eq!(thumb_width(250), 150);
    }
}
