//! The Bluetooth page. GTK only: everything it shows comes from the client's
//! snapshot and events, everything it does goes out as a [`Command`].

pub mod quick;
mod rows;
mod sheet;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{glib, Align, Box, Button, Image, Label, Orientation, Overlay, Spinner, Stack, Switch};

use crate::backend::{Answer, Command, Event, Op, Request};
use crate::client::{self, Client};
use crate::i18n::{self, Text};
use crate::model::{self, Availability, Failure, Pairing, Prompt, Snapshot, Stage, Step};
use crate::ui;

/// Put the page's own stylesheet on the display once.
pub fn install_style() {
    ui::page_style(include_str!("../style.css"));
}

struct Empty {
    root: Box,
    image: Image,
    title: Label,
    body: Label,
    action: Button,
}

struct Lists {
    paired: Box,
    paired_empty: Label,
    nearby: Box,
    nearby_empty: Label,
    searching: Spinner,
    unnamed_row: Box,
    unnamed: Switch,
    unnamed_count: Label,
}

/// The page and what it remembers between snapshots.
pub struct Page {
    client: Rc<Client>,
    text: &'static Text,
    power_card: Box,
    power: Switch,
    visible_label: Label,
    stack: Stack,
    empty: Empty,
    lists: Lists,
    sheet: sheet::Sheet,
    rows: RefCell<HashMap<String, rows::Row>>,
    busy: RefCell<HashMap<String, Op>>,
    errors: RefCell<HashMap<String, Failure>>,
    pairing: RefCell<Pairing>,
    request: RefCell<Option<Request>>,
    shown: Cell<bool>,
    scanning: Cell<bool>,
    syncing: Cell<bool>,
}

fn empty_state() -> Empty {
    let (root, image, title, body) = ui::empty_state_with("bluetooth-disabled-symbolic", "", "");
    let action = ui::primary_button("");
    action.set_halign(Align::Center);
    action.set_margin_top(8);
    root.append(&action);
    Empty {
        root,
        image,
        title,
        body,
        action,
    }
}

fn lists(text: &Text, column: &Box) -> Lists {
    let (paired_section, paired_card) = ui::section(text.my_devices);
    let paired = Box::new(Orientation::Vertical, 0);
    let paired_empty = ui::label(text.no_paired, "fx-subtitle");
    paired_empty.add_css_class("fx-row");
    paired_card.append(&paired);
    paired_card.append(&paired_empty);
    let nearby_parts = ui::section_with(text.nearby, "");
    let (nearby_section, nearby_header, nearby_card) = (nearby_parts.root, nearby_parts.header, nearby_parts.card);
    let searching = Spinner::new();
    searching.set_tooltip_text(Some(text.searching));
    nearby_header.append(&searching);
    let nearby = Box::new(Orientation::Vertical, 0);
    let nearby_empty = ui::label(text.no_nearby, "fx-subtitle");
    nearby_empty.add_css_class("fx-row");
    let unnamed = Switch::new();
    unnamed.update_property(&[gtk4::accessible::Property::Label(text.show_unnamed)]);
    let unnamed_parts = ui::row_with(text.show_unnamed, "", Some(unnamed.upcast_ref()));
    let (unnamed_row, unnamed_count) = (unnamed_parts.root, unnamed_parts.subtitle);
    nearby_card.append(&nearby);
    nearby_card.append(&nearby_empty);
    nearby_card.append(&unnamed_row);
    column.append(&paired_section);
    column.append(&nearby_section);
    Lists {
        paired,
        paired_empty,
        nearby,
        nearby_empty,
        searching,
        unnamed_row,
        unnamed,
        unnamed_count,
    }
}

/// The whole page, connected to the process's client.
pub fn page() -> gtk4::Widget {
    install_style();
    let (page, root) = Page::new(client::client(), i18n::text());
    let keep = RefCell::new(Some(page));
    root.connect_destroy(move |_| {
        keep.borrow_mut().take();
    });
    root.upcast()
}

