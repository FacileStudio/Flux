//! `launcher`: the application palette.
//!
//! This file is the entry point and the window placement. The surface itself
//! (the query field, the results, the keys) is [`popup`].

mod apps;
mod exec;
mod popup;
mod text;

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, Application, Box, Orientation};
use gtk4_layer_shell::{KeyboardMode, Layer};
use shell_common::app::run as run_app;
use shell_common::layer::{ensure_supported, LayerWindow};
use shell_common::{motion, theme};

use apps::App;

const STYLE: &str = include_str!("../style.css");
const WIDTH: i32 = 600;
const TOP_RATIO: f64 = 0.28;

fn main() {
    if std::env::args().skip(1).any(|argument| argument == "--version") {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return;
    }
    let apps = Rc::new(apps::discover(&apps::Env::current()));
    let application = Application::builder()
        .application_id("studio.facile.flux.Launcher")
        .build();
    let shared = apps.clone();
    application.connect_activate(move |app| activate(app, &shared));
    let _ = run_app(&application, env!("CARGO_PKG_NAME"));
}

fn activate(application: &Application, apps: &Rc<Vec<App>>) {
    if !ensure_supported() {
        std::process::exit(1);
    }
    theme::init(STYLE, "launcher");

    let window = layer_window(application).window;
    let (entry, list) = popup::build();

    let surface = Box::new(Orientation::Vertical, 0);
    surface.add_css_class("surface");
    surface.add_css_class("popup");
    surface.add_css_class("fx-reveal");
    surface.set_width_request(WIDTH);
    surface.set_halign(gtk4::Align::Center);
    surface.set_valign(gtk4::Align::Start);
    surface.append(&entry);
    surface.append(&list.scroll);
    surface.append(&list.empty);
    window.set_child(Some(&surface));

    popup::install(&window, &surface, &entry, &list, apps);
    place_on_map(&window, &surface);
    window.present();
    motion::reveal(&surface);
    entry.grab_focus();
}

/// The transparent overlay the launcher floats on. It covers the whole output,
/// so a click beside the palette closes it.
fn layer_window(application: &Application) -> LayerWindow {
    let launcher = LayerWindow::new(application).layer(Layer::Overlay);
    launcher.anchors(true, true, true, true);
    launcher.exclusive_zone(false);
    launcher.keyboard_mode(KeyboardMode::Exclusive);
    launcher
}

/// Once the compositor has put the overlay on an output, drop the palette a
/// little over a quarter of the way down that output.
fn place_on_map(window: &gtk4::ApplicationWindow, surface: &Box) {
    let surface = surface.clone();
    window.connect_map(move |window| {
        let height = window
            .surface()
            .and_then(|gdk_surface| {
                gdk::Display::default().and_then(|display| display.monitor_at_surface(&gdk_surface))
            })
            .map(|monitor| monitor.geometry().height())
            .unwrap_or(1080);
        surface.set_margin_top((f64::from(height) * TOP_RATIO) as i32);
    });
}
