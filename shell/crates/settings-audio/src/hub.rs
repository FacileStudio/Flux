//! One connection per GTK thread, shared by the page and the quick menu. Events from the
//! backend are handed to every listener on the GTK main loop.

use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

use gtk4::{gio, glib};

use crate::backend::{self, Backend, Command, Event};
use crate::model::{Options, Snapshot};

/// The environment variable that swaps the sound server for sample data.
pub const PREVIEW: &str = "FLUX_AUDIO_PREVIEW";

type Listener = Rc<dyn Fn(&Event)>;

pub struct Hub {
    backend: Box<dyn Backend>,
    listeners: RefCell<Vec<(u64, Listener)>>,
    next: Cell<u64>,
    last: RefCell<Option<Event>>,
    options: Cell<Option<Options>>,
    meters: Cell<u32>,
    preview: bool,
}

thread_local! {
    static HUB: OnceCell<Rc<Hub>> = const { OnceCell::new() };
}

/// The thread's hub, connecting on first use.
pub fn get() -> Rc<Hub> {
    HUB.with(|cell| Rc::clone(cell.get_or_init(Hub::start)))
}

impl Hub {
    fn start() -> Rc<Hub> {
        let mode = std::env::var(PREVIEW).ok();
        let preview = mode.is_some();
        let (backend, events) = match mode {
            Some(mode) => backend::preview(&mode),
            None => backend::connect(),
        };
        let hub = Rc::new(Hub {
            backend,
            listeners: RefCell::new(Vec::new()),
            next: Cell::new(0),
            last: RefCell::new(None),
            options: Cell::new(None),
            meters: Cell::new(0),
            preview,
        });
        forward(&hub, events);
        load_options(&hub);
        hub
    }

    fn dispatch(&self, event: Event) {
        if matches!(event, Event::Snapshot(_) | Event::Unavailable) {
            self.last.replace(Some(event.clone()));
        }
        let listeners: Vec<Listener> = self
            .listeners
            .borrow()
            .iter()
            .map(|(_, f)| Rc::clone(f))
            .collect();
        for listener in listeners {
            listener(&event);
        }
    }

    /// Hear every event from now on, starting with the latest state. Returns a key for [`forget`].
    ///
    /// [`forget`]: Hub::forget
    pub fn listen(&self, listener: impl Fn(&Event) + 'static) -> u64 {
        let id = self.next.get();
        self.next.set(id + 1);
        let listener: Listener = Rc::new(listener);
        if let Some(options) = self.options.get() {
            listener(&Event::Options(options));
        }
        if let Some(last) = self.last.borrow().as_ref() {
            listener(last);
        }
        self.listeners.borrow_mut().push((id, listener));
        id
    }

    /// Stop calling a listener.
    pub fn forget(&self, id: u64) {
        self.listeners.borrow_mut().retain(|(key, _)| *key != id);
    }

    pub fn send(&self, command: Command) {
        self.backend.send(command);
    }

    /// The latest snapshot, if the server answered.
    pub fn snapshot(&self) -> Option<Snapshot> {
        match self.last.borrow().as_ref() {
            Some(Event::Snapshot(snapshot)) => Some((**snapshot).clone()),
            _ => None,
        }
    }

    /// Keep new options, tell every listener, and save them off the GTK thread.
    pub fn set_options(&self, options: Options) {
        self.options.set(Some(options));
        self.dispatch(Event::Options(options));
        if !self.preview {
            glib::spawn_future_local(async move {
                let _ = gio::spawn_blocking(move || crate::options::save(options)).await;
            });
        }
    }

    /// Count a view that wants live meters; the backend runs them while the count is above zero.
    pub fn want_meters(&self, on: bool) {
        let before = self.meters.get();
        let after = if on {
            before + 1
        } else {
            before.saturating_sub(1)
        };
        self.meters.set(after);
        if (before == 0) != (after == 0) {
            self.send(Command::Meters(after > 0));
        }
    }
}

fn forward(hub: &Rc<Hub>, events: async_channel::Receiver<Event>) {
    let weak = Rc::downgrade(hub);
    glib::spawn_future_local(async move {
        while let Ok(event) = events.recv().await {
            match weak.upgrade() {
                Some(hub) => hub.dispatch(event),
                None => return,
            }
        }
        if let Some(hub) = weak.upgrade() {
            hub.dispatch(Event::Unavailable);
        }
    });
}

fn load_options(hub: &Rc<Hub>) {
    let weak = Rc::downgrade(hub);
    let preview = hub.preview;
    glib::spawn_future_local(async move {
        let options = if preview {
            Options::default()
        } else {
            gio::spawn_blocking(crate::options::load)
                .await
                .unwrap_or_default()
        };
        if let Some(hub) = weak.upgrade() {
            hub.options.set(Some(options));
            hub.dispatch(Event::Options(options));
        }
    });
}
