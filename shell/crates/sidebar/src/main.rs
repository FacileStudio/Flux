//! `sidebar`: the notification panel.
//!
//! This file is only the entry point - the pid file, the `SIGUSR1` toggle and
//! the window placement. The panel itself is [`panel::App`].

mod dbus;
mod model;
mod panel;
mod popup;
mod text;
mod wallpaper;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{glib, Application};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use shell_common::app::run as run_app;
use shell_common::{instance, layer::LayerWindow};

use dbus::Event;
use panel::{apply_event, App};

/// The name this widget claims in `$XDG_RUNTIME_DIR` while the daemon lives.
const WIDGET: &str = "sidebar";

thread_local! {
    static APP: RefCell<Option<Rc<App>>> = const { RefCell::new(None) };
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--version") => println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        // The compositor binding. With no daemon running yet, starting one *is*
        // the toggle, so the panel must come up visible: a hidden daemon would
        // make the first `mod+n` look like it did nothing.
        Some("toggle") => toggle_or_start(true),
        // `sidebar` on its own primes the daemon without covering the screen,
        // which is what a compositor's `exec` line wants.
        None => toggle_or_start(false),
        Some("wallpaper") => std::process::exit(wallpaper::cli(&args[2..])),
        Some(_) => {
            eprintln!("usage: sidebar [--version|toggle|wallpaper ...]");
            std::process::exit(2);
        }
    }
}

/// Toggle the running daemon, or become it. A bare `sidebar` (`show` false)
/// leaves a running daemon alone instead of opening its panel.
fn toggle_or_start(show: bool) {
    if let Some(pid) = instance::running(WIDGET) {
        if !show || unsafe { libc::kill(pid, libc::SIGUSR1) } == 0 {
            std::process::exit(0);
        }
    }
    run_daemon(show);
}

fn run_daemon(show: bool) {
    // Hook SIGUSR1 before the pid file exists. A toggle that lands in between
    // is then queued for the main loop, instead of running the signal's default
    // action and killing the daemon while it starts.
    let mut signals = match signal_hook::iterator::Signals::new([libc::SIGUSR1]) {
        Ok(signals) => signals,
        Err(err) => {
            eprintln!("sidebar: cannot hook SIGUSR1: {err}");
            std::process::exit(1);
        }
    };
    if let Some(pid) = instance::claim(WIDGET) {
        // Another copy won the race for the pid file: hand it the toggle.
        unsafe { libc::kill(pid, libc::SIGUSR1) };
        std::process::exit(0);
    }
    std::thread::spawn(move || {
        for _ in signals.forever() {
            let _ = glib::idle_add_once(toggle_from_signal);
        }
    });
    let items = model::load();
    let next_id = items
        .iter()
        .map(|item| item.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    let application = Application::builder()
        .application_id("studio.facile.flux.Sidebar")
        .build();
    application.connect_activate(move |app| activate(app, items.clone(), next_id, show));
    let code = run_app(&application, env!("CARGO_PKG_NAME"));
    instance::release(WIDGET);
    std::process::exit(code.into());
}

fn activate(app: &Application, items: Vec<model::Notification>, next_id: u32, show: bool) {
    if let Some(existing) = APP.with(|slot| slot.borrow().clone()) {
        existing.show();
        return;
    }
    if !shell_common::layer::ensure_supported() {
        instance::release(WIDGET);
        std::process::exit(1);
    }
    shell_common::theme::init(include_str!("../style.css"), "sidebar");

    let layer = LayerWindow::new(app).layer(Layer::Overlay);
    layer.anchors(true, false, true, true);
    layer.exclusive_zone(false);
    layer.keyboard_mode(KeyboardMode::OnDemand);
    layer.window.set_width_request(360);
    let window = &layer.window;
    window.set_margin(Edge::Top, shell_common::theme::WINDOW_TOP_GAP);
    window.set_margin(Edge::Right, shell_common::theme::GAP);
    window.set_margin(Edge::Bottom, shell_common::theme::GAP);

    let handle = dbus::start(next_id);
    let built = App::build(app, layer.window.clone(), items, handle);
    if show {
        built.show();
    }
    if preview() {
        for notification in sample() {
            apply_event(&built, &Event::Added(notification));
        }
    }
    APP.with(|slot| *slot.borrow_mut() = Some(built));
}

/// Whether the panel opens with sample notifications for a screenshot.
/// `FLUX_SHELL_PREVIEW=1 XDG_STATE_HOME=/tmp/x sidebar toggle` keeps the
/// sample out of the real history.
fn preview() -> bool {
    std::env::var_os("FLUX_SHELL_PREVIEW").is_some_and(|value| !value.is_empty())
}

/// Two notifications, one of them critical.
fn sample() -> Vec<model::Notification> {
    let now = model::now();
    vec![
        model::Notification {
            id: 1,
            app_name: "Calendar".into(),
            summary: "Design review in 10 minutes".into(),
            body: "Room 2, with the Flux team".into(),
            ts: now - 900,
            urgent: false,
        },
        model::Notification {
            id: 2,
            app_name: "Battery".into(),
            summary: "Battery at 5%".into(),
            body: "Plug in the charger now".into(),
            ts: now,
            urgent: true,
        },
    ]
}

fn toggle_from_signal() {
    APP.with(|slot| {
        if let Some(app) = slot.borrow().as_ref() {
            app.toggle();
        }
    });
}

fn deliver(event: Event) {
    let _ = glib::idle_add_once(move || {
        APP.with(|slot| {
            if let Some(app) = slot.borrow().as_ref() {
                apply_event(app, &event);
            }
        });
    });
}
