//! The Power page: battery, power mode, and the idle and sleep rules.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gio, glib, Orientation};

use crate::config::{self, Lid, PowerConfig};
use crate::format;
use crate::i18n::Text;
use crate::modes::Modes;
use crate::source::{Snapshot, Source};
use crate::ui;

/// The durations each timeout row offers, in seconds; `None` is never.
const CHOICES: [Option<u32>; 8] = [Some(60), Some(120), Some(300), Some(600), Some(900), Some(1800), Some(3600), None];

struct Battery {
    block: gtk4::Box,
    rows: gtk4::Box,
    empty: gtk4::Box,
    level: gtk4::LevelBar,
    level_title: gtk4::Label,
    level_line: gtk4::Label,
    health: gtk4::Box,
    health_line: gtk4::Label,
    health_title: gtk4::Label,
}

struct Page {
    text: &'static Text,
    source: Source,
    config: RefCell<PowerConfig>,
    battery: Battery,
    modes: Modes,
    mode_missing: gtk4::Box,
    suspend_battery: gtk4::Box,
    lid: gtk4::Box,
    error: gtk4::Label,
}

fn battery_block(text: &'static Text, source: &Source) -> Battery {
    let (block, card) = ui::section(text.battery, None);
    let rows = gtk4::Box::new(Orientation::Vertical, 0);
    let level = gtk4::LevelBar::for_interval(0.0, 100.0);
    level.set_width_request(160);
    let (level_row, level_line) = ui::row("", Some(""), Some(level.upcast_ref()));
    let level_title = first_label(&level_row);
    rows.append(&level_row);
    let (health, health_line) = ui::row(text.health, Some(""), None);
    let health_title = first_label(&health);
    rows.append(&health);
    let shown = if source.is_sample() { true } else { config::load_battery_percentage() };
    let (switch_row, switch) = ui::switch_row(text.show_percentage, Some(text.show_percentage_body), shown);
    let sample = source.is_sample();
    switch.connect_active_notify(move |switch| {
        let on = switch.is_active();
        if !sample {
            gio::spawn_blocking(move || {
                if let Err(error) = config::save_battery_percentage(on) {
                    eprintln!("settings-power: shell.json not saved: {error}");
                }
            });
        }
    });
    rows.append(&switch_row);
    card.append(&rows);
    let empty = ui::empty_state("battery-missing-symbolic", text.no_battery, text.no_battery_body);
    card.append(&empty);
    Battery { block, rows, empty, level, level_title, level_line, health, health_line, health_title }
}

fn first_label(row: &gtk4::Box) -> gtk4::Label {
    row.first_child()
        .and_then(|text| text.first_child())
        .and_then(|label| label.downcast::<gtk4::Label>().ok())
        .unwrap_or_default()
}

fn choices(text: &Text, current: Option<u32>) -> (Vec<Option<u32>>, Vec<String>, u32) {
    let mut values: Vec<Option<u32>> = CHOICES.to_vec();
    if !values.contains(&current) {
        values.insert(values.len() - 1, current);
    }
    let names = values.iter().map(|value| format::duration(text, *value)).collect();
    let selected = values.iter().position(|value| *value == current).unwrap_or(0) as u32;
    (values, names, selected)
}

impl Page {
    fn timeout_row(self: &Rc<Self>, title: &str, subtitle: Option<&str>, current: Option<u32>, apply: fn(&mut PowerConfig, Option<u32>)) -> gtk4::Box {
        let (values, names, selected) = choices(self.text, current);
        let (row, dropdown) = ui::choice_row(title, subtitle, &names, selected);
        let page = Rc::downgrade(self);
        dropdown.connect_selected_notify(move |dropdown| {
            let (Some(page), Some(value)) = (page.upgrade(), values.get(dropdown.selected() as usize)) else {
                return;
            };
            apply(&mut page.config.borrow_mut(), *value);
            page.save();
        });
        row
    }

    fn lid_row(self: &Rc<Self>) -> gtk4::Box {
        let names = [self.text.lid_suspend.to_string(), self.text.lid_nothing.to_string()];
        let selected = u32::from(self.config.borrow().lid == Lid::Nothing);
        let (row, dropdown) = ui::choice_row(self.text.lid, None, &names, selected);
        let page = Rc::downgrade(self);
        dropdown.connect_selected_notify(move |dropdown| {
            if let Some(page) = page.upgrade() {
                page.config.borrow_mut().lid = if dropdown.selected() == 1 { Lid::Nothing } else { Lid::Suspend };
                page.save();
            }
        });
        row
    }

