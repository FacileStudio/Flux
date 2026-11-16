//! The pairing sheet: a card over a scrim, inside the page, for the agent's
//! questions and for a pairing that failed.

use std::cell::Cell;
use std::rc::{Rc, Weak};

use gtk4::prelude::*;
use gtk4::{glib, Align, Box, Button, Entry, Image, Label, Orientation, Revealer};

use super::Page;
use crate::i18n::{named, Text};
use crate::model::{self, Failure, Prompt};
use crate::ui;

/// What an entry on the sheet accepts.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Pin,
    Passkey,
}

/// Everything the sheet shows for one state.
pub struct Content {
    title: String,
    body: String,
    code: Option<String>,
    input: Option<Input>,
    cancel: &'static str,
    confirm: Option<&'static str>,
}

fn code_markup(passkey: u32, entered: u16) -> String {
    let mut typed = 0;
    model::passkey_text(passkey)
        .chars()
        .map(|c| {
            if c == ' ' {
                return " ".to_string();
            }
            typed += 1;
            if typed <= entered {
                c.to_string()
            } else {
                format!("<span alpha=\"40%\">{c}</span>")
            }
        })
        .collect()
}

impl Content {
    pub fn prompt(text: &'static Text, name: &str, prompt: &Prompt) -> Content {
        let mut content = Content {
            title: named(text.confirm_title, name),
            body: String::new(),
            code: None,
            input: None,
            cancel: text.cancel,
            confirm: Some(text.pair),
        };
        match prompt {
            Prompt::Confirm(passkey) => {
                content.body = named(text.confirm_body, name);
                content.code = Some(glib::markup_escape_text(&model::passkey_text(*passkey)).to_string());
                content.confirm = Some(text.confirm_button);
            }
            Prompt::Authorize => {
                content.title = named(text.authorize_title, name);
                content.body = named(text.authorize_body, name);
            }
            Prompt::AuthorizeService(service) => {
                content.title = named(text.service_title, name);
                content.body = named(text.service_body, name).replace("{service}", text.service(*service));
                content.cancel = text.deny;
                content.confirm = Some(text.allow);
            }
            Prompt::RequestPin => {
                content.title = named(text.request_pin_title, name);
                content.body = text.request_pin_body.to_string();
                content.input = Some(Input::Pin);
            }
            Prompt::RequestPasskey => {
                content.title = named(text.request_passkey_title, name);
                content.body = text.request_passkey_body.to_string();
                content.input = Some(Input::Passkey);
            }
            Prompt::DisplayPin(pin) => {
                content.title = named(text.display_title, name);
                content.body = named(text.display_pin_body, name);
                content.code = Some(glib::markup_escape_text(pin).to_string());
                content.confirm = None;
            }
            Prompt::DisplayPasskey { passkey, entered } => {
                content.title = named(text.display_title, name);
                content.body = text.display_body.to_string();
                content.code = Some(code_markup(*passkey, *entered));
                content.confirm = None;
            }
        }
        content
    }

    pub fn failed(text: &'static Text, name: &str, failure: Failure) -> Content {
        Content {
            title: named(text.failed_title, name),
            body: text.failure(failure).to_string(),
            code: None,
            input: None,
            cancel: text.close,
            confirm: Some(text.try_again),
        }
    }
}

pub struct Sheet {
    pub root: Revealer,
    icon: Image,
    title: Label,
    body: Label,
    code: Label,
    entry: Entry,
    cancel: Button,
    confirm: Button,
    input: Rc<Cell<Option<Input>>>,
}

fn valid(input: Option<Input>, text: &str) -> bool {
    match input {
        Some(Input::Pin) => model::valid_pin(text),
        Some(Input::Passkey) => model::parse_passkey(text).is_some(),
        None => true,
    }
}

