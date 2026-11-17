//! `statusbar`: one bar on every monitor, following niri.
//!
//! The bars live in one process. A monitor that appears gets a bar, one that
//! goes away takes its bar with it, and all of them draw from the one niri
//! event stream.

mod bar;
mod battery;
mod brightness;
mod control;
mod diagnose;
mod icons;
mod network;
mod niri;
mod popup;
mod stats;
mod sysfs;
#[cfg(test)]
mod testing;
mod text;
mod volume;
mod work;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib, Application};
use shell_common::app::run as run_app;
use shell_common::layer::ensure_supported;

use bar::Bar;

/// The name this widget claims in `$XDG_RUNTIME_DIR` while the bars live.
const WIDGET: &str = "statusbar";

fn main() {
    if std::env::args().skip(1).any(|arg| arg == "--version") {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return;
    }
    if std::env::args().skip(1).any(|arg| arg == "--diagnose") {
        print!("{}", diagnose::report());
        return;
    }
    if let Some(pid) = shell_common::instance::claim(WIDGET) {
        eprintln!("statusbar: already running (pid {pid}), leaving it alone");
        return;
    }
    let app = Application::builder()
        .application_id("studio.facile.flux.Statusbar")
        .build();
    app.connect_activate(activate);
    let code = run_app(&app, env!("CARGO_PKG_NAME"));
    shell_common::instance::release(WIDGET);
    std::process::exit(code.into());
}

/// The bars on screen and the niri state they draw.
struct Shell {
    app: Application,
    bars: RefCell<Vec<Rc<Bar>>>,
    state: RefCell<niri::State>,
    painted: Rc<RefCell<icons::Painted>>,
}

fn activate(app: &Application) {
    if !ensure_supported() {
        std::process::exit(1);
    }
    let Some(display) = gdk::Display::default() else {
        std::process::exit(1);
    };
    shell_common::theme::init(include_str!("../style.css"), "statusbar");
    std::mem::forget(app.hold());

    let painted = Rc::new(RefCell::new(icons::Painted::default()));
    icons::follow_palette(painted.clone());
    let shell = Rc::new(Shell {
        app: app.clone(),
        bars: RefCell::new(Vec::new()),
        state: RefCell::new(niri::State::default()),
        painted,
    });

    let monitors = display.monitors();
    sync(&shell, &monitors);
    let hotplug = shell.clone();
    monitors.connect_items_changed(move |monitors, _, _, _| sync(&hotplug, monitors));

    let (tx, rx) = async_channel::unbounded::<niri::Event>();
    let stream = shell.clone();
    glib::spawn_future_local(async move {
        while let Ok(event) = rx.recv().await {
            if stream.state.borrow_mut().apply(event) {
                for bar in stream.bars.borrow().iter() {
                    bar.render(&stream.state.borrow());
                }
            }
        }
    });
    niri::follow(tx);
}

/// Give every monitor a bar and drop the bars of monitors that are gone.
fn sync(shell: &Rc<Shell>, monitors: &gio::ListModel) {
    let present: Vec<gdk::Monitor> = (0..monitors.n_items())
        .filter_map(|position| monitors.item(position)?.downcast::<gdk::Monitor>().ok())
        .collect();
    let mut bars = shell.bars.borrow_mut();
    bars.retain(|bar| {
        let keep = present.contains(&bar.monitor);
        if !keep {
            bar.destroy();
        }
        keep
    });
    for monitor in present {
        if bars.iter().any(|bar| bar.monitor == monitor) {
            continue;
        }
        let bar = Bar::new(&shell.app, &monitor, &shell.painted);
        bar.render(&shell.state.borrow());
        redraw_when_named(shell, &bar);
        bars.push(bar);
    }
}

/// Draw `bar` again once its monitor's connector is known. On hotplug the
/// monitor is listed before its name arrives, and until then the bar cannot
/// tell its own output's workspaces from the others.
fn redraw_when_named(shell: &Rc<Shell>, bar: &Rc<Bar>) {
    let (shell, target) = (Rc::downgrade(shell), Rc::downgrade(bar));
    bar.monitor.connect_connector_notify(move |_| {
        if let (Some(shell), Some(bar)) = (shell.upgrade(), target.upgrade()) {
            bar.render(&shell.state.borrow());
        }
    });
}