impl Page {
    fn new(client: Rc<Client>, text: &'static Text) -> (Rc<Page>, Overlay) {
        let (scroll, column) = ui::page(text.title, text.description);
        column.add_css_class("bt-page");
        let (power_card, power_rows) = ui::section("");
        let power = Switch::new();
        power.update_property(&[gtk4::accessible::Property::Label(text.title)]);
        let power_parts = ui::row_with(text.title, "", Some(power.upcast_ref()));
        let visible_label = power_parts.subtitle.clone();
        power_rows.append(&power_parts.root);
        column.append(&power_card);
        let empty = empty_state();
        let devices = Box::new(Orientation::Vertical, 32);
        let lists = lists(text, &devices);
        let stack = Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        stack.set_transition_duration(ui::MOTION_MS);
        stack.set_vhomogeneous(false);
        stack.set_interpolate_size(true);
        stack.add_named(&empty.root, Some("empty"));
        stack.add_named(&devices, Some("devices"));
        column.append(&stack);
        let root = Overlay::new();
        root.add_css_class("bt-page");
        root.set_child(Some(&scroll));
        let sheet = sheet::Sheet::new();
        root.add_overlay(&sheet.root);
        let page = Rc::new(Page {
            client,
            text,
            power_card,
            power,
            visible_label,
            stack,
            empty,
            lists,
            sheet,
            rows: RefCell::new(HashMap::new()),
            busy: RefCell::new(HashMap::new()),
            errors: RefCell::new(HashMap::new()),
            pairing: RefCell::new(Pairing::Idle),
            request: RefCell::new(None),
            shown: Cell::new(false),
            scanning: Cell::new(false),
            syncing: Cell::new(false),
        });
        page.connect(&root);
        page.render();
        (page, root)
    }

    fn connect(self: &Rc<Self>, root: &Overlay) {
        let weak = Rc::downgrade(self);
        self.client.subscribe(root, true, move |event| {
            if let Some(page) = weak.upgrade() {
                page.handle(event);
            }
        });
        let weak = Rc::downgrade(self);
        self.power.connect_state_set(move |_, on| {
            let Some(page) = weak.upgrade() else {
                return glib::Propagation::Proceed;
            };
            if page.syncing.get() {
                return glib::Propagation::Proceed;
            }
            page.client.send(Command::SetPowered(on));
            glib::Propagation::Stop
        });
        let weak = Rc::downgrade(self);
        self.empty.action.connect_clicked(move |_| {
            if let Some(page) = weak.upgrade() {
                page.client.send(Command::SetPowered(true));
            }
        });
        let weak = Rc::downgrade(self);
        self.lists.unnamed.connect_active_notify(move |_| {
            if let Some(page) = weak.upgrade() {
                page.render();
            }
        });
        let weak = Rc::downgrade(self);
        root.connect_map(move |_| {
            if let Some(page) = weak.upgrade() {
                page.shown.set(true);
                page.sync_scan(&page.client.snapshot());
            }
        });
        let weak = Rc::downgrade(self);
        root.connect_unmap(move |_| {
            if let Some(page) = weak.upgrade() {
                page.shown.set(false);
                page.sync_scan(&page.client.snapshot());
            }
        });
        self.sheet.connect(Rc::downgrade(self));
    }

    fn weak(self: &Rc<Self>) -> Weak<Page> {
        Rc::downgrade(self)
    }

    fn handle(self: &Rc<Self>, event: &Event) {
        match event {
            Event::Snapshot(_) => self.render(),
            Event::Request(request) => {
                let device = request.device.clone();
                self.step(Step::Prompt(device.clone(), request.prompt.clone()));
                let asked = matches!(&*self.pairing.borrow(), Pairing::Asking { device: current, .. } if *current == device);
                if asked {
                    *self.request.borrow_mut() = Some(request.clone());
                    self.render_sheet();
                } else if let Some(reply) = &request.reply {
                    let _ = reply.try_send(Answer::Reject);
                }
            }
            Event::RequestCancelled => self.step(Step::Cancelled),
            Event::Pairing(step) => {
                if let Step::Start(device) = step {
                    self.errors.borrow_mut().remove(device);
                }
                self.step(step.clone());
            }
            Event::Done { device, op, failure } => {
                self.busy.borrow_mut().remove(device);
                if let (Some(failure), Op::Connect | Op::Disconnect) = (failure, op) {
                    self.flag_error(device, *failure);
                }
                self.render();
            }
        }
    }

    fn flag_error(self: &Rc<Self>, device: &str, failure: Failure) {
        self.errors.borrow_mut().insert(device.to_string(), failure);
        let weak = self.weak();
        let device = device.to_string();
        glib::timeout_add_local_once(Duration::from_secs(6), move || {
            if let Some(page) = weak.upgrade() {
                page.errors.borrow_mut().remove(&device);
                page.render();
            }
        });
    }

    fn step(self: &Rc<Self>, step: Step) {
        let next = self.pairing.take().next(step);
        if !matches!(next, Pairing::Asking { .. }) {
            self.request.borrow_mut().take();
        }
        *self.pairing.borrow_mut() = next;
        self.render();
    }

