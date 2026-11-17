//! `exit`: the session menu: lock, suspend, log out, restart or power off.
//!
//! `FLUX_SHELL_PREVIEW=1` opens the menu with every choice disarmed, so it can
//! be screenshotted without ending anything.

mod actions;
mod text;

use gtk4::prelude::*;
use gtk4::{gdk, glib, Application, Box, Button, Image, Label, Orientation, Overlay};
use gtk4_layer_shell::{KeyboardMode, Layer};
use shell_common::app::run as run_app;
use shell_common::layer::{ensure_supported, LayerWindow};
use shell_common::{motion, theme};

use actions::Action;

const STYLE: &str = include_str!("../style.css");

fn main() {
    if std::env::args().skip(1).any(|argument| argument == "--version") {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return;
    }
    let application = Application::builder()
        .application_id("studio.facile.flux.Exit")
        .build();
    application.connect_activate(activate);
    let _ = run_app(&application, env!("CARGO_PKG_NAME"));
}

/// Whether the menu runs disarmed for a screenshot.
fn preview() -> bool {
    std::env::var_os("FLUX_SHELL_PREVIEW").is_some_and(|value| !value.is_empty())
}

/// Open the menu. A second `exit` hands its activation to this process, so an
/// activation while the menu is open is the key pressed again, and closes it.
fn activate(application: &Application) {
    if !application.windows().is_empty() {
        std::process::exit(0);
    }
    if !ensure_supported() {
        std::process::exit(1);
    }
    theme::init(STYLE, "exit");

    let layer = LayerWindow::new(application).layer(Layer::Overlay);
    layer.anchors(true, true, true, true);
    layer.exclusive_zone(false);
    layer.keyboard_mode(KeyboardMode::Exclusive);
    let window = layer.window;
    window.add_css_class("exit-overlay");

    let scrim = Box::new(Orientation::Vertical, 0);
    scrim.add_css_class("scrim");
    scrim.add_css_class("fx-fade");
    scrim.set_hexpand(true);
    scrim.set_vexpand(true);

    let (dialog, buttons) = dialog();
    let overlay = Overlay::new();
    overlay.set_child(Some(&scrim));
    overlay.add_overlay(&dialog);
    window.set_child(Some(&overlay));

    let parts = Parts { scrim, dialog, buttons };
    window.add_controller(keys(&parts));
    window.add_controller(click_outside(&parts));
    wire(&parts);

    window.present();
    motion::reveal(&parts.scrim);
    motion::reveal(&parts.dialog);
    if let Some(first) = parts.buttons.first() {
        first.grab_focus();
    }
}

/// The pieces the handlers reach back into.
#[derive(Clone)]
struct Parts {
    scrim: Box,
    dialog: Box,
    buttons: Vec<Button>,
}

/// The dialog: a title, a hint, one tile per choice and a cancel button.
fn dialog() -> (Box, Vec<Button>) {
    let strings = text::get();
    let surface = Box::new(Orientation::Vertical, 0);
    surface.add_css_class("surface");
    surface.add_css_class("exit-dialog");
    surface.add_css_class("fx-reveal");
    surface.set_halign(gtk4::Align::Center);
    surface.set_valign(gtk4::Align::Center);

    let title = Label::new(Some(strings.title));
    title.add_css_class("title");
    title.set_halign(gtk4::Align::Start);
    surface.append(&title);

    let subtitle = Label::new(Some(strings.subtitle));
    subtitle.add_css_class("subtitle");
    subtitle.set_halign(gtk4::Align::Start);
    surface.append(&subtitle);

    let row = Box::new(Orientation::Horizontal, 8);
    row.add_css_class("choices");
    let mut buttons: Vec<Button> = actions::ALL.iter().map(|action| tile(*action)).collect();
    for button in &buttons {
        row.append(button);
    }
    surface.append(&row);

    let cancel = Button::with_label(strings.cancel);
    cancel.add_css_class("cancel");
    cancel.set_halign(gtk4::Align::End);
    surface.append(&cancel);
    buttons.push(cancel);
    (surface, buttons)
}

