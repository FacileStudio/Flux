//! The right-hand control row: one pill per control, each leading with its icon
//! and opening its own popup on click, then the session pill.
//!
//! Every pill reads its backend off the main loop on a timer of its own and
//! shows itself only while the backend answers. A service that comes up after
//! the bar (pipewire-pulse at login, a hot-plugged battery) gets its pill on
//! the next tick instead of never.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};

use crate::icons;
use crate::popup::Popup;
use crate::{battery, brightness, network, text, volume, work};

/// One control: its icon, how often its pill is read, the read itself and the
/// popup it opens.
pub struct Control {
    pub name: &'static str,
    pub icon: icons::Icon,
    pub every: u32,
    pub label: fn() -> Option<String>,
    pub show: fn(&Popup),
}

/// Every control, in the order the row reads them.
pub const CONTROLS: [Control; 4] = [
    Control {
        name: "volume",
        icon: icons::Icon::Volume,
        every: 3,
        label: volume::label,
        show: volume::show,
    },
    Control {
        name: "brightness",
        icon: icons::Icon::Brightness,
        every: 3,
        label: brightness::label,
        show: brightness::show,
    },
    Control {
        name: "battery",
        icon: icons::Icon::Battery,
        every: 30,
        label: battery::label,
        show: battery::show,
    },
    Control {
        name: "network",
        icon: icons::Icon::Network,
        every: 10,
        label: network::label,
        show: network::show,
    },
];

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
    let label = Label::new(None);
    let child = Box::new(Orientation::Horizontal, 4);
    child.append(&painted.borrow_mut().image(control.icon));
    child.append(&label);

    let button = Button::new();
    button.set_child(Some(&child));
    button.add_css_class("pill");
    button.add_css_class(control.name);
    button.set_visible(false);

    let popup = popup.clone();
    button.connect_clicked(move |anchor| {
        if popup.is_open_for(anchor) {
            popup.hide();
            return;
        }
        popup.position_under(anchor);
        (control.show)(&popup);
        refresh(control, anchor, &label_of(anchor));
    });

    refresh(control, &button, &label);
    let weak = button.downgrade();
    glib::timeout_add_seconds_local(control.every, move || {
        let Some(button) = weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        refresh(control, &button, &label_of(&button));
        glib::ControlFlow::Continue
    });
    button
}

/// The label a pill paints beside its icon.
fn label_of(button: &Button) -> Label {
    button
        .child()
        .and_then(|child| child.last_child())
        .and_then(|label| label.downcast::<Label>().ok())
        .unwrap_or_default()
}

/// Read the control off the main loop and show what it said, or hide the pill.
fn refresh(control: &'static Control, button: &Button, label: &Label) {
    let (button, label) = (button.downgrade(), label.downgrade());
    work::off_main(control.label, move |text| {
        let (Some(button), Some(label)) = (button.upgrade(), label.upgrade()) else {
            return;
        };
        if let Some(text) = &text {
            label.set_text(text);
        }
        button.set_visible(text.is_some());
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