impl Sheet {
    pub fn new() -> Sheet {
        let card = Box::new(Orientation::Vertical, 12);
        card.add_css_class("bt-sheet");
        card.set_halign(Align::Center);
        card.set_valign(Align::Center);
        card.set_width_request(400);
        let icon = Image::new();
        icon.set_pixel_size(32);
        icon.set_halign(Align::Start);
        icon.add_css_class("bt-sheet-icon");
        let title = ui::label("", "bt-sheet-title");
        let body = ui::label("", "fx-subtitle");
        let code = Label::new(None);
        code.add_css_class("bt-code");
        code.set_selectable(true);
        let entry = Entry::new();
        entry.add_css_class("bt-entry");
        let cancel = Button::new();
        cancel.add_css_class("fx-button");
        cancel.add_css_class("fx-ghost");
        let confirm = Button::new();
        confirm.add_css_class("fx-button");
        confirm.add_css_class("fx-primary");
        let actions = Box::new(Orientation::Horizontal, 8);
        actions.set_halign(Align::End);
        actions.set_margin_top(8);
        actions.append(&cancel);
        actions.append(&confirm);
        card.append(&icon);
        card.append(&title);
        card.append(&body);
        card.append(&code);
        card.append(&entry);
        card.append(&actions);
        let scrim = Box::new(Orientation::Vertical, 0);
        scrim.add_css_class("bt-scrim");
        scrim.set_hexpand(true);
        scrim.set_vexpand(true);
        scrim.append(&card);
        card.set_vexpand(true);
        let root = Revealer::new();
        root.set_transition_type(gtk4::RevealerTransitionType::Crossfade);
        root.set_transition_duration(ui::motion(160));
        root.set_child(Some(&scrim));
        root.set_can_target(false);
        let input = Rc::new(Cell::new(None));
        let (accepts, button) = (Rc::clone(&input), confirm.clone());
        entry.connect_changed(move |entry| button.set_sensitive(valid(accepts.get(), entry.text().trim())));
        Sheet {
            root,
            icon,
            title,
            body,
            code,
            entry,
            cancel,
            confirm,
            input,
        }
    }

    pub fn connect(&self, page: Weak<Page>) {
        let weak = page.clone();
        self.confirm.connect_clicked(move |_| {
            if let Some(page) = weak.upgrade() {
                page.confirm();
            }
        });
        let weak = page.clone();
        self.cancel.connect_clicked(move |_| {
            if let Some(page) = weak.upgrade() {
                page.cancel();
            }
        });
        let confirm = self.confirm.clone();
        self.entry.connect_activate(move |_| {
            if confirm.is_sensitive() {
                confirm.emit_clicked();
            }
        });
        let key = gtk4::EventControllerKey::new();
        key.connect_key_pressed(move |_, key, _, _| {
            if key != gtk4::gdk::Key::Escape {
                return glib::Propagation::Proceed;
            }
            if let Some(page) = page.upgrade() {
                page.cancel();
            }
            glib::Propagation::Stop
        });
        self.root.add_controller(key);
    }

    pub fn entry_text(&self) -> String {
        self.entry.text().trim().to_string()
    }

    pub fn show(&self, icon: &str, content: &Content) {
        let opening = !self.root.reveals_child();
        self.icon.set_icon_name(Some(icon));
        self.title.set_text(&content.title);
        self.body.set_text(&content.body);
        self.body.set_visible(!content.body.is_empty());
        self.code.set_markup(content.code.as_deref().unwrap_or_default());
        self.code.set_visible(content.code.is_some());
        self.cancel.set_label(content.cancel);
        self.confirm.set_visible(content.confirm.is_some());
        self.confirm.set_label(content.confirm.unwrap_or_default());
        self.entry.set_visible(content.input.is_some());
        let input = content.input;
        self.input.set(input);
        if opening {
            self.entry.set_text("");
        }
        self.entry.set_input_purpose(match input {
            Some(Input::Passkey) => gtk4::InputPurpose::Digits,
            _ => gtk4::InputPurpose::FreeForm,
        });
        self.entry.set_max_length(if input == Some(Input::Passkey) { 7 } else { 16 });
        self.entry.set_placeholder_text(Some(match input {
            Some(Input::Passkey) => "000000",
            _ => "0000",
        }));
        self.confirm.set_sensitive(valid(input, &self.entry_text()));
        self.root.set_can_target(true);
        self.root.set_reveal_child(true);
        if input.is_some() {
            self.entry.grab_focus();
        } else if content.confirm.is_some() {
            self.confirm.grab_focus();
        } else {
            self.cancel.grab_focus();
        }
    }

    pub fn hide(&self) {
        self.root.set_can_target(false);
        self.root.set_reveal_child(false);
    }
}
