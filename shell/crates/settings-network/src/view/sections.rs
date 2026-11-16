//! The parts around the Wi-Fi list: hidden network, wired, VPN, quick menu link.

use std::cell::Cell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    glib, Box, Button, DrawingArea, DropDown, Entry, GestureClick, Label, Orientation,
    PasswordEntry, Revealer,
};

use super::expand::{button, failure_text, problem_text};
use super::{clear, draw, Page};
use crate::backend::{Command, Join};
use crate::i18n::Text;
use crate::model::wifi::{self, Failure};
use crate::model::{IpInfo, Link, Network, Security, Snapshot};
use crate::ui;

const HIDDEN_KINDS: [Security; 3] = [Security::Personal, Security::Wpa3, Security::Open];

/// The muted line under the Wi-Fi switch.
pub fn wifi_subtitle(text: &Text, snapshot: &Snapshot) -> Option<String> {
    let device = snapshot.wifi.as_ref()?;
    if !snapshot.wifi_hardware {
        return Some(text.hardware_off.to_string());
    }
    if !snapshot.wifi_enabled {
        return None;
    }
    let active = snapshot.networks.iter().find(|network| network.active);
    match (device.link, active) {
        (Link::Connected, Some(network)) => Some(network.ssid.clone()),
        (Link::Connecting, _) => Some(text.connecting.to_string()),
        _ => Some(text.disconnected.to_string()),
    }
}

/// Set or clear the muted line of a `ui::row`.
pub fn set_subtitle(row: &Box, subtitle: Option<&str>) {
    let Some(column) = row.first_child().and_downcast::<Box>() else {
        return;
    };
    let existing = column
        .first_child()
        .and_then(|title| title.next_sibling())
        .and_downcast::<Label>();
    match (existing, subtitle) {
        (Some(label), Some(text)) => label.set_text(text),
        (Some(label), None) => column.remove(&label),
        (None, Some(text)) => column.append(&ui::label(text, "fx-subtitle")),
        (None, None) => {}
    }
}

/// The empty state in place of the list, when there is one.
pub fn wifi_empty(
    text: &'static Text,
    snapshot: &Snapshot,
) -> Option<(&'static str, &'static str)> {
    if snapshot.wifi.is_none() {
        return Some((text.no_adapter_title, text.no_adapter_body));
    }
    if snapshot.airplane() {
        return Some((text.airplane_title, text.airplane_hint));
    }
    if !snapshot.wifi_hardware {
        return Some((text.off_title, text.hardware_off));
    }
    if !snapshot.wifi_enabled {
        return Some((text.off_title, text.off_body));
    }
    (snapshot.networks.is_empty() && !snapshot.scanning).then_some((text.no_networks, ""))
}

/// Address, gateway and DNS, for the lines that have a value.
pub fn ip_lines(text: &'static Text, ip: &IpInfo) -> Vec<(&'static str, String)> {
    let mut lines = Vec::new();
    if let Some(address) = &ip.address {
        lines.push((text.ip, address.clone()));
    }
    if let Some(gateway) = &ip.gateway {
        lines.push((text.gateway, gateway.clone()));
    }
    if !ip.dns.is_empty() {
        lines.push((text.dns, ip.dns.join(", ")));
    }
    lines
}

/// What the details of `network` depend on, to redraw them only when it moves.
pub fn details_key(page: &Page, network: &Network) -> String {
    let snapshot = page.snapshot.borrow();
    let device = snapshot
        .wifi
        .as_ref()
        .map(|device| (device.link, device.ip.clone(), device.mac.clone()));
    let saved: Vec<bool> = snapshot
        .saved_for(&network.ssid)
        .map(|saved| saved.autoconnect)
        .collect();
    format!(
        "{}{:?}{device:?}{saved:?}",
        network.active, network.security
    )
}