    fn sync_scan(&self, snapshot: &Snapshot) {
        let powered = snapshot.availability() == Availability::On;
        let wanted = self.shown.get() && powered;
        if wanted == self.scanning.get() {
            return;
        }
        self.scanning.set(wanted);
        if powered {
            self.client.send(Command::Discover(wanted));
            self.client.send(Command::SetVisible(wanted));
        }
    }

    fn render(self: &Rc<Self>) {
        let snapshot = self.client.snapshot();
        let availability = snapshot.availability();
        self.render_power(&snapshot, availability);
        self.render_empty(availability);
        if availability == Availability::On {
            self.render_lists(&snapshot);
        }
        self.sync_scan(&snapshot);
        self.render_sheet();
    }

    fn render_power(&self, snapshot: &Snapshot, availability: Availability) {
        let powered = matches!(availability, Availability::On);
        self.power_card
            .set_visible(!matches!(availability, Availability::NoService | Availability::NoAdapter));
        self.syncing.set(true);
        self.power.set_active(powered);
        self.power.set_state(powered);
        self.syncing.set(false);
        self.power.set_sensitive(availability != Availability::HardBlocked);
        let adapter = snapshot.adapter.as_ref();
        let visible = powered && adapter.is_some_and(|adapter| adapter.discoverable && !adapter.name.is_empty());
        let line = match adapter {
            Some(adapter) if visible => i18n::named(self.text.visible_as, &adapter.name),
            _ => String::new(),
        };
        ui::set_subtitle(&self.visible_label, &line);
    }

    fn render_empty(&self, availability: Availability) {
        let text = self.text;
        let (icon, title, body, action) = match availability {
            Availability::On => {
                self.stack.set_visible_child_name("devices");
                return;
            }
            Availability::NoService => ("bluetooth-disabled-symbolic", text.no_service_title, text.no_service_body, false),
            Availability::NoAdapter => ("bluetooth-disabled-symbolic", text.no_adapter_title, text.no_adapter_body, false),
            Availability::HardBlocked => ("airplane-mode-symbolic", text.hard_title, text.hard_body, false),
            Availability::SoftBlocked => ("airplane-mode-symbolic", text.soft_title, text.soft_body, true),
            Availability::Off => ("bluetooth-disabled-symbolic", text.off_title, text.off_body, true),
        };
        self.empty.image.set_icon_name(Some(icon));
        self.empty.title.set_text(title);
        ui::set_subtitle(&self.empty.body, body);
        self.empty.action.set_label(text.turn_on);
        self.empty.action.set_visible(action);
        self.stack.set_visible_child_name("empty");
    }

    fn row_state(&self, path: &str) -> rows::State {
        let stage = match &*self.pairing.borrow() {
            Pairing::Working { device, stage } if device == path => Some(*stage),
            Pairing::Asking { device, .. } if device == path => Some(Stage::Pairing),
            _ => None,
        };
        rows::State {
            busy: self.busy.borrow().get(path).copied(),
            failure: self.errors.borrow().get(path).copied(),
            stage,
        }
    }

    fn render_lists(self: &Rc<Self>, snapshot: &Snapshot) {
        let show_unnamed = self.lists.unnamed.is_active();
        let paired = snapshot.paired();
        let nearby = snapshot.nearby(show_unnamed);
        let mut seen = HashSet::new();
        for (list, devices, is_paired) in [(&self.lists.paired, &paired, true), (&self.lists.nearby, &nearby, false)] {
            let mut previous: Option<gtk4::Widget> = None;
            for device in devices.iter() {
                seen.insert(device.path.clone());
                let state = self.row_state(&device.path);
                let mut rows = self.rows.borrow_mut();
                let stale = rows.get(&device.path).is_some_and(|row| row.paired != is_paired);
                if stale {
                    if let Some(row) = rows.remove(&device.path) {
                        row.detach();
                    }
                }
                let row = rows
                    .entry(device.path.clone())
                    .or_insert_with(|| rows::Row::new(self, device, is_paired));
                if row.root.parent().is_none() {
                    list.append(&row.root);
                }
                list.reorder_child_after(&row.root, previous.as_ref());
                row.update(self.text, device, &state);
                previous = Some(row.root.clone().upcast());
            }
        }
        self.rows.borrow_mut().retain(|path, row| {
            let keep = seen.contains(path);
            if !keep {
                row.detach();
            }
            keep
        });
        self.lists.paired_empty.set_visible(paired.is_empty());
        self.lists.nearby_empty.set_visible(nearby.is_empty());
        let discovering = snapshot.adapter.as_ref().is_some_and(|adapter| adapter.discovering);
        self.lists.searching.set_visible(discovering);
        self.lists.searching.set_spinning(discovering);
        let unnamed = snapshot.unnamed_count();
        self.lists.unnamed_row.set_visible(unnamed > 0 || show_unnamed);
        self.lists
            .unnamed_count
            .set_text(&self.text.unnamed_count.replace("{n}", &unnamed.to_string()));
        self.lists.unnamed_count.set_visible(unnamed > 0);
    }

