//! The lock itself: one surface per output, keys, and the unlock decision.
//!
//! [`Lock::submit`] is the only path to [`gtk4_session_lock::Instance::unlock`],
//! and it takes it only on [`Verdict::Unlock`]. Every other branch, including
//! a backend error, leaves the session locked.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib};
use zeroize::Zeroizing;

use crate::auth::{self, Verdict};
use crate::i18n::Text;
use crate::surface::{self, Surface};
use crate::{background, pam, status};

/// What a preview window does instead of asking PAM.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Preview {
    Off,
    On,
}

pub struct Lock {
    pub text: &'static Text,
    pub user: String,
    pub name: String,
    pub instance: Option<gtk4_session_lock::Instance>,
    preview: Preview,
    busy: Cell<bool>,
    surfaces: RefCell<Vec<Surface>>,
    wallpaper: RefCell<Option<gdk::MemoryTexture>>,
}

impl Lock {
    pub fn new(text: &'static Text, instance: Option<gtk4_session_lock::Instance>, preview: Preview) -> Rc<Lock> {
        let (user, name) = status::current_user().unwrap_or_else(|| (String::new(), String::new()));
        let lock = Rc::new(Lock {
            text,
            user,
            name,
            instance,
            preview,
            busy: Cell::new(false),
            surfaces: RefCell::new(Vec::new()),
            wallpaper: RefCell::new(None),
        });
        start_timers(&lock);
        load_wallpaper(&lock);
        lock
    }

    /// Fill `window` with a new surface and wire its keys.
    pub fn attach(self: &Rc<Self>, window: &gtk4::Window) -> Surface {
        let surface = surface::build(self.text, &self.name);
        window.add_css_class("flux-lock");
        window.set_child(Some(&surface.root));
        surface.tick(&now());
        if let Some(texture) = self.wallpaper.borrow().as_ref() {
            surface.set_background(texture);
        }
        wire_keys(self, window, &surface);
        let weak = Rc::downgrade(self);
        let root = surface.root.clone();
        window.connect_destroy(move |_| {
            if let Some(lock) = weak.upgrade() {
                lock.surfaces.borrow_mut().retain(|kept| kept.root != root);
            }
        });
        let shown = surface.clone();
        window.connect_map(move |_| shown.fade_in());
        self.surfaces.borrow_mut().push(surface.clone());
        refresh_status(self);
        surface
    }

    fn each(&self, apply: impl Fn(&Surface)) {
        for surface in self.surfaces.borrow().iter() {
            apply(surface);
        }
    }

    /// Check what was typed in `surface`. Runs PAM on a worker thread.
    pub fn submit(self: &Rc<Self>, surface: &Surface) {
        if self.busy.get() {
            return;
        }
        let secret = take_secret(&surface.entry);
        self.busy.set(true);
        self.each(|each| {
            each.entry.set_sensitive(false);
            each.entry.remove_css_class("error");
            each.message.set_text(self.text.checking);
        });
        let lock = self.clone();
        let user = self.user.clone();
        let preview = self.preview;
        glib::spawn_future_local(async move {
            let verdict = gio::spawn_blocking(move || match preview {
                Preview::Off => auth::verify(&pam::LibPam, &user, secret),
                Preview::On => Verdict::Wrong(None),
            })
            .await
            .unwrap_or_else(|_| Verdict::Unavailable("authentication worker panicked".into()));
            lock.settle(verdict);
        });
    }

    fn settle(&self, verdict: Verdict) {
        self.busy.set(false);
        let message = match verdict {
            Verdict::Unlock => {
                if let Some(instance) = &self.instance {
                    instance.unlock();
                }
                return;
            }
            Verdict::Wrong(pam_message) => pam_message.unwrap_or_else(|| self.text.wrong.to_string()),
            Verdict::Unavailable(reason) => {
                eprintln!("flux-lock: {reason}");
                self.text.unavailable.to_string()
            }
        };
        self.each(|each| {
            each.entry.set_sensitive(true);
            each.entry.add_css_class("error");
            each.message.set_text(&message);
        });
        if let Some(first) = self.surfaces.borrow().first() {
            first.entry.grab_focus();
        }
    }
}

/// Move the typed password out of `entry` into a buffer that wipes itself,
/// and wipe GTK's copy of it.
///
/// `PasswordEntry` keeps its text in a buffer GTK erases when it is cleared;
/// the `GString` copy `text()` hands out is overwritten here before it is freed.
fn take_secret(entry: &gtk4::PasswordEntry) -> Zeroizing<String> {
    let text = entry.text();
    let secret = Zeroizing::new(text.as_str().to_owned());
    let raw: *mut std::ffi::c_char = glib::translate::IntoGlibPtr::into_glib_ptr(text);
    if !raw.is_null() {
        unsafe {
            std::ptr::write_bytes(raw, 0, libc::strlen(raw));
            glib::ffi::g_free(raw.cast());
        }
    }
    entry.set_text("");
    secret
}

