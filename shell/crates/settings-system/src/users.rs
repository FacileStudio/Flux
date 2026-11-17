//! The Users page: who can sign in, adding people, passwords and removal.

use std::cell::Cell;
use std::rc::Rc;

use flux_config::progress::Step;
use flux_config::settings::{Settings, User};
use flux_config::validate;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Entry, Label, Orientation, PasswordEntry, Revealer, Switch, Widget};

use crate::backend;
use crate::i18n::{self, fill, Text};
use crate::task::{self, Outcome, Task};
use crate::ui;

pub const ID: &str = "users";
pub const ICON: &str = "system-users-symbolic";

/// The page name in the navigation.
pub fn title() -> String {
    i18n::text().users_title.to_string()
}

/// Up to two capital initials of a full name: `Ada Lovelace` gives `AL`.
pub fn initials(full_name: &str) -> String {
    full_name
        .split_whitespace()
        .filter_map(|word| word.chars().find(|c| c.is_alphanumeric()))
        .take(2)
        .flat_map(char::to_uppercase)
        .collect()
}

/// Why a new account cannot be added yet, or `None` when it can.
pub fn add_problem(text: &Text, settings: &Settings, full_name: &str, username: &str, password: &str, again: &str) -> Option<&'static str> {
    if validate::full_name(full_name.trim()).is_err() {
        return Some(text.name_invalid);
    }
    if validate::username(username).is_err() {
        return Some(text.username_invalid);
    }
    if settings.user(username).is_some() {
        return Some(text.username_taken);
    }
    password_problem(text, password, again)
}

/// Why a password pair is refused, or `None`.
pub fn password_problem(text: &Text, password: &str, again: &str) -> Option<&'static str> {
    if password.is_empty() {
        Some(text.password_empty)
    } else if password != again {
        Some(text.password_mismatch)
    } else {
        None
    }
}

struct Page {
    text: &'static Text,
    body: Box,
    task: Rc<Task>,
}

/// The whole page.
pub fn page() -> Widget {
    let text = i18n::text();
    let (scroll, column) = ui::system_page(text.users_title, text.users_subtitle);
    let task = Task::new(text);
    column.append(task.widget());
    let body = Box::new(Orientation::Vertical, 24);
    body.append(&ui::spinner_row(text.users_title));
    column.append(&body);
    let page = Rc::new(Page { text, body, task });
    let weak = Rc::downgrade(&page);
    page.task.on_finish(move |outcome| {
        if let (Some(page), Outcome::Done { .. }) = (weak.upgrade(), outcome) {
            page.reload();
        }
    });
    page.reload();
    let attach = page.clone();
    backend::spawn(backend::running, move |state| {
        if let Some(state) = state.filter(|state| backend::owner(&state.operation) == ID) {
            attach.task.attach(&state.line);
        }
    });
    if backend::preview_state(ID).as_deref() == Some("progress") {
        page.task.sample(Step::Build, 46);
    }
    scroll.upcast()
}

fn entry_field(title: &str, entry: &impl IsA<Widget>) -> Box {
    ui::field(title, entry).0
}

fn password_entry() -> PasswordEntry {
    let entry = PasswordEntry::new();
    entry.set_show_peek_icon(true);
    entry.set_width_chars(8);
    entry
}

impl Page {
    fn reload(self: &Rc<Self>) {
        let page = self.clone();
        backend::spawn(|| (backend::settings(), backend::me()), move |(settings, me)| page.fill(&settings, &me));
    }

    fn fill(self: &Rc<Self>, settings: &Settings, me: &str) {
        ui::clear(&self.body);
        let text = self.text;
        let (section, card) = ui::section("");
        let header = Box::new(Orientation::Horizontal, 12);
        header.add_css_class("fx-section-header");
        let heading = ui::label(text.users_section, "fx-section-title");
        heading.set_hexpand(true);
        header.append(&heading);
        let add = ui::secondary_button(text.add_user);
        header.append(&add);
        section.prepend(&header);
        let form = self.add_form(settings);
        let reveal = ui::reveal(&form);
        let open = reveal.clone();
        add.connect_clicked(move |_| ui::show(&open, !open.reveals_child()));
        card.append(&reveal);
        if settings.users.is_empty() {
            card.append(&ui::empty_state("system-users-symbolic", text.no_users_title, text.no_users_body));
        }
        let admins = settings.users.iter().filter(|user| user.admin).count();
        for user in &settings.users {
            card.append(&self.user_row(user, user.name == me, user.admin && admins == 1));
        }
        self.body.append(&section);
        if backend::preview_state(ID).as_deref() == Some("add") {
            ui::show(&reveal, true);
        }
    }

    fn change(self: &Rc<Self>, args: &[&str], input: Option<String>) {
        self.task.start(args.iter().map(|arg| arg.to_string()).collect(), input);
    }

