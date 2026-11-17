//! The System page: device name, date and time, language and formats.

use std::rc::Rc;

use flux_config::progress::Step;
use flux_config::settings::Settings;
use flux_config::validate::{self, KEYMAPS, LOCALES};
use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, ListBox, Orientation, ScrolledWindow, SearchEntry, Widget};

use crate::backend::{self, Software};
use crate::i18n::{self, keymap_name, locale_name, Text};
use crate::task::{Outcome, Task};
use crate::ui;

pub const ID: &str = "system";
pub const ICON: &str = "computer-symbolic";

/// The page name in the navigation.
pub fn title() -> String {
    i18n::text().system_title.to_string()
}

struct Data {
    settings: Settings,
    problem: Option<String>,
    zones: Vec<String>,
    ntp: Option<bool>,
    clock: Option<bool>,
    nvidia_gpu: bool,
}

fn load() -> Data {
    Data {
        settings: backend::settings(),
        problem: backend::problem(),
        zones: backend::zones(),
        ntp: backend::ntp(),
        clock: backend::clock_24h(),
        nvidia_gpu: backend::nvidia_gpu(),
    }
}

/// One software switch: title, description, current state, and how it changes the choices.
type SoftwareRow = (&'static str, &'static str, bool, fn(&mut Software, bool));

/// The `flux-config set` arguments that move the software choices from `old` to `new`; empty when nothing moved.
pub fn software_args(old: Software, new: Software) -> Vec<String> {
    let pairs = [
        ("software.facile", old.facile, new.facile),
        ("software.apps", old.apps, new.apps),
        ("software.gaming", old.gaming, new.gaming),
        ("hardware.nvidia", old.nvidia, new.nvidia),
    ];
    let changed: Vec<String> = pairs
        .into_iter()
        .filter(|(_, before, after)| before != after)
        .flat_map(|(key, _, after)| [key.to_string(), after.to_string()])
        .collect();
    if changed.is_empty() {
        changed
    } else {
        std::iter::once("set".to_string()).chain(changed).collect()
    }
}

fn zone_list(names: &[String], zones: &[String], current: Option<&str>) -> ListBox {
    let list = ListBox::new();
    list.add_css_class("fx-list");
    for (index, name) in names.iter().enumerate() {
        let row = gtk4::ListBoxRow::new();
        row.set_child(Some(&ui::label(name, "fx-title")));
        let chosen = match index {
            0 => current.is_none(),
            _ => current == zones.get(index - 1).map(String::as_str),
        };
        if chosen {
            row.add_css_class("fx-chosen");
        }
        list.append(&row);
    }
    list
}

struct Page {
    text: &'static Text,
    body: Box,
    task: Rc<Task>,
}

/// The whole page.
pub fn page() -> Widget {
    let text = i18n::text();
    let (scroll, column) = ui::system_page(text.system_title, text.system_subtitle);
    let task = Task::new(text);
    column.append(task.widget());
    let body = Box::new(Orientation::Vertical, 24);
    body.append(&ui::spinner_row(text.system_title));
    column.append(&body);
    task.guard(&body);
    let page = Rc::new(Page { text, body, task });
    let weak = Rc::downgrade(&page);
    page.task.on_finish(move |_| {
        if let Some(page) = weak.upgrade() {
            page.reload();
        }
    });
    page.reload();
    page.reattach();
    page.preview();
    scroll.upcast()
}

impl Page {
    fn reload(self: &Rc<Self>) {
        let page = self.clone();
        backend::spawn(load, move |data| page.fill(&data));
    }

    fn reattach(self: &Rc<Self>) {
        let page = self.clone();
        backend::spawn(backend::running, move |state| {
            if let Some(state) = state.filter(|state| backend::owner(&state.operation) == ID) {
                page.task.attach(&state.line);
            }
        });
    }

    fn preview(self: &Rc<Self>) {
        match backend::preview_state(ID).as_deref() {
            Some("progress") => self.task.sample(Step::Activate, 88),
            Some("error") => self.task.sample_end(Outcome::Failed, 1, "flux-config: nixos-rebuild failed (exit status: 1): error: attribute 'x' missing"),
            Some("auth") => self.task.sample_end(Outcome::Failed, backend::AUTH_REFUSED, "Error executing command as another user: Not authorized\n\nThis incident has been reported."),
            _ => {}
        }
    }

    fn fill(self: &Rc<Self>, data: &Data) {
        ui::clear(&self.body);
        if let Some(problem) = &data.problem {
            self.body.append(&ui::problem_banner(self.text, problem));
        }
        if backend::preview_state(ID).as_deref() != Some("software") {
            self.body.append(&self.device(&data.settings));
            self.body.append(&self.datetime(data));
        }
        self.body.append(&self.language(&data.settings));
        self.body.append(&self.software(backend::software(&data.settings), data.nvidia_gpu));
    }

    fn change(self: &Rc<Self>, args: &[&str]) {
        self.task.start(args.iter().map(|arg| arg.to_string()).collect(), None);
    }

