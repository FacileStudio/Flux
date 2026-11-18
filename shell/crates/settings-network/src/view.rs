//! The GTK side: the page, the quick menu, and how events redraw them.
//!
//! One [`Page`] holds the last snapshot plus what the person is doing (which
//! row is open, which network is joining, the last failure per network) and
//! redraws from both in [`Page::render`]. Rows are kept by SSID and updated in
//! place, so a new scan result reorders the list without rebuilding it.

mod draw;
mod expand;
mod list;
mod preview;
mod row;
mod sections;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{
    glib, Align, Box, Button, Label, Orientation, Revealer, Spinner, Stack, Switch, Widget,
};

use crate::backend::{Backend, Command, Event, Join};
use crate::i18n::{self, Text};
use crate::model::wifi::Failure;
use crate::model::{Network, Snapshot, Status};
use crate::ui;

/// How much of the page to draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Full,
    Quick,
}

/// What an open row shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expand {
    Password,
    Details,
    Unsupported,
}

/// The page's state and the widgets that are redrawn from it.
pub struct Page {
    pub backend: Backend,
    pub text: &'static Text,
    pub mode: Mode,
    pub snapshot: RefCell<Snapshot>,
    pub joining: RefCell<Option<String>>,
    pub failures: RefCell<HashMap<String, Failure>>,
    pub open: RefCell<Option<(String, Expand)>>,
    updating: Cell<bool>,
    root: Box,
    outer: Widget,
    state: Stack,
    message: Box,
    wifi_switch: Switch,
    wifi_row: Box,
    airplane: Option<Switch>,
    scan_spinner: Spinner,
    scan_button: Button,
    scan_label: Label,
    networks: Stack,
    list: list::List,
    empty: Box,
    other: Option<sections::Other>,
    extra: Box,
    banner: Revealer,
    banner_label: Label,
}

impl Page {
    /// Build the page, start its backend (or the preview data), and return it.
    pub fn new(mode: Mode) -> Rc<Page> {
        let preview = std::env::var("FLUX_NETWORK_PREVIEW").ok();
        let text = i18n::text();
        let (backend, events) = match preview {
            Some(_) => (Backend::detached(), None),
            None => {
                let (backend, events) = Backend::start();
                (backend, Some(events))
            }
        };
        let page = Rc::new(Page::build(mode, text, backend));
        page.connect();
        match (events, preview) {
            (Some(events), _) => page.listen(events),
            (None, Some(name)) => preview::apply(&page, &name),
            (None, None) => {}
        }
        page.render();
        page
    }

    fn build(mode: Mode, text: &'static Text, backend: Backend) -> Page {
        let (root, outer): (Box, Widget) = if mode == Mode::Full {
            let (scroll, column) = ui::page(text.title, text.description);
            (column, scroll.upcast())
        } else {
            let column = Box::new(Orientation::Vertical, 8);
            column.add_css_class("fx-quick");
            (column.clone(), column.upcast())
        };
        root.add_css_class("fx-network");
        let (banner, banner_label) = banner();
        root.append(&banner);
        let state = Stack::new();
        state.set_transition_type(gtk4::StackTransitionType::Crossfade);
        state.set_transition_duration(ui::MOTION_MS);
        state.set_vhomogeneous(false);
        let message = Box::new(Orientation::Vertical, 0);
        state.add_named(&message, Some("message"));
        let content = Box::new(
            Orientation::Vertical,
            if mode == Mode::Full { 32 } else { 4 },
        );
        state.add_named(&content, Some("content"));
        root.append(&state);
        let (radios_root, radios) = ui::section("");
        let (wifi_row, wifi_switch) = ui::switch_row(text.wifi, "", false, |_| {});
        radios.append(&wifi_row);
        let airplane = (mode == Mode::Full).then(|| {
            let (row, switch) = ui::switch_row(text.airplane, text.airplane_body, false, |_| {});
            radios.append(&row);
            switch
        });
        content.append(&radios_root);
        let parts = ui::section_with(if mode == Mode::Full { text.networks } else { "" }, "");
        let section = parts.card.clone();
        let (scan_spinner, scan_button, scan_label) = scan_header(text, &parts.header);
        parts.header.set_visible(true);
        let networks = Stack::new();
        networks.set_vhomogeneous(false);
        networks.set_transition_type(gtk4::StackTransitionType::Crossfade);
        networks.set_transition_duration(ui::MOTION_MS);
        let list = list::List::new();
        networks.add_named(&list.container, Some("list"));
        let empty = Box::new(Orientation::Vertical, 0);
        networks.add_named(&empty, Some("empty"));
        section.append(&networks);
        let other = (mode == Mode::Full).then(sections::Other::new);
        if let Some(other) = &other {
            section.append(&other.root);
        }
        content.append(&parts.root);
        let extra = Box::new(Orientation::Vertical, 32);
        content.append(&extra);
        Page {
            backend,
            text,
            mode,
            snapshot: RefCell::new(Snapshot::empty(Status::Loading)),
            joining: RefCell::new(None),
            failures: RefCell::new(HashMap::new()),
            open: RefCell::new(None),
            updating: Cell::new(false),
            root,
            outer,
            state,
            message,
            wifi_switch,
            wifi_row,
            airplane,
            scan_spinner,
            scan_button,
            scan_label,
            networks,
            list,
            empty,
            other,
            extra,
            banner,
            banner_label,
        }
    }

