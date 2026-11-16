//! What opens under a row: the password field, the details, or a notice.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation, PasswordEntry, Widget};

use super::{sections, Page};
use crate::backend::{Command, Join};
use crate::i18n::Text;
use crate::model::wifi::{self, Failure, PasswordProblem};
use crate::model::{Link, Network, Security};
use crate::ui;

/// The words for a kind of security.
pub fn security_label(text: &Text, security: Security) -> &'static str {
    match security {
        Security::Open => text.open,
        Security::Owe => text.owe,
        Security::Wep => text.wep,
        Security::Personal => text.personal,
        Security::Wpa3 => text.wpa3,
        Security::Enterprise => text.enterprise,
    }
}

/// The sentence for a failed join.
pub fn failure_text(text: &Text, failure: Failure) -> &'static str {
    match failure {
        Failure::WrongPassword => text.failed_password,
        Failure::Timeout => text.failed_timeout,
        Failure::NotFound => text.failed_missing,
        Failure::NoAddress => text.failed_address,
        Failure::Other(_) => text.failed_other,
    }
}

/// The sentence for a password that cannot be sent.
pub fn problem_text(text: &Text, problem: PasswordProblem) -> &'static str {
    match problem {
        PasswordProblem::Empty => text.password_empty,
        PasswordProblem::TooShort => text.password_short,
        PasswordProblem::TooLong => text.password_long,
        PasswordProblem::NotAscii => text.password_ascii,
    }
}

/// The join request for a listed network.
pub fn join_for(network: &Network, password: Option<String>) -> Join {
    Join {
        ssid: network.ssid.clone(),
        ap_path: Some(network.ap_path.clone()),
        security: network.security,
        password,
        hidden: false,
    }
}

/// A primary or quiet button.
pub fn button(label: &str, class: &str) -> Button {
    let button = Button::with_label(label);
    button.add_css_class(class);
    button
}

/// The password field of a secured network, with its error line and buttons.
pub struct PasswordForm {
    pub root: Box,
    entry: PasswordEntry,
    error: Label,
    cancel: Button,
    connect: Button,
}

impl PasswordForm {
    pub fn new(page: &Rc<Page>, network: &Network) -> PasswordForm {
        let text = page.text;
        let root = Box::new(Orientation::Vertical, 8);
        root.add_css_class("fx-expand");
        let entry = PasswordEntry::new();
        entry.set_show_peek_icon(true);
        entry.set_placeholder_text(Some(text.password));
        entry.update_property(&[gtk4::accessible::Property::Label(text.password)]);
        root.append(&entry);
        let error = ui::label("", "fx-error");
        error.set_visible(false);
        root.append(&error);
        let actions = Box::new(Orientation::Horizontal, 8);
        actions.set_halign(Align::End);
        let cancel = button(text.cancel, "fx-ghost");
        let connect = button(text.connect, "fx-primary");
        actions.append(&cancel);
        actions.append(&connect);
        root.append(&actions);
        let form = PasswordForm {
            root,
            entry,
            error,
            cancel,
            connect,
        };
        form.wire(page, network);
        form
    }

    fn wire(&self, page: &Rc<Page>, network: &Network) {
        let submit = {
            let weak = Rc::downgrade(page);
            let entry = self.entry.clone();
            let error = self.error.clone();
            let network = network.clone();
            move || {
                let Some(page) = weak.upgrade() else {
                    return;
                };
                let password = entry.text().to_string();
                match wifi::check_password(network.security, &password) {
                    Ok(()) => page.join(join_for(&network, Some(password))),
                    Err(problem) => {
                        error.set_text(problem_text(page.text, problem));
                        error.set_visible(true);
                    }
                }
            }
        };
        let on_activate = submit.clone();
        self.entry.connect_activate(move |_| on_activate());
        self.connect.connect_clicked(move |_| submit());
        let error = self.error.clone();
        self.entry
            .connect_changed(move |_| error.set_visible(false));
        let weak = Rc::downgrade(page);
        self.cancel.connect_clicked(move |_| {
            if let Some(page) = weak.upgrade() {
                page.close();
            }
        });
    }

