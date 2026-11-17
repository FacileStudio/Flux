//! The right-hand control row: one pill per control, each leading with its icon
//! and opening its own popup on click, then the session pill.
//!
//! Every pill reads its backend off the main loop on a timer of its own and
//! shows itself only while the backend answers. A service that comes up after
//! the bar (pipewire-pulse at login, a hot-plugged battery) gets its pill on
//! the next tick instead of never. A control that can report its own changes
//! is followed as well, so its pills move with the keys and the timer is only
//! the safety net.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation};

use crate::icons;
use crate::popup::Popup;
use crate::{battery, brightness, network, text, volume, work};

/// What one read of a control says: the words beside the icon, the icon to
/// draw, and whether the pill warns.
pub struct Reading {
    pub text: String,
    pub icon: icons::Icon,
    pub low: bool,
}

impl Reading {
    /// A reading that only says `text` beside `icon`.
    pub fn of(icon: icons::Icon, text: String) -> Self {
        Self { text, icon, low: false }
    }
}

/// Where a followed control reports that it changed. The channel holds one
/// tick, so a burst of changes costs one read.
pub type Changes = async_channel::Sender<()>;

/// One control: its icon, how often its pill is read, the read itself, what
/// reports its changes when something can, and the popup it opens.
pub struct Control {
    pub name: &'static str,
    pub icon: icons::Icon,
    pub every: u32,
    pub read: fn() -> Option<Reading>,
    pub follow: Option<fn(Changes)>,
    pub show: fn(&Popup),
}

/// Every control, in the order the row reads them.
pub const CONTROLS: [Control; 4] = [
    Control {
        name: "volume",
        icon: icons::Icon::Volume,
        every: 30,
        read: volume::reading,
        follow: Some(volume::follow),
        show: volume::show,
    },
    Control {
        name: "brightness",
        icon: icons::Icon::Brightness,
        every: 3,
        read: brightness::reading,
        follow: Some(brightness::follow),
        show: brightness::show,
    },
    Control {
        name: "battery",
        icon: icons::Icon::Battery,
        every: 30,
        read: battery::pill,
        follow: Some(battery::follow),
        show: battery::show,
    },
    Control {
        name: "network",
        icon: icons::Icon::Network,
        every: 10,
        read: network::reading,
        follow: None,
        show: network::show,
    },
];

/// A pill's refresh, answering whether the pill is still on a bar.
type Refresh = std::boxed::Box<dyn Fn() -> bool>;

thread_local! {
    /// The pills of every followed control, by control name. One follower runs
    /// per control for the whole process, whatever the number of bars.
    static FOLLOWED: RefCell<HashMap<&'static str, Vec<Refresh>>> = RefCell::new(HashMap::new());
}

/// Build the row. `painted` collects the glyphs so the palette can reach them
/// later; see [`icons::follow_palette`].
pub fn build(popup: &Popup, painted: &Rc<RefCell<icons::Painted>>) -> Box {
    let row = Box::new(Orientation::Horizontal, shell_common::theme::MODULE_GAP);
    row.add_css_class("controls");
    row.set_halign(gtk4::Align::End);
    for control in &CONTROLS {
        row.append(&pill(control, popup, painted));
    }
    row.append(&session(painted));
    row
}

/// One control's pill: hidden until its first read answers, then kept current.
fn pill(control: &'static Control, popup: &Popup, painted: &Rc<RefCell<icons::Painted>>) -> Button {
    let child = Box::new(Orientation::Horizontal, 4);
    child.append(&painted.borrow_mut().image(control.icon));
    child.append(&Label::new(None));

    let button = Button::new();
    button.set_child(Some(&child));
    button.add_css_class("pill");
    button.add_css_class(control.name);
    button.set_visible(false);

    let popup = popup.clone();
    let clicked = painted.clone();
    button.connect_clicked(move |anchor| {
        if popup.is_open_for(anchor) {
            popup.hide();
            return;
        }
        popup.position_under(anchor);
        (control.show)(&popup);
        refresh(control, anchor, &clicked);
    });

    refresh(control, &button, painted);
    let (weak, ticked) = (button.downgrade(), painted.clone());
    glib::timeout_add_seconds_local(control.every, move || {
        let Some(button) = weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        refresh(control, &button, &ticked);
        glib::ControlFlow::Continue
    });
    let (weak, changed) = (button.downgrade(), painted.clone());
    follow(control, move || {
        let Some(button) = weak.upgrade() else {
            return false;
        };
        refresh(control, &button, &changed);
        true
    });
    button
}

