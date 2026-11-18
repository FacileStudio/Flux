//! `launcher`: the application palette.
//!
//! This file is the entry point and the window placement. The surface itself
//! (the query field, the results, the keys) is [`popup`].

mod apps;
mod exec;
mod popup;
mod text;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::gio::ApplicationHoldGuard;
use gtk4::prelude::*;
use gtk4::{gdk, Application, ApplicationWindow, Box, Entry, Orientation};
use gtk4_layer_shell::{KeyboardMode, Layer};
use shell_common::app::run as run_app;
use shell_common::layer::{ensure_supported, LayerWindow};
use shell_common::{motion, theme};

use popup::{Catalog, Results};

const STYLE: &str = include_str!("../style.css");
const WIDTH: i32 = 600;
const TOP_RATIO: f64 = 0.28;

/// The palette, built once and kept for the whole session: an activation only
/// shows it, so opening it costs no process start, icon lookup or row build.
struct Launcher {
    window: ApplicationWindow,
    surface: Box,
    entry: Entry,
    results: Results,
    catalog: Rc<RefCell<Catalog>>,
    matches: Rc<RefCell<Vec<usize>>>,
    _hold: ApplicationHoldGuard,
}

fn main() {
    let first = std::env::args().nth(1);
    if first.as_deref() == Some("--version") {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return;
    }
    let background = first.as_deref() == Some("--background");
    let application = Application::builder()
        .application_id("studio.facile.flux.Launcher")
        .build();
    if background && application.register(None::<&gtk4::gio::Cancellable>).is_ok() && application.is_remote() {
        return;
    }
    let launcher: Rc<RefCell<Option<Launcher>>> = Rc::default();
    let quiet = Cell::new(background);
    application.connect_activate(move |app| activate(app, &launcher, quiet.replace(false)));
    let _ = run_app(&application, env!("CARGO_PKG_NAME"));
}

/// Show the palette, or close it when it is already open: a second `launcher`
/// hands its activation to this process, so that is the key pressed again.
/// `quiet` builds the palette without showing it, for the session service.
fn activate(application: &Application, launcher: &Rc<RefCell<Option<Launcher>>>, quiet: bool) {
    if launcher.borrow().is_none() {
        if !ensure_supported() {
            std::process::exit(1);
        }
        theme::init(STYLE, "launcher");
        *launcher.borrow_mut() = Some(build(application));
    }
    let launcher = launcher.borrow();
    let Some(launcher) = launcher.as_ref() else {
        return;
    };
    if quiet {
        return;
    }
    if launcher.window.is_visible() {
        popup::close(&launcher.surface);
    } else {
        show(launcher);
    }
}

fn build(application: &Application) -> Launcher {
    let window = layer_window(application).window;
    let (entry, results) = popup::build();

    let surface = Box::new(Orientation::Vertical, 0);
    surface.add_css_class("surface");
    surface.add_css_class("popup");
    surface.add_css_class("fx-reveal");
    surface.set_width_request(WIDTH);
    surface.set_halign(gtk4::Align::Center);
    surface.set_valign(gtk4::Align::Start);
    surface.append(&entry);
    surface.append(&results.scroll);
    surface.append(&results.empty);
    window.set_child(Some(&surface));

    let catalog = Rc::new(RefCell::new(Catalog::new(apps::discover(&apps::Env::current()))));
    let matches = popup::install(&window, &surface, &entry, &results, &catalog);
    place_on_map(&window, &surface);
    Launcher { window, surface, entry, results, catalog, matches, _hold: application.hold() }
}

/// Bring the palette up empty, with the apps installed since it last opened.
fn show(launcher: &Launcher) {
    let found = apps::discover(&apps::Env::current());
    if found != launcher.catalog.borrow().apps {
        *launcher.catalog.borrow_mut() = Catalog::new(found);
    }
    launcher.entry.set_text("");
    *launcher.matches.borrow_mut() = popup::fill(&launcher.results, &launcher.catalog.borrow(), "");
    launcher.window.present();
    motion::reveal(&launcher.surface);
    launcher.entry.grab_focus();
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
