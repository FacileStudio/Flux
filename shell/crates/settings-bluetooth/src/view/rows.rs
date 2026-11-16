//! One device row: the clickable head, and for paired devices a details panel
//! that slides open.

use std::cell::Cell;
use std::rc::{Rc, Weak};
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{glib, Align, Box, Button, Image, Label, Orientation, Revealer, Spinner, Switch};

use super::Page;
use crate::backend::Op;
use crate::i18n::Text;
use crate::model::{self, Device, Failure, Stage};
use crate::ui;

/// What the page knows about a row beyond the device itself.
pub struct State {
    pub busy: Option<Op>,
    pub failure: Option<Failure>,
    pub stage: Option<Stage>,
}

struct Details {
    revealer: Revealer,
    expand: Button,
    address: Label,
    kind: Label,
    trusted: Switch,
    forget: Button,
}

pub struct Row {
    pub root: Box,
    pub paired: bool,
    icon: Image,
    title: Label,
    subtitle: Label,
    battery: Box,
    battery_icon: Image,
    battery_label: Label,
    spinner: Spinner,
    details: Option<Details>,
    syncing: Rc<Cell<bool>>,
}

fn value_label() -> Label {
    let label = Label::new(None);
    label.add_css_class("fx-value");
    label.set_selectable(true);
    label.set_halign(Align::End);
    label
}

fn details(page: &Weak<Page>, path: &str, text: &Text, syncing: &Rc<Cell<bool>>) -> Details {
    let column = Box::new(Orientation::Vertical, 0);
    column.add_css_class("bt-details");
    let address = value_label();
    let kind = value_label();
    column.append(&ui::row(text.address, "", Some(address.upcast_ref())).0);
    column.append(&ui::row(text.kind, "", Some(kind.upcast_ref())).0);
    let (trust_row, trusted, _) = ui::switch_row(text.trusted, text.trusted_hint, false);
    column.append(&trust_row);
    let forget = Button::with_label(text.forget);
    forget.add_css_class("fx-button");
    forget.add_css_class("fx-danger");
    forget.set_halign(Align::Start);
    let actions = Box::new(Orientation::Horizontal, 8);
    actions.add_css_class("fx-row");
    actions.append(&forget);
    column.append(&actions);
    let revealer = Revealer::new();
    revealer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
    revealer.set_transition_duration(ui::motion(180));
    revealer.set_child(Some(&column));
    let expand = Button::from_icon_name("pan-down-symbolic");
    expand.add_css_class("bt-expand");
    expand.set_valign(Align::Center);
    expand.set_tooltip_text(Some(text.details));
    let shown = revealer.clone();
    expand.connect_clicked(move |button| {
        let open = !shown.reveals_child();
        shown.set_reveal_child(open);
        if open {
            button.add_css_class("bt-open");
        } else {
            button.remove_css_class("bt-open");
        }
    });
    let (weak, device) = (page.clone(), path.to_string());
    forget.connect_clicked(move |_| {
        if let Some(page) = weak.upgrade() {
            page.forget(&device);
        }
    });
    let (weak, device, guard) = (page.clone(), path.to_string(), Rc::clone(syncing));
    trusted.connect_active_notify(move |switch| {
        if guard.get() {
            return;
        }
        if let Some(page) = weak.upgrade() {
            page.trust(&device, switch.is_active());
        }
    });
    Details {
        revealer,
        expand,
        address,
        kind,
        trusted,
        forget,
    }
}

