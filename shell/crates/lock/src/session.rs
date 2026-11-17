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
use crate::keys::{self, KeyBuffer, Typed};
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
    held: RefCell<KeyBuffer>,
    typed_on: RefCell<Option<gtk4::Overlay>>,
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
            held: RefCell::new(KeyBuffer::new()),
            typed_on: RefCell::new(None),
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
        if self.preview == Preview::Off {
            surface.caps.set_visible(keyboard().is_some_and(|keyboard| keyboard.is_caps_locked()));
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
        self.held.borrow_mut().press(Typed::Forget);
        self.typed_on.replace(Some(surface.root.clone()));
        self.each(|each| {
            each.entry.set_sensitive(false);
            each.go.set_sensitive(false);
            each.set_error(false);
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

    /// The surface the last keys were typed on, or the first one.
    fn typed_surface(&self) -> Option<Surface> {
        let typed_on = self.typed_on.borrow();
        let surfaces = self.surfaces.borrow();
        surfaces
            .iter()
            .find(|surface| Some(&surface.root) == typed_on.as_ref())
            .or_else(|| surfaces.first())
            .cloned()
    }

    /// Keep a key pressed on `surface` while PAM is checking. Nothing is
    /// drawn for it; [`Lock::settle`] decides what becomes of it.
    fn hold(&self, surface: &Surface, typed: Typed) {
        if typed == Typed::Nothing {
            return;
        }
        self.held.borrow_mut().press(typed);
        self.typed_on.replace(Some(surface.root.clone()));
    }

    /// Act on PAM's answer. On anything but an unlock, the keys typed while
    /// it was checking go into the field of the surface they were typed on,
    /// which takes the focus back. On an unlock they are wiped unread.
    fn settle(&self, verdict: Verdict) {
        self.busy.set(false);
        let held = self.held.borrow_mut().take();
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
        let target = self.typed_surface();
        self.each(|each| {
            each.entry.set_sensitive(true);
            each.go.set_sensitive(true);
        });
        if let Some(target) = &target {
            target.append(&held);
        }
        self.each(|each| {
            each.set_error(true);
            each.message.set_text(&message);
        });
        if let Some(target) = &target {
            target.entry.grab_focus();
            target.entry.set_position(-1);
        }
    }
}

/// Move the typed password out of `entry` into a buffer that wipes itself,
/// and wipe GTK's copy of it.
///
/// `PasswordEntry` keeps its text in a buffer GTK erases when it is cleared;
/// [`surface::read_secret`] overwrites the copy `text()` hands out.
fn take_secret(entry: &gtk4::PasswordEntry) -> Zeroizing<String> {
    let secret = surface::read_secret(entry);
    entry.set_text("");
    secret
}

/// Wire the keys, the clicks and the submit paths of one surface.
fn wire_keys(lock: &Rc<Lock>, window: &gtk4::Window, surface: &Surface) {
    window.add_controller(key_presses(lock, surface));
    window.add_controller(clicks(lock, surface));
    let weak = Rc::downgrade(lock);
    let target = surface.clone();
    surface.entry.connect_activate(move |_| {
        if let Some(lock) = weak.upgrade() {
            lock.submit(&target);
        }
    });
    let weak = Rc::downgrade(lock);
    let target = surface.clone();
    surface.go.connect_clicked(move |_| {
        if let Some(lock) = weak.upgrade() {
            lock.submit(&target);
        }
    });
    let target = surface.clone();
    surface.entry.connect_changed(move |_| target.set_error(false));
}

/// The key handling of one surface, ahead of every widget.
///
/// While PAM is checking, every key is held and none reaches a widget. Escape
/// hides the field. Once the field shows and has the focus, keys go to it
/// untouched, through its input method. Before that, a key shows the field,
/// and a key that types a character puts it there; a shortcut (Control, Alt
/// or Super held) shows the field and types nothing.
fn key_presses(lock: &Rc<Lock>, surface: &Surface) -> gtk4::EventControllerKey {
    let pressed = gtk4::EventControllerKey::new();
    pressed.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let weak = Rc::downgrade(lock);
    let target = surface.clone();
    pressed.connect_key_pressed(move |_, key, _, state| {
        let Some(lock) = weak.upgrade() else {
            return glib::Propagation::Stop;
        };
        let typed = keys::typed(key, state);
        if lock.busy.get() {
            lock.hold(&target, typed);
            return glib::Propagation::Stop;
        }
        if typed == Typed::Forget {
            target.conceal();
            return glib::Propagation::Stop;
        }
        if target.revealed() && target.typing() {
            return glib::Propagation::Proceed;
        }
        target.reveal(match typed {
            Typed::Character(character) => Some(character),
            _ => None,
        });
        refresh_layout(&lock);
        glib::Propagation::Stop
    });
    pressed
}

/// A click or a tap shows the field, as a key does: the hint says "any key",
/// and a touch screen has none.
fn clicks(lock: &Rc<Lock>, surface: &Surface) -> gtk4::GestureClick {
    let click = gtk4::GestureClick::new();
    let weak = Rc::downgrade(lock);
    let target = surface.clone();
    click.connect_pressed(move |_, _, _, _| {
        let Some(lock) = weak.upgrade() else {
            return;
        };
        if !lock.busy.get() && !target.revealed() {
            target.reveal(None);
            refresh_layout(&lock);
        }
    });
    click
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

/// The seat's keyboard, which knows whether Caps Lock is on.
fn keyboard() -> Option<gdk::Device> {
    gdk::Display::default()?.default_seat()?.keyboard()
}

/// Keep the Caps Lock note of every surface in step with the keyboard. A
/// surface created later reads the state itself, in [`Lock::attach`].
fn watch_caps_lock(lock: &Rc<Lock>) {
    let Some(keyboard) = keyboard() else {
        return;
    };
    let weak = Rc::downgrade(lock);
    keyboard.connect_caps_lock_state_notify(move |device| {
        if let Some(lock) = weak.upgrade() {
            let on = device.is_caps_locked();
            lock.each(|surface| surface.caps.set_visible(on));
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
