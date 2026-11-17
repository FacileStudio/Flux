//! The Flux lock screen, on the Wayland ext-session-lock-v1 protocol. The
//! `flux-lock` binary is a thin front for [`run`]; see the crate README for the
//! security model.
//!
//! The preview used for screenshots lives in `examples/preview.rs` and calls
//! [`preview`]. The shipped binary has no path to it: no flag or environment
//! variable changes what `flux-lock` does.

mod auth;
mod background;
mod i18n;
pub mod launch;
mod pam;
mod session;
mod status;
mod surface;

use std::cell::Cell;
use std::io::Write;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::Application;
use session::{Lock, Preview};

const CSS: &str = include_str!("../style.css");

fn notify_locked() {
    let mut stdout = std::io::stdout();
    let _ = writeln!(stdout, "locked");
    let _ = stdout.flush();
}

/// Ignore SIGTERM, SIGINT and SIGHUP: a locker that quits on a signal would
/// let anything that can signal it unlock the screen. SIGKILL cannot be
/// ignored; the compositor then keeps the outputs blanked and locked.
fn ignore_termination() {
    for signal in [libc::SIGTERM, libc::SIGINT, libc::SIGHUP] {
        unsafe { libc::signal(signal, libc::SIG_IGN) };
    }
}

/// Lock the session and run until it is unlocked. With `notify`, print
/// `locked` on stdout once the compositor confirms the lock.
pub fn run_locked(notify: bool) -> i32 {
    let guard = match launch::claim() {
        Ok(Some(guard)) => guard,
        Ok(None) => {
            if notify {
                notify_locked();
            }
            return 0;
        }
        Err(error) => {
            eprintln!("flux-lock: cannot take the instance lock: {error}");
            return 1;
        }
    };
    ignore_termination();
    let failed = Rc::new(Cell::new(false));
    let application = Application::builder()
        .application_id("studio.facile.FluxLock")
        .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    let flag = failed.clone();
    application.connect_activate(move |app| activate(app, notify, flag.clone()));
    let _ = shell_common::app::run(&application, "flux-lock");
    drop(guard);
    i32::from(failed.get())
}

fn activate(app: &Application, notify: bool, failed: Rc<Cell<bool>>) {
    shell_common::theme::init(CSS, "flux-lock");
    if !gtk4_session_lock::is_supported() {
        eprintln!("flux-lock: this compositor does not support ext-session-lock-v1");
        failed.set(true);
        return;
    }
    let hold = Rc::new(app.hold());
    let instance = gtk4_session_lock::Instance::new();
    let lock = Lock::new(i18n::current(), Some(instance.clone()), Preview::Off);
    let owner = app.clone();
    instance.connect_monitor(move |instance, monitor| {
        let window = gtk4::ApplicationWindow::new(&owner);
        lock.attach(window.upcast_ref());
        instance.assign_window_to_monitor(&window, monitor);
        window.present();
    });
    instance.connect_locked(move |_| {
        if notify {
            notify_locked();
        }
    });
    let held = hold.clone();
    instance.connect_failed(move |_| {
        eprintln!("flux-lock: the compositor refused the lock (another locker may hold it)");
        failed.set(true);
        let _ = &held;
        if let Some(app) = gtk4::gio::Application::default() {
            app.quit();
        }
    });
    instance.connect_unlocked(move |_| {
        let _ = &hold;
        if let Some(app) = gtk4::gio::Application::default() {
            app.quit();
        }
    });
    instance.lock();
}

/// Open the lock surface in a plain window with sample state, for
/// screenshots: `<lang>:<idle|typing|wrong|unavailable|caps>`. Nothing is
/// locked and PAM is never called.
pub fn preview(spec: &str) -> i32 {
    let (language, state) = spec.split_once(':').unwrap_or((spec, "idle"));
    let text = i18n::text(language);
    let state = state.to_string();
    let application = Application::builder()
        .application_id("studio.facile.FluxLockPreview")
        .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    application.connect_activate(move |app| {
        shell_common::theme::init(CSS, "flux-lock");
        let lock = Lock::new(text, None, Preview::On);
        let window = gtk4::ApplicationWindow::new(app);
        window.set_title(Some(text.preview));
        window.set_default_size(1280, 800);
        let surface = lock.attach(window.upcast_ref());
        window.present();
        session::preview_state(&lock, &surface, &state);
        window.connect_destroy(move |_| {
            let _ = &lock;
        });
    });
    let _ = shell_common::app::run(&application, "flux-lock");
    0
}