    fn user_row(self: &Rc<Self>, user: &User, me: bool, last_admin: bool) -> Box {
        let text = self.text;
        let column = Box::new(Orientation::Vertical, 0);
        column.add_css_class("fx-user");
        let line = Box::new(Orientation::Horizontal, 12);
        line.add_css_class("fx-row");
        let avatar = Label::new(Some(&initials(&user.full_name)));
        avatar.add_css_class("fx-avatar");
        avatar.set_valign(Align::Center);
        line.append(&avatar);
        let names = Box::new(Orientation::Vertical, 2);
        names.set_hexpand(true);
        names.set_valign(Align::Center);
        names.append(&ui::label(&user.full_name, "fx-title"));
        let role = if user.admin { text.administrator } else { text.standard };
        names.append(&ui::label(&format!("{} · {role}", user.name), "fx-subtitle"));
        line.append(&names);
        if me {
            let tag = Label::new(Some(text.you));
            tag.add_css_class("fx-tag");
            tag.set_valign(Align::Center);
            line.append(&tag);
        }
        let more = ui::ghost_button(text.change);
        line.append(&more);
        column.append(&line);
        let panel = self.user_panel(user, me, last_admin);
        let reveal = ui::reveal(&panel);
        column.append(&reveal);
        if user.name == "grace" && backend::preview_state(ID).as_deref() == Some("manage") {
            ui::show(&reveal, true);
        }
        more.connect_clicked(move |_| ui::show(&reveal, !reveal.reveals_child()));
        column
    }

    fn user_panel(self: &Rc<Self>, user: &User, me: bool, last_admin: bool) -> Box {
        let text = self.text;
        let panel = Box::new(Orientation::Vertical, 12);
        panel.add_css_class("fx-user-panel");
        let name = user.name.clone();
        let page = self.clone();
        let started = Rc::new(Cell::new(user.admin));
        let (admin_row, admin_switch) = ui::switch_row(text.administrator, text.admin_hint, user.admin, move |on| {
            if started.get() != on {
                started.set(on);
                page.change(&["user", "set-admin", &name, if on { "on" } else { "off" }], None);
            }
        });
        admin_switch.set_sensitive(!me && !last_admin);
        panel.append(&admin_row);
        panel.append(&self.password_box(user));
        panel.append(&self.remove_box(user, me, last_admin));
        panel
    }

    fn password_box(self: &Rc<Self>, user: &User) -> Box {
        let text = self.text;
        let column = Box::new(Orientation::Vertical, 8);
        let open = ui::secondary_button(text.change_password);
        open.set_halign(Align::Start);
        let first = password_entry();
        let second = password_entry();
        let fields = Box::new(Orientation::Vertical, 8);
        fields.append(&entry_field(text.password, &first));
        let (again, error) = ui::field(text.password_again, &second);
        fields.append(&again);
        let save = ui::primary_button(text.save);
        save.set_sensitive(false);
        fields.append(&ui::actions(&[&save]));
        let reveal = ui::reveal(&fields);
        column.append(&open);
        column.append(&reveal);
        open.connect_clicked(move |_| ui::show(&reveal, !reveal.reveals_child()));
        let check = {
            let (first, second, save) = (first.clone(), second.clone(), save.clone());
            move || {
                let problem = password_problem(text, &first.text(), &second.text());
                let shown = if second.text().is_empty() { "" } else { problem.unwrap_or("") };
                ui::set_error(&error, shown);
                save.set_sensitive(problem.is_none());
            }
        };
        let again_check = check.clone();
        first.connect_changed(move |_| check());
        second.connect_changed(move |_| again_check());
        let page = self.clone();
        let name = user.name.clone();
        save.connect_clicked(move |_| {
            let password = first.text().to_string();
            first.set_text("");
            second.set_text("");
            page.change(&["user", "set-password", &name], Some(password));
        });
        column
    }

    fn remove_box(self: &Rc<Self>, user: &User, me: bool, last_admin: bool) -> Box {
        let text = self.text;
        let column = Box::new(Orientation::Vertical, 8);
        let remove = ui::danger_button(text.remove);
        remove.set_halign(Align::Start);
        let reason = if me {
            Some(text.self_remove)
        } else if last_admin {
            Some(text.last_admin)
        } else {
            None
        };
        remove.set_sensitive(reason.is_none());
        column.append(&remove);
        if let Some(reason) = reason {
            column.append(&ui::label(reason, "fx-subtitle"));
            return column;
        }
        let title = fill(text.remove_title, &[("name", &user.full_name)]);
        let body = fill(text.remove_body, &[("name", &user.full_name), ("user", &user.name)]);
        let (confirm, no, yes) = task::confirm(&title, &body, text.cancel, text.remove);
        let reveal = ui::reveal(&confirm);
        column.append(&reveal);
        let open = reveal.clone();
        remove.connect_clicked(move |_| ui::show(&open, true));
        let close = reveal.clone();
        no.connect_clicked(move |_| ui::show(&close, false));
        let page = self.clone();
        let name = user.name.clone();
        yes.connect_clicked(move |_| {
            ui::show(&reveal, false);
            page.change(&["user", "remove", &name], None);
        });
        column
    }