    /// The widget to put in a window or a popover.
    pub fn widget(&self) -> Widget {
        self.outer.clone()
    }

    fn connect(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.wifi_switch.connect_state_set(move |_, on| {
            if let Some(page) = weak.upgrade().filter(|page| !page.updating.get()) {
                page.backend.send(Command::SetWifi(on));
            }
            glib::Propagation::Proceed
        });
        if let Some(airplane) = &self.airplane {
            let weak = Rc::downgrade(self);
            airplane.connect_state_set(move |_, on| {
                if let Some(page) = weak.upgrade().filter(|page| !page.updating.get()) {
                    page.backend.send(Command::SetAirplane(on));
                }
                glib::Propagation::Proceed
            });
        }
        let weak = Rc::downgrade(self);
        self.scan_button.connect_clicked(move |_| {
            if let Some(page) = weak.upgrade() {
                page.backend.send(Command::Scan);
            }
        });
        if let Some(other) = &self.other {
            other.connect(self);
        }
        if self.mode == Mode::Quick {
            sections::settings_link(self);
        }
    }

    fn listen(self: &Rc<Self>, events: async_channel::Receiver<Event>) {
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            while let Ok(event) = events.recv().await {
                let Some(page) = weak.upgrade() else {
                    return;
                };
                page.apply(event);
            }
        });
    }

    /// Fold one backend event into the page.
    pub fn apply(self: &Rc<Self>, event: Event) {
        match event {
            Event::Snapshot(snapshot) => *self.snapshot.borrow_mut() = *snapshot,
            Event::Connecting(ssid) => {
                self.failures.borrow_mut().remove(&ssid);
                *self.joining.borrow_mut() = Some(ssid);
            }
            Event::Connected(ssid) => {
                self.joining.borrow_mut().take();
                let open = self
                    .open
                    .borrow()
                    .as_ref()
                    .is_some_and(|(name, _)| *name == ssid);
                if open {
                    self.open.borrow_mut().take();
                }
                if let Some(other) = &self.other {
                    other.outcome(self, &ssid, None);
                }
            }
            Event::Failed(ssid, failure) => {
                self.joining.borrow_mut().take();
                if let Some(other) = &self.other {
                    other.outcome(self, &ssid, Some(failure));
                }
                let secured = self
                    .network(&ssid)
                    .is_none_or(|network| network.security.secured());
                if failure == Failure::WrongPassword || secured {
                    *self.open.borrow_mut() = Some((ssid.clone(), Expand::Password));
                }
                self.failures.borrow_mut().insert(ssid, failure);
            }
            Event::ActionFailed(message) => self.warn(&message),
        }
        self.render();
    }

    /// The network named `ssid` in the last snapshot.
    pub fn network(&self, ssid: &str) -> Option<Network> {
        self.snapshot
            .borrow()
            .networks
            .iter()
            .find(|network| network.ssid == ssid)
            .cloned()
    }

    /// Join `join`, remembering it as the network in progress.
    pub fn join(self: &Rc<Self>, join: Join) {
        self.failures.borrow_mut().remove(&join.ssid);
        *self.joining.borrow_mut() = Some(join.ssid.clone());
        self.backend.send(Command::Connect(join));
        self.render();
    }

    /// Open `ssid`'s row on `expand`, or close it when it already shows that.
    pub fn toggle(self: &Rc<Self>, ssid: &str, expand: Expand) {
        let same = self
            .open
            .borrow()
            .as_ref()
            .is_some_and(|(name, open)| name == ssid && *open == expand);
        *self.open.borrow_mut() = (!same).then(|| (ssid.to_string(), expand));
        self.render();
    }

    /// Close whichever row is open.
    pub fn close(self: &Rc<Self>) {
        self.open.borrow_mut().take();
        self.render();
    }

    fn warn(self: &Rc<Self>, message: &str) {
        self.banner_label
            .set_text(&format!("{}: {message}", self.text.action_failed));
        ui::show(&self.banner, true);
        let banner = self.banner.clone();
        glib::timeout_add_local_once(Duration::from_secs(6), move || {
            ui::show(&banner, false)
        });
    }

    /// Redraw everything from the snapshot and the page state.
    pub fn render(self: &Rc<Self>) {
        self.updating.set(true);
        let snapshot = self.snapshot.borrow().clone();
        match &snapshot.status {
            Status::Ready => self.state.set_visible_child_name("content"),
            other => {
                self.show_message(other);
                self.state.set_visible_child_name("message");
            }
        }
        self.render_radios(&snapshot);
        self.render_networks(&snapshot);
        if self.mode == Mode::Full {
            sections::render_extra(self, &self.extra, &snapshot);
        }
        self.updating.set(false);
    }

    fn show_message(&self, status: &Status) {
        clear(&self.message);
        let text = self.text;
        let widget: Widget = match status {
            Status::Loading => ui::spinner_row(text.loading).upcast(),
            Status::NotRunning => ui::empty_state(EMPTY_ICON, text.missing_title, text.missing_body).upcast(),
            Status::Error(error) => ui::empty_state(EMPTY_ICON, text.error_title, error).upcast(),
            Status::Ready => return,
        };
        let (section, card) = ui::section("");
        card.append(&widget);
        self.message.append(&section);
    }

    fn render_radios(&self, snapshot: &Snapshot) {
        let has_wifi = snapshot.wifi.is_some();
        self.wifi_row.set_visible(has_wifi);
        self.wifi_switch.set_active(snapshot.wifi_enabled);
        self.wifi_switch.set_state(snapshot.wifi_enabled);
        self.wifi_switch.set_sensitive(snapshot.wifi_hardware);
        if let Some(airplane) = &self.airplane {
            airplane.set_active(snapshot.airplane());
            airplane.set_state(snapshot.airplane());
        }
        let subtitle = sections::wifi_subtitle(self.text, snapshot);
        sections::set_subtitle(&self.wifi_row, subtitle.as_deref());
    }

    fn render_networks(self: &Rc<Self>, snapshot: &Snapshot) {
        let usable = snapshot.wifi.is_some() && snapshot.wifi_enabled;
        self.scan_button.set_visible(usable && !snapshot.scanning);
        self.scan_spinner.set_visible(usable && snapshot.scanning);
        self.scan_spinner.set_spinning(usable && snapshot.scanning);
        self.scan_label.set_visible(usable && snapshot.scanning);
        if let Some(other) = &self.other {
            other.root.set_visible(usable);
        }
        let empty = sections::wifi_empty(self.text, snapshot);
        let limit = if self.mode == Mode::Quick {
            5
        } else {
            usize::MAX
        };
        let shown: Vec<Network> = if usable {
            snapshot.networks.iter().take(limit).cloned().collect()
        } else {
            Vec::new()
        };
        self.list.update(self, &shown);
        match empty {
            Some((title, body)) => {
                clear(&self.empty);
                self.empty.append(&ui::empty_state(EMPTY_ICON, title, body));
                self.networks.set_visible_child_name("empty");
            }
            None => self.networks.set_visible_child_name("list"),
        }
    }
}

const EMPTY_ICON: &str = "network-wireless-offline-symbolic";

fn banner() -> (Revealer, Label) {
    let label = ui::label("", "fx-banner");
    label.add_css_class("fx-danger");
    (ui::reveal(&label), label)
}

fn scan_header(text: &Text, header: &Box) -> (Spinner, Button, Label) {
    let spinner = Spinner::new();
    spinner.set_valign(Align::Center);
    header.append(&spinner);
    let scanning = ui::label(text.scanning, "fx-subtitle");
    scanning.set_valign(Align::Center);
    scanning.set_hexpand(false);
    header.append(&scanning);
    let button = ui::secondary_button(text.scan);
    button.set_valign(Align::Center);
    header.append(&button);
    (spinner, button, scanning)
}

pub use ui::clear;