    /// Show the last failure and lock the field while a join runs.
    pub fn refresh(&self, page: &Page, failure: Option<Failure>, connecting: bool) {
        self.entry.set_sensitive(!connecting);
        self.connect.set_sensitive(!connecting);
        match failure {
            Some(failure) => {
                self.error.set_text(failure_text(page.text, failure));
                self.error.set_visible(true);
                if failure == Failure::WrongPassword {
                    self.entry.select_region(0, -1);
                    self.entry.grab_focus();
                }
            }
            None if connecting => self.error.set_visible(false),
            None => {}
        }
    }

    pub fn focus(&self) {
        self.entry.grab_focus();
    }
}

/// The honest notice for WEP and enterprise networks.
pub fn unsupported(page: &Page) -> Widget {
    let root = Box::new(Orientation::Vertical, 0);
    root.add_css_class("fx-expand");
    root.append(&ui::label(page.text.unsupported, "fx-subtitle"));
    root.upcast()
}

fn detail(grid: &gtk4::Grid, line: i32, name: &str, value: &str) {
    let key = ui::label(name, "fx-subtitle");
    key.set_wrap(false);
    let shown = ui::label(value, "fx-value");
    shown.set_selectable(true);
    shown.set_hexpand(true);
    grid.attach(&key, 0, line, 1, 1);
    grid.attach(&shown, 1, line, 1, 1);
}

/// Addressing, security, auto-connect, Disconnect and Forget.
pub fn details(page: &Rc<Page>, network: &Network) -> Widget {
    let text = page.text;
    let root = Box::new(Orientation::Vertical, 12);
    root.add_css_class("fx-expand");
    let grid = gtk4::Grid::new();
    grid.set_row_spacing(8);
    grid.set_column_spacing(24);
    let snapshot = page.snapshot.borrow();
    let mut lines: Vec<(&str, String)> = vec![(
        text.security,
        security_label(text, network.security).to_string(),
    )];
    let band = wifi::band(network.frequency);
    if !band.is_empty() {
        lines.push((text.band, band.to_string()));
    }
    let device = snapshot
        .wifi
        .as_ref()
        .filter(|device| network.active && device.link == Link::Connected);
    if let Some(device) = device {
        lines.extend(sections::ip_lines(text, &device.ip));
        lines.push((text.mac, device.mac.clone()));
    }
    for (line, (name, value)) in (0..).zip(lines.iter()) {
        detail(&grid, line, name, value);
    }
    root.append(&grid);
    let autoconnect = snapshot
        .saved_for(&network.ssid)
        .next()
        .map(|saved| saved.autoconnect);
    let connected = device.is_some();
    drop(snapshot);
    if let Some(active) = autoconnect {
        let (row, switch) = ui::switch_row(text.autoconnect, None, active);
        row.add_css_class("fx-inline");
        let weak = Rc::downgrade(page);
        let ssid = network.ssid.clone();
        switch.connect_state_set(move |_, on| {
            if let Some(page) = weak.upgrade() {
                page.backend.send(Command::SetAutoconnect(ssid.clone(), on));
            }
            gtk4::glib::Propagation::Proceed
        });
        root.append(&row);
    }
    root.append(&detail_actions(
        page,
        network,
        connected,
        autoconnect.is_some(),
    ));
    root.upcast()
}

fn detail_actions(page: &Rc<Page>, network: &Network, connected: bool, known: bool) -> Box {
    let actions = Box::new(Orientation::Horizontal, 8);
    actions.set_halign(Align::End);
    if known {
        let forget = button(page.text.forget, "fx-ghost-danger");
        let weak = Rc::downgrade(page);
        let ssid = network.ssid.clone();
        forget.connect_clicked(move |_| {
            if let Some(page) = weak.upgrade() {
                page.backend.send(Command::Forget(ssid.clone()));
                page.close();
            }
        });
        actions.append(&forget);
    }
    if connected {
        let disconnect = button(page.text.disconnect, "fx-secondary");
        let weak = Rc::downgrade(page);
        disconnect.connect_clicked(move |_| {
            if let Some(page) = weak.upgrade() {
                page.backend.send(Command::Disconnect);
                page.close();
            }
        });
        actions.append(&disconnect);
    }
    actions
}