    fn device(self: &Rc<Self>, settings: &Settings) -> Box {
        let text = self.text;
        let (section, card) = ui::section(text.device_section);
        let entry = Entry::new();
        entry.set_text(&settings.hostname);
        entry.set_width_chars(18);
        entry.set_max_length(63);
        let rename = ui::secondary_button(text.rename);
        rename.set_sensitive(false);
        let suffix = Box::new(Orientation::Horizontal, 8);
        suffix.append(&entry);
        suffix.append(&rename);
        card.append(&ui::row(text.device_name, text.device_name_hint, &suffix));
        let error = ui::label("", "fx-field-error");
        error.add_css_class("fx-note");
        error.set_visible(false);
        card.append(&error);
        let current = settings.hostname.clone();
        let button = rename.clone();
        entry.connect_changed(move |entry| {
            let name = entry.text().to_string();
            let valid = validate::hostname(&name).is_ok();
            ui::set_error(&error, if valid || name.is_empty() { "" } else { text.hostname_invalid });
            button.set_sensitive(valid && name != current);
        });
        let page = self.clone();
        let input = entry.clone();
        rename.connect_clicked(move |_| page.change(&["set", "hostname", input.text().as_str()]));
        let button = rename.clone();
        entry.connect_activate(move |_| {
            if button.is_sensitive() {
                button.emit_clicked();
            }
        });
        section
    }

    fn datetime(self: &Rc<Self>, data: &Data) -> Box {
        let text = self.text;
        let (section, card) = ui::section(text.datetime_section);
        match data.ntp {
            Some(active) => card.append(&ui::switch_row(text.auto_time, text.auto_time_hint, active, self.toggle(backend::set_ntp)).0),
            None => card.append(&ui::row(text.auto_time, text.unavailable, &ui::nothing())),
        }
        let zone = data.settings.timezone.clone();
        let subtitle = zone.clone().unwrap_or_else(|| text.timezone_auto.to_string());
        let change = ui::secondary_button(text.change);
        card.append(&ui::row(text.timezone, &subtitle, &change));
        let picker = self.zone_picker(&data.zones, zone.as_deref());
        let revealer = ui::reveal(&picker);
        card.append(&revealer);
        if backend::preview_state(ID).as_deref() == Some("zones") {
            ui::show(&revealer, true);
        }
        change.connect_clicked(move |_| ui::show(&revealer, !revealer.reveals_child()));
        if let Some(active) = data.clock {
            card.append(&ui::switch_row(text.clock_24h, text.clock_24h_hint, active, self.toggle(backend::set_clock_24h)).0);
        }
        section
    }