    fn add_form(self: &Rc<Self>, settings: &Settings) -> Box {
        let text = self.text;
        let form = Box::new(Orientation::Vertical, 12);
        form.add_css_class("fx-form");
        let full_name = Entry::new();
        let username = Entry::new();
        let first = password_entry();
        let second = password_entry();
        let admin = Switch::new();
        form.append(&entry_field(text.full_name, &full_name));
        let (username_field, _) = ui::field(text.username, &username);
        username_field.append(&ui::label(text.username_hint, "fx-subtitle"));
        form.append(&username_field);
        let passwords = Box::new(Orientation::Horizontal, 12);
        passwords.set_homogeneous(true);
        passwords.append(&entry_field(text.password, &first));
        passwords.append(&entry_field(text.password_again, &second));
        form.append(&passwords);
        form.append(&ui::row(text.administrator, text.admin_hint, &admin));
        let error = ui::label("", "fx-field-error");
        error.set_visible(false);
        form.append(&error);
        let cancel = ui::ghost_button(text.cancel);
        let add = ui::primary_button(text.add_user);
        add.set_sensitive(false);
        form.append(&ui::actions(&[&cancel, &add]));
        if backend::preview() {
            full_name.set_text("Katherine Johnson");
            username.set_text("katherine");
        }
        let edited = Rc::new(Cell::new(false));
        let settings = settings.clone();
        let check = {
            let (full_name, username, first, second, add) = (full_name.clone(), username.clone(), first.clone(), second.clone(), add.clone());
            move || {
                let problem = add_problem(text, &settings, &full_name.text(), &username.text(), &first.text(), &second.text());
                let typed = !full_name.text().is_empty() && !second.text().is_empty();
                ui::set_error(&error, if typed { problem.unwrap_or("") } else { "" });
                add.set_sensitive(problem.is_none());
            }
        };
        let derive = {
            let (username, edited, check) = (username.clone(), edited.clone(), check.clone());
            move |entry: &Entry| {
                if !edited.get() {
                    username.set_text(&validate::derive_username(&entry.text()));
                }
                check();
            }
        };
        full_name.connect_changed(derive);
        let username_check = check.clone();
        let derived = edited.clone();
        username.connect_changed(move |entry| {
            if entry.state_flags().contains(gtk4::StateFlags::FOCUS_WITHIN) {
                derived.set(true);
            }
            username_check();
        });
        let first_check = check.clone();
        first.connect_changed(move |_| first_check());
        second.connect_changed(move |_| check());
        self.wire_add(&form, &cancel, &add, (&full_name, &username, &first, &second, &admin));
        form
    }

    fn wire_add(self: &Rc<Self>, form: &Box, cancel: &Button, add: &Button, fields: (&Entry, &Entry, &PasswordEntry, &PasswordEntry, &Switch)) {
        let (full_name, username, first, second, admin) =
            (fields.0.clone(), fields.1.clone(), fields.2.clone(), fields.3.clone(), fields.4.clone());
        let hide = form.clone();
        cancel.connect_clicked(move |_| {
            if let Some(revealer) = hide.parent().and_downcast::<Revealer>() {
                ui::show(&revealer, false);
            }
        });
        let page = self.clone();
        add.connect_clicked(move |_| {
            let password = first.text().to_string();
            first.set_text("");
            second.set_text("");
            let mut args = vec!["user", "add"];
            let name = username.text().to_string();
            let full = full_name.text().trim().to_string();
            args.push(&name);
            args.push("--full-name");
            args.push(&full);
            if admin.is_active() {
                args.push("--admin");
            }
            page.change(&args, Some(password));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::EN;

    #[test]
    fn initials_of_names() {
        assert_eq!(initials("Ada Lovelace"), "AL");
        assert_eq!(initials("élodie"), "É");
        assert_eq!(initials("Jean-Luc de la Fontaine"), "JD");
        assert_eq!(initials(""), "");
    }

    #[test]
    fn add_rules() {
        let mut settings = Settings::default();
        let _ = settings.add_user("ada", "Ada", true, &[]);
        assert_eq!(add_problem(&EN, &settings, "Bob", "bob", "pw", "pw"), None);
        assert_eq!(add_problem(&EN, &settings, "", "bob", "pw", "pw"), Some(EN.name_invalid));
        assert_eq!(add_problem(&EN, &settings, "Bob", "Bob", "pw", "pw"), Some(EN.username_invalid));
        assert_eq!(add_problem(&EN, &settings, "Ada", "ada", "pw", "pw"), Some(EN.username_taken));
        assert_eq!(add_problem(&EN, &settings, "Bob", "bob", "pw", "px"), Some(EN.password_mismatch));
        assert_eq!(add_problem(&EN, &settings, "Bob", "bob", "", ""), Some(EN.password_empty));
    }
}