    fn device_name(&self, path: &str) -> String {
        self.client
            .snapshot()
            .device(path)
            .map(|device| device.display_name())
            .unwrap_or_else(|| path.rsplit('/').next().unwrap_or(path).to_string())
    }

    fn device_icon(&self, path: &str) -> &'static str {
        self.client
            .snapshot()
            .device(path)
            .map(|device| device.kind().icon())
            .unwrap_or(model::Kind::Other.icon())
    }

    fn render_sheet(&self) {
        let pairing = self.pairing.borrow().clone();
        let content = match &pairing {
            Pairing::Asking { device, prompt, .. } => sheet::Content::prompt(self.text, &self.device_name(device), prompt),
            Pairing::Failed { device, failure } => sheet::Content::failed(self.text, &self.device_name(device), *failure),
            _ => {
                self.sheet.hide();
                return;
            }
        };
        let device = pairing.device().unwrap_or_default();
        self.sheet.show(self.device_icon(device), &content);
    }

    /// Click on a device row: connect or disconnect a paired device, pair a
    /// nearby one, or cancel the pairing in progress.
    pub fn activate(self: &Rc<Self>, path: &str) {
        let snapshot = self.client.snapshot();
        let Some(device) = snapshot.device(path) else {
            return;
        };
        if self.busy.borrow().contains_key(path) {
            return;
        }
        let pairing = self.pairing.borrow().clone();
        if pairing.device() == Some(path) && !matches!(pairing, Pairing::Failed { .. }) {
            self.client.send(Command::CancelPairing(path.to_string()));
            return;
        }
        if !device.paired {
            if matches!(pairing, Pairing::Idle | Pairing::Failed { .. }) {
                self.client.send(Command::Pair(path.to_string()));
            }
            return;
        }
        self.errors.borrow_mut().remove(path);
        let (op, command) = if device.connected {
            (Op::Disconnect, Command::Disconnect(path.to_string()))
        } else {
            (Op::Connect, Command::Connect(path.to_string()))
        };
        self.busy.borrow_mut().insert(path.to_string(), op);
        self.client.send(command);
        self.render();
    }

    /// Forget a paired device.
    pub fn forget(self: &Rc<Self>, path: &str) {
        self.busy.borrow_mut().insert(path.to_string(), Op::Forget);
        self.client.send(Command::Forget(path.to_string()));
        self.render();
    }

    /// Trust or distrust a paired device.
    pub fn trust(self: &Rc<Self>, path: &str, on: bool) {
        self.client.send(Command::SetTrusted(path.to_string(), on));
    }

    /// Whether the preview asked for `path` to open expanded.
    pub fn starts_expanded(&self, path: &str) -> bool {
        self.client.expanded() == Some(path)
    }

    /// The sheet's main button.
    pub fn confirm(self: &Rc<Self>) {
        let pairing = self.pairing.borrow().clone();
        match pairing {
            Pairing::Asking { prompt, .. } => {
                let answer = match prompt {
                    Prompt::RequestPin => Answer::Pin(self.sheet.entry_text()),
                    Prompt::RequestPasskey => match model::parse_passkey(&self.sheet.entry_text()) {
                        Some(passkey) => Answer::Passkey(passkey),
                        None => return,
                    },
                    _ => Answer::Accept,
                };
                if let Some(reply) = self.request.borrow().as_ref().and_then(|request| request.reply.clone()) {
                    let _ = reply.try_send(answer);
                }
                self.step(Step::Answered);
            }
            Pairing::Failed { device, .. } => {
                self.step(Step::Dismiss);
                self.client.send(Command::Pair(device));
            }
            _ => {}
        }
    }

    /// The sheet's cancel button.
    pub fn cancel(self: &Rc<Self>) {
        let pairing = self.pairing.borrow().clone();
        if let Pairing::Asking { device, .. } = &pairing {
            match self.request.borrow().as_ref().and_then(|request| request.reply.clone()) {
                Some(reply) => {
                    let _ = reply.try_send(Answer::Reject);
                }
                None => self.client.send(Command::CancelPairing(device.clone())),
            }
        }
        self.step(Step::Dismiss);
    }
}