    fn save(self: &Rc<Self>) {
        if self.source.is_sample() {
            return;
        }
        let config = self.config.borrow().clone();
        let page = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let saved = gio::spawn_blocking(move || {
                config::save(&config)?;
                std::process::Command::new("systemctl")
                    .args(["--user", "try-restart", "flux-idle.service"])
                    .status()
                    .map(|_| ())
            })
            .await;
            let failed = !matches!(saved, Ok(Ok(())));
            if let Some(page) = page.upgrade() {
                page.error.set_visible(failed);
            }
        });
    }

    fn sync(&self, snapshot: &Snapshot) {
        let battery = &self.battery;
        battery.rows.set_visible(snapshot.battery.is_some());
        battery.empty.set_visible(snapshot.battery.is_none());
        self.suspend_battery.set_visible(snapshot.battery.is_some());
        self.lid.set_visible(snapshot.lid);
        if let Some(level) = &snapshot.battery {
            battery.level.set_value(level.percentage.clamp(0.0, 100.0));
            battery.level_title.set_text(&format::percent(level.percentage));
            battery.level_line.set_text(&format::battery_line(self.text, level));
            battery.health.set_visible(true);
            match (level.health, level.cycles) {
                (None, None) => {
                    battery.health_title.set_text(self.text.health);
                    battery.health_line.set_text(self.text.health_unknown);
                    battery.health_line.set_visible(true);
                }
                (health, cycles) => {
                    battery.health_title.set_text(&match health {
                        Some(health) => format!("{} · {}", self.text.health, format::percent(health)),
                        None => self.text.health.to_string(),
                    });
                    battery.health_line.set_text(&cycles.map(|cycles| format!("{cycles} {}", self.text.cycles)).unwrap_or_default());
                    battery.health_line.set_visible(cycles.is_some());
                }
            }
        }
        self.modes.sync(snapshot);
        self.mode_missing.set_visible(snapshot.profiles.is_empty());
    }
}

fn mode_block(text: &Text, modes: &Modes) -> (gtk4::Box, gtk4::Box) {
    let (block, card) = ui::section(text.mode, Some(text.mode_body));
    let holder = gtk4::Box::new(Orientation::Vertical, 0);
    holder.add_css_class("fx-row");
    holder.append(&modes.root);
    card.append(&holder);
    let (missing, _) = ui::row(text.mode_missing, None, None);
    card.append(&missing);
    (block, missing)
}

/// Build the page from `source`.
pub fn build(text: &'static Text, source: Source) -> gtk4::Widget {
    ui::style();
    let column = gtk4::Box::new(Orientation::Vertical, 40);
    column.add_css_class("fx-page");
    column.set_width_request(560);
    column.set_halign(gtk4::Align::Center);
    let title = gtk4::Label::new(Some(text.title));
    title.add_css_class("fx-page-title");
    title.set_xalign(0.0);
    column.append(&title);
    let battery = battery_block(text, &source);
    column.append(&battery.block);
    let modes = Modes::new(text, &source);
    let (modes_block, mode_missing) = mode_block(text, &modes);
    column.append(&modes_block);
    let (idle_block, idle_card) = ui::section(text.idle, Some(text.idle_body));
    let error = gtk4::Label::new(Some(text.save_failed));
    error.add_css_class("fx-danger");
    error.set_xalign(0.0);
    error.set_visible(false);
    let config = if source.is_sample() { PowerConfig::default() } else { config::load() };
    let page = Rc::new(Page {
        text,
        source: source.clone(),
        config: RefCell::new(config.clone()),
        battery,
        modes,
        mode_missing,
        suspend_battery: gtk4::Box::new(Orientation::Vertical, 0),
        lid: gtk4::Box::new(Orientation::Vertical, 0),
        error: error.clone(),
    });
    idle_card.append(&page.timeout_row(text.screen_off, None, config.screen_off, |config, value| config.screen_off = value));
    idle_card.append(&page.timeout_row(text.lock, Some(text.lock_body), config.lock, |config, value| config.lock = value));
    page.suspend_battery.append(&page.timeout_row(text.suspend_battery, None, config.suspend_battery, |config, value| config.suspend_battery = value));
    idle_card.append(&page.suspend_battery);
    idle_card.append(&page.timeout_row(text.suspend_ac, None, config.suspend_ac, |config, value| config.suspend_ac = value));
    page.lid.append(&page.lid_row());
    idle_card.append(&page.lid);
    idle_block.append(&error);
    column.append(&idle_block);
    page.sync(&source.snapshot());
    let weak = Rc::downgrade(&page);
    source.on_change(move |source| {
        if let Some(page) = weak.upgrade() {
            page.sync(&source.snapshot());
        }
    });
    let scroller = gtk4::ScrolledWindow::new();
    scroller.set_hscrollbar_policy(gtk4::PolicyType::Never);
    scroller.set_child(Some(&column));
    scroller.set_vexpand(true);
    let keep = page.clone();
    scroller.connect_destroy(move |_| {
        let _ = &keep;
    });
    scroller.upcast()
}
