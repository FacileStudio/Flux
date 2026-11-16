//! One network in the list: a header line and the part that opens under it.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box, Button, DrawingArea, GestureClick, Label, Orientation, Revealer, Spinner};

use super::{draw, expand, Expand, Page};
use crate::model::{wifi, Link, Network};
use crate::ui;

pub struct NetRow {
    pub outer: Revealer,
    ssid: String,
    header: Box,
    level: Rc<Cell<u8>>,
    bars: DrawingArea,
    status: Label,
    lock: DrawingArea,
    spinner: Spinner,
    more: Button,
    more_open: Rc<Cell<bool>>,
    more_icon: DrawingArea,
    body: Revealer,
    shown: RefCell<Option<(Expand, String)>>,
    form: RefCell<Option<expand::PasswordForm>>,
}

impl NetRow {
    pub fn new(page: &Rc<Page>, ssid: &str) -> Rc<NetRow> {
        let outer = Revealer::new();
        outer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
        outer.set_transition_duration(0);
        let column = Box::new(Orientation::Vertical, 0);
        column.add_css_class("fx-net");
        outer.set_child(Some(&column));
        let header = Box::new(Orientation::Horizontal, 12);
        header.add_css_class("fx-net-header");
        let level = Rc::new(Cell::new(0));
        let bars = draw::bars(level.clone());
        header.append(&bars);
        let text = Box::new(Orientation::Vertical, 2);
        text.set_hexpand(true);
        text.set_valign(Align::Center);
        let name = ui::label(ssid, "fx-title");
        name.set_wrap(false);
        name.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        text.append(&name);
        let status = ui::label("", "fx-subtitle");
        text.append(&status);
        header.append(&text);
        let spinner = Spinner::new();
        header.append(&spinner);
        let lock = draw::lock();
        header.append(&lock);
        let more_open = Rc::new(Cell::new(false));
        let more_icon = draw::chevron(more_open.clone());
        let more = Button::new();
        more.add_css_class("fx-icon-button");
        more.set_child(Some(&more_icon));
        more.set_valign(Align::Center);
        more.update_property(&[gtk4::accessible::Property::Label(page.text.details)]);
        header.append(&more);
        column.append(&header);
        let body = Revealer::new();
        body.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
        column.append(&body);
        let row = Rc::new(NetRow {
            outer,
            ssid: ssid.to_string(),
            header,
            level,
            bars,
            status,
            lock,
            spinner,
            more,
            more_open,
            more_icon,
            body,
            shown: RefCell::new(None),
            form: RefCell::new(None),
        });
        row.wire(page);
        row
    }

    fn wire(&self, page: &Rc<Page>) {
        let click = GestureClick::new();
        let weak = Rc::downgrade(page);
        let ssid = self.ssid.clone();
        click.connect_released(move |_, _, _, _| {
            if let Some(page) = weak.upgrade() {
                activate(&page, &ssid);
            }
        });
        self.header.add_controller(click);
        let weak = Rc::downgrade(page);
        let ssid = self.ssid.clone();
        self.more.connect_clicked(move |_| {
            if let Some(page) = weak.upgrade() {
                page.toggle(&ssid, Expand::Details);
            }
        });
    }