    fn toggle(self: &Rc<Self>, set: fn(bool) -> bool) -> impl Fn(bool) + 'static {
        let page = self.clone();
        move |on| {
            let page = page.clone();
            backend::spawn(move || set(on), move |ok| page.after_toggle(ok));
        }
    }

    fn after_toggle(self: &Rc<Self>, ok: bool) {
        if !ok {
            self.reload();
        }
    }

    fn zone_picker(self: &Rc<Self>, zones: &[String], current: Option<&str>) -> Box {
        let text = self.text;
        let column = Box::new(Orientation::Vertical, 8);
        column.add_css_class("fx-picker");
        let search = SearchEntry::new();
        search.set_placeholder_text(Some(text.zone_search));
        let mut names = vec![text.zone_automatic.to_string()];
        names.extend(zones.iter().map(|zone| zone.replace('_', " ")));
        let list = zone_list(&names, zones, current);
        let filter = search.clone();
        let labels = names.clone();
        list.set_filter_func(move |row| {
            let query = filter.text().to_lowercase();
            let index = usize::try_from(row.index()).unwrap_or(0);
            query.is_empty() || labels.get(index).is_some_and(|name| name.to_lowercase().contains(&query))
        });
        let refilter = list.clone();
        search.connect_search_changed(move |_| refilter.invalidate_filter());
        let page = self.clone();
        let values = zones.to_vec();
        list.connect_row_activated(move |_, row| {
            let index = usize::try_from(row.index()).unwrap_or(0);
            let zone = if index == 0 { Some("auto".to_string()) } else { values.get(index - 1).cloned() };
            if let Some(zone) = zone {
                page.change(&["set", "timezone", &zone]);
            }
        });
        let scroll = ScrolledWindow::new();
        scroll.add_css_class("fx-list-panel");
        scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        scroll.set_min_content_height(240);
        scroll.set_child(Some(&list));
        column.append(&search);
        column.append(&scroll);
        column
    }

    fn language(self: &Rc<Self>, settings: &Settings) -> Box {
        let text = self.text;
        let (section, card) = ui::section(text.language_section);
        let names: Vec<&str> = LOCALES.iter().map(|locale| locale_name(locale)).collect();
        let language = LOCALES.iter().position(|locale| *locale == settings.locale).unwrap_or(0);
        let mut format_names = vec![text.formats_same];
        format_names.extend(names.iter().copied());
        let formats = settings
            .formats
            .as_ref()
            .and_then(|formats| LOCALES.iter().position(|locale| locale == formats))
            .map_or(0, |index| index + 1);
        let keymap_names: Vec<&str> = KEYMAPS.iter().map(|keymap| keymap_name(keymap)).collect();
        let keymap = KEYMAPS.iter().position(|keymap| *keymap == settings.keymap);
        let apply = ui::primary_button(text.apply);
        let actions = ui::reveal(&ui::actions(&[&apply]));
        let (language_row, language_drop) = ui::choice_row(text.language, "", &names, language, |_| {});
        let (formats_row, formats_drop) = ui::choice_row(text.formats, text.formats_hint, &format_names, formats, |_| {});
        let (keymap_row, keymap_drop) = ui::choice_row(text.login_keyboard, text.login_keyboard_hint, &keymap_names, keymap.unwrap_or(0), |_| {});
        keymap_row.set_visible(keymap.is_some());
        let keymap = keymap.unwrap_or(0);
        card.append(&language_row);
        card.append(&formats_row);
        card.append(&keymap_row);
        let note = ui::label(text.next_login, "fx-subtitle");
        note.add_css_class("fx-note");
        card.append(&note);
        card.append(&actions);
        let changed = {
            let (language_drop, formats_drop, keymap_drop, actions) = (language_drop.clone(), formats_drop.clone(), keymap_drop.clone(), actions.clone());
            move || {
                let moved = language_drop.selected() as usize != language
                    || formats_drop.selected() as usize != formats
                    || keymap_drop.selected() as usize != keymap;
                ui::show(&actions, moved);
            }
        };
        for drop in [&language_drop, &formats_drop, &keymap_drop] {
            let changed = changed.clone();
            drop.connect_selected_notify(move |_| changed());
        }
        self.wire_apply(&apply, [&language_drop, &formats_drop, &keymap_drop], keymap);
        section
    }

    fn wire_apply(self: &Rc<Self>, apply: &Button, drops: [&gtk4::DropDown; 3], keymap: usize) {
        let page = self.clone();
        let [language, formats, keymaps] = drops.map(Clone::clone);
        apply.connect_clicked(move |_| {
            let locale = LOCALES.get(language.selected() as usize).copied().unwrap_or("en_US.UTF-8");
            let format = match formats.selected() as usize {
                0 => "same",
                index => LOCALES.get(index - 1).copied().unwrap_or("same"),
            };
            let mut args = vec!["set", "locale", locale, "formats", format];
            let chosen = keymaps.selected() as usize;
            if let Some(layout) = KEYMAPS.get(chosen).filter(|_| chosen != keymap) {
                args.extend(["keymap", layout]);
            }
            page.change(&args);
        });
    }

    fn software(self: &Rc<Self>, current: Software, nvidia_gpu: bool) -> Box {
        let text = self.text;
        let (section, card) = ui::section(text.software_section);
        let chosen = Rc::new(std::cell::Cell::new(current));
        let apply = ui::primary_button(text.apply);
        let actions = ui::reveal(&ui::actions(&[&apply]));
        let rows: [SoftwareRow; 4] = [
            (text.software_facile, text.software_facile_body, current.facile, |software, on| software.facile = on),
            (text.software_apps, text.software_apps_body, current.apps, |software, on| software.apps = on),
            (text.software_gaming, text.software_gaming_body, current.gaming, |software, on| software.gaming = on),
            (text.software_nvidia, text.software_nvidia_body, current.nvidia, |software, on| software.nvidia = on),
        ];
        for (index, (title, body, active, set)) in rows.into_iter().enumerate() {
            let (chosen, actions) = (chosen.clone(), actions.clone());
            let (row, _) = ui::switch_row(title, body, active, move |on| {
                let mut software = chosen.get();
                set(&mut software, on);
                chosen.set(software);
                ui::show(&actions, software != current);
            });
            row.set_visible(index < 3 || nvidia_gpu || current.nvidia);
            card.append(&row);
        }
        let note = ui::label(text.software_note, "fx-subtitle");
        note.add_css_class("fx-note");
        card.append(&note);
        card.append(&actions);
        let page = self.clone();
        apply.connect_clicked(move |_| {
            let args = software_args(current, chosen.get());
            if !args.is_empty() {
                page.task.start(args, None);
            }
        });
        section
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn software_changes_become_one_command() {
        let old = Software { facile: true, apps: true, gaming: false, nvidia: false };
        assert!(software_args(old, old).is_empty());
        let new = Software { gaming: true, nvidia: true, ..old };
        assert_eq!(software_args(old, new), ["set", "software.gaming", "true", "hardware.nvidia", "true"]);
        let minimal = Software { apps: false, ..old };
        assert_eq!(software_args(old, minimal), ["set", "software.apps", "false"]);
    }
}
