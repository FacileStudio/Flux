//! The System page: device name, date and time, language and formats.

use std::rc::Rc;

use flux_config::progress::Step;
use flux_config::settings::Settings;
use flux_config::validate::{self, LOCALES};
use gtk4::prelude::*;
use gtk4::{Box, Button, Entry, ListBox, Orientation, ScrolledWindow, SearchEntry, Widget};

use crate::backend;
use crate::i18n::{self, locale_name, Text};
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
    zones: Vec<String>,
    ntp: Option<bool>,
    clock: Option<bool>,
}

fn load() -> Data {
    Data { settings: backend::settings(), zones: backend::zones(), ntp: backend::ntp(), clock: backend::clock_24h() }
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
    let page = Rc::new(Page { text, body, task });
    let weak = Rc::downgrade(&page);
    page.task.on_finish(move |outcome| {
        if let (Some(page), Outcome::Done { .. }) = (weak.upgrade(), outcome) {
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
            Some("error") => self.task.sample_end(Outcome::Failed, "the device name \"bad-\" is not valid"),
            _ => {}
        }
    }

    fn fill(self: &Rc<Self>, data: &Data) {
        ui::clear(&self.body);
        self.body.append(&self.device(&data.settings));
        self.body.append(&self.datetime(data));
        self.body.append(&self.language(&data.settings));
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
        let apply = ui::primary_button(text.apply);
        let actions = ui::reveal(&ui::actions(&[&apply]));
        let (language_row, language_drop) = ui::choice_row(text.language, "", &names, language, |_| {});
        let (formats_row, formats_drop) = ui::choice_row(text.formats, text.formats_hint, &format_names, formats, |_| {});
        card.append(&language_row);
        card.append(&formats_row);
        let note = ui::label(text.next_login, "fx-subtitle");
        note.add_css_class("fx-note");
        card.append(&note);
        card.append(&actions);
        let changed = {
            let (language_drop, formats_drop, actions) = (language_drop.clone(), formats_drop.clone(), actions.clone());
            move || {
                let moved = language_drop.selected() as usize != language || formats_drop.selected() as usize != formats;
                ui::show(&actions, moved);
            }
        };
        let again = changed.clone();
        language_drop.connect_selected_notify(move |_| changed());
        formats_drop.connect_selected_notify(move |_| again());
        self.wire_apply(&apply, &language_drop, &formats_drop);
        section
    }

    fn wire_apply(self: &Rc<Self>, apply: &Button, language: &gtk4::DropDown, formats: &gtk4::DropDown) {
        let page = self.clone();
        let (language, formats) = (language.clone(), formats.clone());
        apply.connect_clicked(move |_| {
            let locale = LOCALES.get(language.selected() as usize).copied().unwrap_or("en_US.UTF-8");
            let format = match formats.selected() as usize {
                0 => "same",
                index => LOCALES.get(index - 1).copied().unwrap_or("same"),
            };
            page.change(&["set", "locale", locale, "formats", format]);
        });
    }
}