    /// Redraw from `network` and the page state.
    pub fn update(&self, page: &Rc<Page>, network: &Network) {
        let snapshot = page.snapshot.borrow();
        let joining = page.joining.borrow().as_deref() == Some(network.ssid.as_str());
        let link = snapshot
            .wifi
            .as_ref()
            .map_or(Link::Disconnected, |device| device.link);
        let connected = network.active && link == Link::Connected;
        let connecting = joining || (network.active && link == Link::Connecting);
        self.level.set(wifi::bars(network.strength));
        self.bars.queue_draw();
        self.status
            .set_text(&status(page, network, connected, connecting));
        self.lock.set_visible(network.security.secured());
        self.spinner.set_visible(connecting);
        self.spinner.set_spinning(connecting);
        let more = network.known || connected;
        self.more.set_visible(page.mode == super::Mode::Full);
        self.more.set_opacity(if more { 1.0 } else { 0.0 });
        self.more.set_can_target(more);
        self.more.set_focusable(more);
        if connected {
            self.header.add_css_class("fx-connected");
        } else {
            self.header.remove_css_class("fx-connected");
        }
        let open = page
            .open
            .borrow()
            .clone()
            .filter(|(ssid, _)| *ssid == network.ssid)
            .map(|(_, expand)| expand);
        self.more_open.set(open == Some(Expand::Details));
        self.more_icon.queue_draw();
        drop(snapshot);
        self.show(page, network, open, connecting);
    }

    fn show(&self, page: &Rc<Page>, network: &Network, open: Option<Expand>, connecting: bool) {
        self.body.set_transition_duration(ui::motion_ms());
        let Some(expand) = open else {
            self.body.set_reveal_child(false);
            self.shown.borrow_mut().take();
            self.form.borrow_mut().take();
            return;
        };
        let failure = page.failures.borrow().get(&network.ssid).copied();
        let key = format!(
            "{failure:?}{connecting}{}",
            super::sections::details_key(page, network)
        );
        let same = self
            .shown
            .borrow()
            .as_ref()
            .is_some_and(|(shown, old)| *shown == expand && *old == key);
        if same {
            return;
        }
        let typing = matches!(self.shown.borrow().as_ref(), Some((Expand::Password, _)));
        match expand {
            Expand::Password if typing => {
                if let Some(form) = self.form.borrow().as_ref() {
                    form.refresh(page, failure, connecting);
                }
            }
            Expand::Password => {
                let form = expand::PasswordForm::new(page, network);
                form.refresh(page, failure, connecting);
                self.body.set_child(Some(&form.root));
                form.focus();
                *self.form.borrow_mut() = Some(form);
            }
            Expand::Details => self.body.set_child(Some(&expand::details(page, network))),
            Expand::Unsupported => self.body.set_child(Some(&expand::unsupported(page))),
        }
        *self.shown.borrow_mut() = Some((expand, key));
        self.body.set_reveal_child(true);
    }
}

fn status(page: &Page, network: &Network, connected: bool, connecting: bool) -> String {
    let text = page.text;
    if connecting {
        return text.connecting.to_string();
    }
    if connected {
        return text.connected.to_string();
    }
    let open = page
        .open
        .borrow()
        .as_ref()
        .is_some_and(|(ssid, _)| *ssid == network.ssid);
    if let Some(failure) = page.failures.borrow().get(&network.ssid).filter(|_| !open) {
        return expand::failure_text(text, *failure).to_string();
    }
    let security = expand::security_label(text, network.security);
    if network.known {
        format!("{} · {security}", text.saved)
    } else {
        security.to_string()
    }
}

/// What a click on a row does: details for the connected one, a direct join for
/// a known or open network, the password field for a new secured one.
fn activate(page: &Rc<Page>, ssid: &str) {
    let Some(network) = page.network(ssid) else {
        return;
    };
    let snapshot_link = page
        .snapshot
        .borrow()
        .wifi
        .as_ref()
        .map(|device| device.link);
    if network.active && snapshot_link == Some(Link::Connected) {
        if page.mode == super::Mode::Full {
            page.toggle(ssid, Expand::Details);
        }
        return;
    }
    if !network.security.supported() {
        page.toggle(ssid, Expand::Unsupported);
        return;
    }
    let wrong_password = page
        .failures
        .borrow()
        .get(ssid)
        .is_some_and(|failure| *failure == wifi::Failure::WrongPassword);
    if (network.known && !wrong_password) || !network.security.secured() {
        page.close();
        page.join(expand::join_for(&network, None));
        return;
    }
    page.toggle(ssid, Expand::Password);
}