/// The "Other network" row: name, security and password of a hidden network.
pub struct Other {
    pub root: Box,
    header: Box,
    open: Rc<Cell<bool>>,
    chevron: DrawingArea,
    body: Revealer,
    ssid: Entry,
    kind: DropDown,
    password: PasswordEntry,
    error: Label,
    cancel: Button,
    connect: Button,
}

impl Other {
    pub fn new() -> Other {
        let text = crate::i18n::text();
        let root = Box::new(Orientation::Vertical, 0);
        root.add_css_class("fx-net");
        let open = Rc::new(Cell::new(false));
        let chevron = draw::chevron(open.clone());
        let header = ui::row(
            text.other,
            Some(text.other_body),
            Some(chevron.upcast_ref()),
        );
        header.add_css_class("fx-net-header");
        let indent = Box::new(Orientation::Horizontal, 0);
        indent.set_size_request(18, -1);
        header.prepend(&indent);
        root.append(&header);
        let body = Revealer::new();
        body.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
        let form = Box::new(Orientation::Vertical, 8);
        form.add_css_class("fx-expand");
        let ssid = Entry::new();
        ssid.set_placeholder_text(Some(text.network_name));
        form.append(&ssid);
        let labels: Vec<&str> = HIDDEN_KINDS
            .iter()
            .map(|kind| super::expand::security_label(text, *kind))
            .collect();
        let kind = DropDown::from_strings(&labels);
        form.append(&kind);
        let password = PasswordEntry::new();
        password.set_show_peek_icon(true);
        password.set_placeholder_text(Some(text.password));
        form.append(&password);
        let error = ui::label("", "fx-error");
        error.set_visible(false);
        form.append(&error);
        let actions = Box::new(Orientation::Horizontal, 8);
        actions.set_halign(gtk4::Align::End);
        let cancel = button(text.cancel, "fx-ghost");
        let connect = button(text.connect, "fx-primary");
        actions.append(&cancel);
        actions.append(&connect);
        form.append(&actions);
        body.set_child(Some(&form));
        root.append(&body);
        Other {
            root,
            header,
            open,
            chevron,
            body,
            ssid,
            kind,
            password,
            error,
            cancel,
            connect,
        }
    }

    pub fn set_open(&self, open: bool) {
        self.open.set(open);
        self.chevron.queue_draw();
        self.body.set_transition_duration(ui::motion_ms());
        self.body.set_reveal_child(open);
        if open {
            self.ssid.grab_focus();
        }
    }

    pub fn connect(&self, page: &Rc<Page>) {
        let click = GestureClick::new();
        let (open, body, chevron, ssid) = (
            self.open.clone(),
            self.body.clone(),
            self.chevron.clone(),
            self.ssid.clone(),
        );
        click.connect_released(move |_, _, _, _| {
            open.set(!open.get());
            chevron.queue_draw();
            body.set_transition_duration(ui::motion_ms());
            body.set_reveal_child(open.get());
            if open.get() {
                ssid.grab_focus();
            }
        });
        self.header.add_controller(click);
        let password = self.password.clone();
        self.kind.connect_selected_notify(move |kind| {
            password
                .set_visible(HIDDEN_KINDS.get(kind.selected() as usize) != Some(&Security::Open));
        });
        let (open, body, chevron) = (self.open.clone(), self.body.clone(), self.chevron.clone());
        self.cancel.connect_clicked(move |_| {
            open.set(false);
            chevron.queue_draw();
            body.set_reveal_child(false);
        });
        let submit = self.submitter(page);
        let again = submit.clone();
        self.connect.connect_clicked(move |_| submit());
        self.password.connect_activate(move |_| again());
    }