impl Row {
    pub fn new(page: &Rc<Page>, device: &Device, paired: bool) -> Row {
        let root = Box::new(Orientation::Vertical, 0);
        root.add_css_class("bt-device");
        if ui::animations() {
            root.add_css_class("bt-enter");
            let fading = root.downgrade();
            glib::timeout_add_local_once(Duration::from_millis(400), move || {
                if let Some(root) = fading.upgrade() {
                    root.remove_css_class("bt-enter");
                }
            });
        }
        let line = Box::new(Orientation::Horizontal, 4);
        let content = Box::new(Orientation::Horizontal, 12);
        let icon = Image::from_icon_name(device.kind().icon());
        icon.set_pixel_size(20);
        icon.add_css_class("bt-icon");
        let (texts, title, subtitle) = ui::texts(&device.display_name(), "");
        let battery = Box::new(Orientation::Horizontal, 4);
        battery.add_css_class("bt-battery");
        let battery_icon = Image::new();
        let battery_label = Label::new(None);
        battery.append(&battery_icon);
        battery.append(&battery_label);
        battery.set_valign(Align::Center);
        let spinner = Spinner::new();
        spinner.set_valign(Align::Center);
        content.append(&icon);
        content.append(&texts);
        content.append(&battery);
        content.append(&spinner);
        let head = Button::new();
        head.add_css_class("bt-head");
        head.set_child(Some(&content));
        head.set_hexpand(true);
        line.append(&head);
        let weak = Rc::downgrade(page);
        let syncing = Rc::new(Cell::new(false));
        let details = paired.then(|| details(&weak, &device.path, page.text, &syncing));
        if let Some(details) = &details {
            line.append(&details.expand);
            if page.starts_expanded(&device.path) {
                details.revealer.set_reveal_child(true);
                details.expand.add_css_class("bt-open");
            }
        }
        root.append(&line);
        if let Some(details) = &details {
            root.append(&details.revealer);
        }
        let path = device.path.clone();
        head.connect_clicked(move |_| {
            if let Some(page) = weak.upgrade() {
                page.activate(&path);
            }
        });
        Row {
            root,
            paired,
            icon,
            title,
            subtitle,
            battery,
            battery_icon,
            battery_label,
            spinner,
            details,
            syncing,
        }
    }

    /// Take the row out of its list.
    pub fn detach(&self) {
        if let Some(parent) = self.root.parent().and_downcast::<Box>() {
            parent.remove(&self.root);
        }
    }

    fn status(&self, text: &Text, device: &Device, state: &State) -> (String, bool) {
        if self.paired {
            let line = match (state.stage, state.busy, state.failure) {
                (Some(_), _, _) | (_, Some(Op::Connect), _) => text.connecting,
                (_, Some(Op::Disconnect), _) => text.disconnecting,
                (_, _, Some(_)) => text.connect_failed,
                _ if device.connected => text.connected,
                _ => text.not_connected,
            };
            return (line.to_string(), state.failure.is_some() && state.busy.is_none());
        }
        if state.stage.is_some() {
            return (text.pairing.to_string(), false);
        }
        let kind = text.kind(device.kind());
        let line = match device.rssi {
            Some(rssi) => format!("{kind} \u{b7} {}", text.signal(model::signal(rssi))),
            None => kind.to_string(),
        };
        (line, false)
    }

    pub fn update(&self, text: &Text, device: &Device, state: &State) {
        self.title.set_text(&device.display_name());
        self.icon.set_icon_name(Some(device.kind().icon()));
        let (line, failed) = self.status(text, device, state);
        self.subtitle.set_text(&line);
        self.subtitle.set_visible(true);
        if failed {
            self.subtitle.add_css_class("fx-danger");
        } else {
            self.subtitle.remove_css_class("fx-danger");
        }
        if device.connected {
            self.root.add_css_class("bt-connected");
        } else {
            self.root.remove_css_class("bt-connected");
        }
        let working = state.stage.is_some() || state.busy.is_some();
        self.spinner.set_visible(working);
        self.spinner.set_spinning(working);
        match device.battery.filter(|_| self.paired && device.connected) {
            Some(percent) => {
                self.battery_icon.set_icon_name(Some(&model::battery_icon(percent)));
                self.battery_label.set_text(&model::battery_label(percent, text.battery));
                self.battery.set_visible(true);
            }
            None => self.battery.set_visible(false),
        }
        if let Some(details) = &self.details {
            details.address.set_text(&device.address);
            details.kind.set_text(text.kind(device.kind()));
            self.syncing.set(true);
            details.trusted.set_active(device.trusted);
            self.syncing.set(false);
            details.forget.set_sensitive(state.busy != Some(Op::Forget));
        }
    }
}