/// Run `refresh` every time `control` reports a change, for as long as it
/// answers `true`. The first pill of a control starts its follower.
fn follow(control: &'static Control, refresh: impl Fn() -> bool + 'static) {
    let Some(start) = control.follow else {
        return;
    };
    let first = FOLLOWED.with(|slot| {
        let mut slot = slot.borrow_mut();
        let first = !slot.contains_key(control.name);
        slot.entry(control.name).or_default().push(std::boxed::Box::new(refresh));
        first
    });
    if !first {
        return;
    }
    let (changes, changed) = async_channel::bounded::<()>(1);
    start(changes);
    glib::spawn_future_local(async move {
        while changed.recv().await.is_ok() {
            FOLLOWED.with(|slot| {
                if let Some(pills) = slot.borrow_mut().get_mut(control.name) {
                    pills.retain(|refresh| refresh());
                }
            });
        }
    });
}

/// The glyph and the label a pill paints, in that order.
fn parts(button: &Button) -> Option<(Image, Label)> {
    let row = button.child()?;
    let glyph = row.first_child()?.downcast::<Image>().ok()?;
    let label = row.last_child()?.downcast::<Label>().ok()?;
    Some((glyph, label))
}

/// Read the control off the main loop and show what it said, or hide the pill.
fn refresh(control: &'static Control, button: &Button, painted: &Rc<RefCell<icons::Painted>>) {
    let (button, painted) = (button.downgrade(), painted.clone());
    work::off_main(control.read, move |reading| {
        let Some(button) = button.upgrade() else {
            return;
        };
        if let (Some(reading), Some((glyph, label))) = (&reading, parts(&button)) {
            label.set_text(&reading.text);
            label.set_visible(!reading.text.is_empty());
            painted.borrow_mut().swap(&glyph, reading.icon);
            if reading.low {
                button.add_css_class("low");
            } else {
                button.remove_css_class("low");
            }
        }
        button.set_visible(reading.is_some());
    });
}

/// The session pill: the glyph alone, opening the exit menu.
fn session(painted: &Rc<RefCell<icons::Painted>>) -> Button {
    let button = Button::new();
    button.set_child(Some(&painted.borrow_mut().image(icons::Icon::Logout)));
    button.add_css_class("pill");
    button.add_css_class("session");
    button.set_tooltip_text(Some(text::get().session_tooltip));
    button.connect_clicked(|_| shell_common::spawn::detached(shell_common::spawn::sibling("exit"), &[]));
    button
}

#[cfg(test)]
mod tests {
    use gtk4::gio;

    use super::*;

    #[test]
    fn every_control_has_a_distinct_name() {
        let mut names: Vec<&str> = CONTROLS.iter().map(|control| control.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), CONTROLS.len());
    }

    #[test]
    fn a_pill_carries_its_glyph_beside_its_label_and_waits_for_a_reading() {
        let ran = crate::testing::on_gtk_thread(|| {
            let application = gtk4::Application::builder()
                .application_id("studio.facile.flux.ControlPillTest")
                .build();
            application
                .register(None::<&gio::Cancellable>)
                .expect("register the test application");
            let popup = Popup::new(&application, None);
            let painted = Rc::new(RefCell::new(icons::Painted::default()));
            let button = pill(&CONTROLS[0], &popup, &painted);

            assert!(!button.is_visible(), "a pill stays hidden until its backend answers");
            let row = button
                .child()
                .and_then(|child| child.downcast::<Box>().ok())
                .expect("the pill holds a row");
            let glyph = row
                .first_child()
                .and_then(|child| child.downcast::<gtk4::Image>().ok())
                .expect("the row leads with the pill's icon");
            assert!(glyph.paintable().is_some(), "the icon has a texture to paint");
            assert!(
                glyph.next_sibling().is_some_and(|next| next.is::<Label>()),
                "and the label comes after it"
            );
        });
        if !ran {
            eprintln!("statusbar: no display, skipping the pill test");
        }
    }
}