/// One choice: its icon over its label.
fn tile(action: Action) -> Button {
    let content = Box::new(Orientation::Vertical, 8);
    let icon = Image::from_icon_name(action.icon());
    icon.set_pixel_size(24);
    content.append(&icon);
    content.append(&Label::new(Some(text::get().label(action))));

    let button = Button::new();
    button.set_child(Some(&content));
    button.add_css_class("choice");
    button.add_css_class(action.class());
    button
}

/// Wire every choice to its command and the cancel button to closing.
fn wire(parts: &Parts) {
    let count = actions::ALL.len();
    for (button, action) in parts.buttons.iter().zip(actions::ALL) {
        let parts_for = parts.clone();
        button.connect_clicked(move |_| choose(&parts_for, action));
    }
    if let Some(cancel) = parts.buttons.get(count) {
        let parts_for = parts.clone();
        cancel.connect_clicked(move |_| close(&parts_for));
    }
}

/// Run `action` and leave. The menu fades first, so the lock screen or the
/// greeter never shows it half drawn.
fn choose(parts: &Parts, action: Action) {
    motion::conceal(&parts.scrim, || {});
    motion::conceal(&parts.dialog, move || {
        if !preview() && !actions::run(&action.commands()) {
            eprintln!("exit: nothing could run {action:?}");
        }
        std::process::exit(0);
    });
}

/// Fade the menu out and leave without doing anything.
fn close(parts: &Parts) {
    motion::conceal(&parts.scrim, || {});
    motion::conceal(&parts.dialog, || std::process::exit(0));
}

/// `Esc` closes; the arrow keys walk the choices and wrap at either end.
fn keys(parts: &Parts) -> gtk4::EventControllerKey {
    let controller = gtk4::EventControllerKey::new();
    let parts = parts.clone();
    controller.connect_key_pressed(move |_, key, _, _| {
        let step = match key {
            gdk::Key::Escape => {
                close(&parts);
                return glib::Propagation::Stop;
            }
            gdk::Key::Left | gdk::Key::Up => -1,
            gdk::Key::Right | gdk::Key::Down | gdk::Key::Tab => 1,
            gdk::Key::ISO_Left_Tab => -1,
            _ => return glib::Propagation::Proceed,
        };
        let focused = parts.buttons.iter().position(|button| button.has_focus());
        let next = next_index(focused, step, parts.buttons.len());
        if let Some(button) = next.and_then(|index| parts.buttons.get(index)) {
            button.grab_focus();
        }
        glib::Propagation::Stop
    });
    controller
}

/// The index `step` away from `current` among `count` items, wrapping.
fn next_index(current: Option<usize>, step: i32, count: usize) -> Option<usize> {
    let count = i32::try_from(count).ok().filter(|count| *count > 0)?;
    let current = current.map_or(if step > 0 { -1 } else { count }, |index| index as i32);
    Some((current + step).rem_euclid(count) as usize)
}

/// A click on the scrim, outside the dialog, closes the menu.
fn click_outside(parts: &Parts) -> gtk4::GestureClick {
    let click = gtk4::GestureClick::new();
    let parts = parts.clone();
    click.connect_pressed(move |gesture, _, x, y| {
        let Some(window) = gesture.widget() else {
            return;
        };
        let point = gtk4::graphene::Point::new(x as f32, y as f32);
        let inside = parts
            .dialog
            .compute_bounds(&window)
            .is_some_and(|bounds| bounds.contains_point(&point));
        if !inside {
            close(&parts);
        }
    });
    click
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_arrows_wrap_around_the_choices() {
        assert_eq!(next_index(Some(0), 1, 6), Some(1));
        assert_eq!(next_index(Some(5), 1, 6), Some(0));
        assert_eq!(next_index(Some(0), -1, 6), Some(5));
        assert_eq!(next_index(None, 1, 6), Some(0));
        assert_eq!(next_index(None, -1, 6), Some(5));
        assert_eq!(next_index(None, 1, 0), None);
    }

    #[test]
    fn the_style_sheet_styles_the_dialog() {
        assert!(STYLE.contains(".exit-dialog"));
        assert!(STYLE.contains("button.choice"));
        assert!(STYLE.contains(&format!("button.choice.{}", Action::PowerOff.class())));
    }
}
