//! The night light section: a switch, the warmth, and when it runs.

use std::cell::Cell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Entry, Label, Revealer, RevealerTransitionType};

use crate::backend;
use crate::i18n::Text;
use crate::model::config::{NightLight, Schedule};
use crate::model::night::{format_time, parse_time, COOLEST, FALLBACK_HOURS, WARMEST};
use crate::ui;

use super::Debounce;

struct Night {
    value: Cell<NightLight>,
    debounce: Debounce,
    error: Label,
}

impl Night {
    fn change(self: &Rc<Self>, change: impl FnOnce(&mut NightLight)) {
        let mut value = self.value.get();
        change(&mut value);
        self.value.set(value);
        let this = self.clone();
        self.debounce.run(400, move || {
            let value = this.value.get();
            let error = this.error.clone();
            backend::spawn(move || backend::apply_night(value), move |result| {
                error.set_visible(result.is_err());
                if let Err(message) = result {
                    error.set_label(&message);
                }
            });
        });
    }
}

fn time_entry(time: (u8, u8), label: &str) -> Entry {
    let entry = Entry::builder().text(format_time(time)).max_length(5).width_chars(5).max_width_chars(5).build();
    entry.update_property(&[gtk4::accessible::Property::Label(label)]);
    entry
}

fn manual_rows(night: &Rc<Night>, text: &'static Text) -> gtk4::Box {
    let (from, to) = match night.value.get().schedule {
        Schedule::Manual { from, to } => (from, to),
        Schedule::Sun => FALLBACK_HOURS,
    };
    let rows = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    for (label, time, is_from) in [(text.from, from, true), (text.to, to, false)] {
        let entry = time_entry(time, label);
        let this = night.clone();
        entry.connect_changed(move |entry| {
            let Some(time) = parse_time(&entry.text()) else {
                entry.add_css_class("invalid");
                return;
            };
            entry.remove_css_class("invalid");
            this.change(|value| {
                let (mut from, mut to) = match value.schedule {
                    Schedule::Manual { from, to } => (from, to),
                    Schedule::Sun => FALLBACK_HOURS,
                };
                if is_from {
                    from = time;
                } else {
                    to = time;
                }
                value.schedule = Schedule::Manual { from, to };
            });
        });
        rows.append(&ui::row(label, "", &entry));
    }
    rows
}

/// The night light section, loaded from `initial`.
pub fn section(text: &'static Text, initial: NightLight, located: bool) -> gtk4::Box {
    let section = ui::section_with(text.night, "");
    let error = Label::builder().xalign(0.0).wrap(true).visible(false).css_classes(["fx-error"]).build();
    let night = Rc::new(Night { value: Cell::new(initial), debounce: Debounce::default(), error: error.clone() });

    let (switch_row, switch) = ui::switch_row(text.night_switch, text.night_hint, initial.enabled, |_| {});
    let details = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideDown)
        .transition_duration(160)
        .reveal_child(initial.enabled)
        .build();
    let body = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    details.set_child(Some(&body));

    let (warm_row, warmth) =
        ui::slider_row(text.warmth, "", (f64::from(WARMEST), f64::from(COOLEST), 100.0), f64::from(initial.temperature), |_| {});
    warmth.set_inverted(true);
    body.append(&warm_row);

    let sun = if located { text.sun } else { text.sun_unknown };
    let manual_now = matches!(initial.schedule, Schedule::Manual { .. });
    let (schedule_row, schedule) =
        ui::choice_row(text.schedule, "", &[sun.to_string(), text.manual.to_string()], usize::from(manual_now), |_| {});
    body.append(&schedule_row);
    let manual = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideDown)
        .transition_duration(160)
        .reveal_child(manual_now)
        .child(&manual_rows(&night, text))
        .build();
    body.append(&manual);

    let this = night.clone();
    let revealer = details.clone();
    switch.connect_active_notify(move |switch| {
        let on = switch.is_active();
        revealer.set_reveal_child(on);
        this.change(|value| value.enabled = on);
    });
    let this = night.clone();
    warmth.connect_value_changed(move |scale| {
        let kelvin = (scale.value() / 100.0).round() as u32 * 100;
        this.change(|value| value.temperature = kelvin);
    });
    let this = night.clone();
    schedule.connect_selected_notify(move |dropdown| {
        let custom = dropdown.selected() == 1;
        manual.set_reveal_child(custom);
        this.change(|value| {
            value.schedule = match (custom, value.schedule) {
                (false, _) => Schedule::Sun,
                (true, Schedule::Manual { from, to }) => Schedule::Manual { from, to },
                (true, Schedule::Sun) => Schedule::Manual { from: FALLBACK_HOURS.0, to: FALLBACK_HOURS.1 },
            };
        });
    });

    section.card.append(&switch_row);
    section.card.append(&details);
    section.root.append(&error);
    section.root
}