fn wire_keys(lock: &Rc<Lock>, window: &gtk4::Window, surface: &Surface) {
    let keys = gtk4::EventControllerKey::new();
    keys.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let weak = Rc::downgrade(lock);
    let target = surface.clone();
    keys.connect_key_pressed(move |_, key, _, _| {
        let Some(lock) = weak.upgrade() else {
            return glib::Propagation::Stop;
        };
        if lock.busy.get() {
            return glib::Propagation::Stop;
        }
        if key == gdk::Key::Escape {
            target.conceal();
            return glib::Propagation::Stop;
        }
        if target.revealer.reveals_child() && target.entry.has_focus() {
            return glib::Propagation::Proceed;
        }
        let typed = key.to_unicode().filter(|character| !character.is_control());
        target.reveal(typed);
        refresh_layout(&lock);
        glib::Propagation::Stop
    });
    window.add_controller(keys);
    let weak = Rc::downgrade(lock);
    let target = surface.clone();
    surface.entry.connect_activate(move |_| {
        if let Some(lock) = weak.upgrade() {
            lock.submit(&target);
        }
    });
    let entry = surface.entry.clone();
    surface.entry.connect_changed(move |_| entry.remove_css_class("error"));
}

fn now() -> glib::DateTime {
    glib::DateTime::now_local().unwrap_or_else(|_| glib::DateTime::from_unix_utc(0).expect("epoch is a valid date"))
}

fn start_timers(lock: &Rc<Lock>) {
    let weak = Rc::downgrade(lock);
    glib::timeout_add_seconds_local(1, move || match weak.upgrade() {
        Some(lock) => {
            let now = now();
            lock.each(|surface| surface.tick(&now));
            glib::ControlFlow::Continue
        }
        None => glib::ControlFlow::Break,
    });
    let weak = Rc::downgrade(lock);
    glib::timeout_add_seconds_local(60, move || match weak.upgrade() {
        Some(lock) => {
            refresh_status(&lock);
            glib::ControlFlow::Continue
        }
        None => glib::ControlFlow::Break,
    });
    if lock.preview == Preview::Off {
        watch_caps_lock(lock);
    }
}

fn watch_caps_lock(lock: &Rc<Lock>) {
    let Some(keyboard) = gdk::Display::default().and_then(|display| display.default_seat()).and_then(|seat| seat.keyboard()) else {
        return;
    };
    let show = |lock: &Lock, on: bool| lock.each(|surface| surface.caps.set_visible(on));
    show(lock, keyboard.is_caps_locked());
    let weak = Rc::downgrade(lock);
    keyboard.connect_caps_lock_state_notify(move |device| {
        if let Some(lock) = weak.upgrade() {
            show(&lock, device.is_caps_locked());
        }
    });
}

fn load_wallpaper(lock: &Rc<Lock>) {
    let weak = Rc::downgrade(lock);
    glib::spawn_future_local(async move {
        let loaded = gio::spawn_blocking(|| background::load(status::wallpaper()))
            .await
            .unwrap_or_else(|_| Err("wallpaper worker panicked".into()));
        let blurred = match loaded {
            Ok(blurred) => blurred,
            Err(reason) => {
                eprintln!("flux-lock: no wallpaper: {reason}");
                return;
            }
        };
        let Some(lock) = weak.upgrade() else {
            return;
        };
        let texture = background::texture(blurred);
        lock.each(|surface| surface.set_background(&texture));
        lock.wallpaper.replace(Some(texture));
    });
}

fn refresh_status(lock: &Rc<Lock>) {
    if lock.preview == Preview::On {
        return;
    }
    let weak = Rc::downgrade(lock);
    glib::spawn_future_local(async move {
        let battery = gio::spawn_blocking(status::battery).await.ok().flatten();
        let Some(lock) = weak.upgrade() else {
            return;
        };
        let line = battery.map(|(level, charging)| match charging {
            true => format!("{} · {}", lock.text.percent.replace("{n}", &level.to_string()), lock.text.charging),
            false => lock.text.percent.replace("{n}", &level.to_string()),
        });
        lock.each(|surface| {
            surface.battery.set_visible(line.is_some());
            surface.battery.set_text(line.as_deref().unwrap_or(""));
        });
    });
}

fn refresh_layout(lock: &Rc<Lock>) {
    if lock.preview == Preview::On {
        return;
    }
    let weak = Rc::downgrade(lock);
    glib::spawn_future_local(async move {
        let layout = gio::spawn_blocking(status::layout).await.ok().flatten();
        if let Some(lock) = weak.upgrade() {
            lock.each(|surface| {
                surface.layout.set_visible(layout.is_some());
                surface.layout.set_text(layout.as_deref().unwrap_or(""));
            });
        }
    });
}

/// Fill the preview window with sample state, for screenshots.
pub fn preview_state(lock: &Rc<Lock>, surface: &Surface, state: &str) {
    match state {
        "typing" => surface.reveal(Some('x')),
        "wrong" => {
            surface.reveal(None);
            lock.settle(Verdict::Wrong(None));
        }
        "unavailable" => {
            surface.reveal(None);
            lock.settle(Verdict::Unavailable("preview".into()));
        }
        "caps" => {
            surface.reveal(Some('x'));
            surface.caps.set_visible(true);
        }
        _ => {}
    }
    surface.layout.set_text("English (US)");
    surface.layout.set_visible(true);
    surface.battery.set_text(&lock.text.percent.replace("{n}", "76"));
    surface.battery.set_visible(true);
}