    fn submitter(&self, page: &Rc<Page>) -> Rc<dyn Fn()> {
        let weak = Rc::downgrade(page);
        let (ssid, password, error, kind) = (
            self.ssid.clone(),
            self.password.clone(),
            self.error.clone(),
            self.kind.clone(),
        );
        Rc::new(move || {
            let Some(page) = weak.upgrade() else {
                return;
            };
            let name = ssid.text().trim().to_string();
            let security = HIDDEN_KINDS
                .get(kind.selected() as usize)
                .copied()
                .unwrap_or(Security::Personal);
            let secret = password.text().to_string();
            let problem = if name.is_empty() || name.len() > 32 {
                Some(page.text.network_name)
            } else if security.secured() {
                wifi::check_password(security, &secret)
                    .err()
                    .map(|problem| problem_text(page.text, problem))
            } else {
                None
            };
            if let Some(problem) = problem {
                error.set_text(problem);
                error.set_visible(true);
                return;
            }
            error.set_visible(false);
            page.join(Join {
                ssid: name,
                ap_path: None,
                security,
                password: security.secured().then_some(secret),
                hidden: true,
            });
        })
    }

    /// Fold a join outcome for the hidden network into the form.
    pub fn outcome(&self, page: &Page, ssid: &str, failure: Option<Failure>) {
        if self.ssid.text().trim() != ssid {
            return;
        }
        match failure {
            None => {
                self.ssid.set_text("");
                self.password.set_text("");
                self.error.set_visible(false);
                self.set_open(false);
            }
            Some(failure) => {
                self.error.set_text(failure_text(page.text, failure));
                self.error.set_visible(true);
            }
        }
    }
}

/// Wired ports and VPN profiles, redrawn from `snapshot`.
pub fn render_extra(page: &Rc<Page>, extra: &Box, snapshot: &Snapshot) {
    clear(extra);
    let text = page.text;
    if !snapshot.wired.is_empty() {
        let section = ui::section("");
        for port in &snapshot.wired {
            let title = if snapshot.wired.len() > 1 {
                port.iface.as_str()
            } else {
                text.wired
            };
            section.append(&ui::row(title, Some(&wired_line(text, port)), None));
        }
        extra.append(&section);
    }
    if !snapshot.vpns.is_empty() {
        let section = ui::section(text.vpn);
        for vpn in &snapshot.vpns {
            let subtitle = match vpn.link {
                Link::Connected => Some(text.connected),
                Link::Connecting => Some(text.connecting),
                _ => None,
            };
            let (row, switch) = ui::switch_row(&vpn.name, subtitle, vpn.active.is_some());
            let weak = Rc::downgrade(page);
            let path = vpn.path.clone();
            switch.connect_state_set(move |_, on| {
                if let Some(page) = weak.upgrade() {
                    page.backend.send(Command::SetVpn(path.clone(), on));
                }
                glib::Propagation::Proceed
            });
            section.append(&row);
        }
        extra.append(&section);
    }
}

fn wired_line(text: &Text, port: &crate::model::Wired) -> String {
    if !port.carrier && port.link != Link::Connected {
        return text.unplugged.to_string();
    }
    let state = match port.link {
        Link::Connected => text.connected,
        Link::Connecting => text.connecting,
        Link::Unavailable => text.unavailable,
        Link::Disconnected | Link::Failed => text.disconnected,
    };
    let mut parts = vec![state.to_string()];
    match port.speed {
        0 => {}
        speed if speed >= 1000 && speed % 1000 == 0 => parts.push(format!("{} Gb/s", speed / 1000)),
        speed => parts.push(format!("{speed} Mb/s")),
    }
    if let Some(address) = &port.ip.address {
        parts.push(address.clone());
    }
    parts.join(" · ")
}

/// The quick menu's way out to the full page: the `network.open-settings`
/// action when the host installed one, `flux-settings network` otherwise.
pub fn settings_link(page: &Rc<Page>) {
    let link = button(page.text.settings_link, "fx-link");
    link.set_halign(gtk4::Align::Start);
    link.connect_clicked(|link| {
        if link.activate_action("network.open-settings", None).is_err() {
            let _ = glib::spawn_command_line_async("flux-settings network");
        }
    });
    page.root.append(&link);
}
