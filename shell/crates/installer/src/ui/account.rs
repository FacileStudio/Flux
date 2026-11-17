use std::cell::Cell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, Label, Orientation, PasswordEntry, Widget};

use super::{label, page};
use crate::app::{App, Screen};
use crate::model;

struct Fields {
    full_name: Entry,
    username: Entry,
    hostname: Entry,
    password: PasswordEntry,
    again: PasswordEntry,
    name_error: Label,
    username_error: Label,
    hostname_error: Label,
    password_error: Label,
    next: Button,
    syncing: Cell<bool>,
}

fn field(title: &str, input: &impl IsA<Widget>, error: &str) -> (Box, Label) {
    let column = Box::new(Orientation::Vertical, 6);
    column.append(&label(title, "field-label"));
    column.append(input);
    let error = label(error, "field-error");
    error.set_visible(false);
    column.append(&error);
    (column, error)
}

fn entry(value: &str) -> Entry {
    let entry = Entry::new();
    entry.set_text(value);
    entry.set_activates_default(true);
    entry
}

fn password(value: &str) -> PasswordEntry {
    let entry = PasswordEntry::new();
    entry.set_text(value);
    entry.set_show_peek_icon(true);
    entry.set_activates_default(true);
    entry
}

fn refresh(app: &App, fields: &Fields) {
    let state = app.state.borrow();
    fields.name_error.set_visible(!state.full_name.is_empty() && !model::valid_full_name(&state.full_name));
    fields.username_error.set_visible(!state.username.is_empty() && !model::valid_username(&state.username));
    fields.hostname_error.set_visible(!model::valid_hostname(&state.hostname));
    fields.password_error.set_visible(!state.password_again.is_empty() && state.password != state.password_again);
    fields.next.set_sensitive(state.account_ready());
}

fn set_quietly(fields: &Fields, entry: &Entry, value: &str) {
    if entry.text() != value {
        fields.syncing.set(true);
        entry.set_text(value);
        fields.syncing.set(false);
    }
}

fn derive(app: &App, fields: &Fields) {
    let mut state = app.state.borrow_mut();
    if !state.username_edited {
        state.username = model::derive_username(&state.full_name);
        set_quietly(fields, &fields.username, &state.username);
    }
    if !state.hostname_edited {
        state.hostname = model::derive_hostname(&state.username);
        set_quietly(fields, &fields.hostname, &state.hostname);
    }
}

fn wire(app: &Rc<App>, fields: &Rc<Fields>) {
    let connect = |editable: &gtk4::Editable, apply: fn(&App, &Fields, String)| {
        let weak = Rc::downgrade(app);
        let fields = Rc::downgrade(fields);
        editable.connect_changed(move |editable| {
            let (Some(app), Some(fields)) = (weak.upgrade(), fields.upgrade()) else {
                return;
            };
            if fields.syncing.get() {
                return;
            }
            apply(&app, &fields, editable.text().to_string());
            refresh(&app, &fields);
        });
    };
    connect(fields.full_name.upcast_ref::<gtk4::Editable>(), |app, fields, value| {
        app.state.borrow_mut().full_name = value;
        derive(app, fields);
    });
    connect(fields.username.upcast_ref::<gtk4::Editable>(), |app, fields, value| {
        let mut state = app.state.borrow_mut();
        state.username = value;
        state.username_edited = true;
        drop(state);
        derive(app, fields);
    });
    connect(fields.hostname.upcast_ref::<gtk4::Editable>(), |app, _, value| {
        let mut state = app.state.borrow_mut();
        state.hostname = value;
        state.hostname_edited = true;
    });
    connect(fields.password.upcast_ref::<gtk4::Editable>(), |app, _, value| {
        app.state.borrow_mut().password = value;
    });
    connect(fields.again.upcast_ref::<gtk4::Editable>(), |app, _, value| {
        app.state.borrow_mut().password_again = value;
    });
}

pub fn build(app: &Rc<App>) -> Widget {
    let text = app.text();
    let state = app.state.borrow();
    let full_name = entry(&state.full_name);
    let username = entry(&state.username);
    let hostname = entry(&state.hostname);
    let first = password(&state.password);
    let again = password(&state.password_again);
    drop(state);
    let (name_field, name_error) = field(text.full_name, &full_name, text.name_invalid);
    let (user_field, username_error) = field(text.username, &username, text.username_invalid);
    let (first_field, _) = field(text.password, &first, "");
    let (again_field, password_error) = field(text.password_again, &again, text.password_mismatch);
    let (host_field, hostname_error) = field(text.hostname, &hostname, text.hostname_invalid);
    let fields = Rc::new(Fields {
        full_name,
        username,
        hostname,
        password: first,
        again,
        name_error,
        username_error,
        hostname_error,
        password_error,
        next: super::next(text.next),
        syncing: Cell::new(false),
    });
    let passwords = Box::new(Orientation::Horizontal, 16);
    passwords.set_homogeneous(true);
    passwords.append(&first_field);
    passwords.append(&again_field);
    host_field.append(&label(text.hostname_hint, "field-hint"));
    let names = Box::new(Orientation::Horizontal, 16);
    names.set_homogeneous(true);
    names.append(&user_field);
    names.append(&host_field);
    let body = Box::new(Orientation::Vertical, 16);
    body.append(&name_field);
    body.append(&names);
    body.append(&passwords);
    wire(app, &fields);
    refresh(app, &fields);
    let weak = Rc::downgrade(app);
    fields.next.connect_clicked(move |_| {
        if let Some(app) = weak.upgrade() {
            if app.state.borrow().account_ready() {
                app.go(Screen::TimeZone);
            }
        }
    });
    super::default(app, &fields.next);
    let focus = fields.full_name.clone();
    gtk4::glib::idle_add_local_once(move || {
        focus.grab_focus();
    });
    let screen = page(app, text.account_title, text.account_subtitle, &body, &[&fields.next]);
    let owner = fields.clone();
    screen.connect_destroy(move |_| {
        let _ = &owner;
    });
    screen
}
