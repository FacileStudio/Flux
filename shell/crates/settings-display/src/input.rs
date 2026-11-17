//! The Keyboard and mouse page: layouts, repeat, Num Lock, touchpad and
//! mouse. Every change is written to `input.kdl` and niri reloads it live.

mod layouts;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};

use crate::backend;
use crate::display::Debounce;
use crate::i18n::{self, Text};
use crate::model::config::{AccelProfile, InputConfig, Layout, ScrollMethod};
use crate::model::xkb::{switch_option, Registry, SWITCHES};
use crate::ui;

pub const ID: &str = "input";
pub const ICON: &str = "input-keyboard-symbolic";

pub fn title() -> String {
    i18n::text().input_title.to_string()
}

/// Where a scroll direction sits in the two-option choice: Natural first, then Traditional.
pub fn direction_index(natural: bool) -> usize {
    usize::from(!natural)
}

/// Whether the scroll direction at `index` of the choice is Natural.
pub fn natural_at(index: usize) -> bool {
    index == 0
}

pub(crate) struct Input {
    text: &'static Text,
    config: RefCell<InputConfig>,
    system: (Vec<Layout>, String),
    registry: Option<Registry>,
    debounce: Debounce,
    error: Label,
    layout_rows: GtkBox,
    switch: RefCell<Option<gtk4::DropDown>>,
}

impl Input {
    fn layouts(&self) -> Vec<Layout> {
        let config = self.config.borrow();
        if config.layouts.is_empty() {
            self.system.0.clone()
        } else {
            config.layouts.clone()
        }
    }

    fn switch_value(&self) -> String {
        let config = self.config.borrow();
        if config.layouts.is_empty() && config.switch.is_empty() {
            switch_option(&self.system.1)
        } else {
            config.switch.clone()
        }
    }

    fn change(self: &Rc<Self>, change: impl FnOnce(&mut InputConfig)) {
        change(&mut self.config.borrow_mut());
        let this = self.clone();
        self.debounce.run(300, move || {
            let input = this.config.borrow().clone();
            let error = this.error.clone();
            backend::spawn(move || backend::save_input(move |state| state.input = input), move |result| {
                error.set_visible(result.is_err());
                if let Err(message) = result {
                    error.set_label(&i18n::fill(i18n::text().save_failed, "error", &message));
                }
            });
        });
    }

    fn materialize(&self, config: &mut InputConfig) {
        if config.layouts.is_empty() {
            config.layouts = self.system.0.clone();
            if config.switch.is_empty() {
                config.switch = switch_option(&self.system.1);
            }
        }
    }

    fn change_layouts(self: &Rc<Self>, change: impl FnOnce(&mut Vec<Layout>)) {
        let mut config = self.config.borrow().clone();
        self.materialize(&mut config);
        change(&mut config.layouts);
        let count = config.layouts.len();
        self.change(move |current| *current = config);
        layouts::fill(self, &self.layout_rows);
        if let Some(dropdown) = self.switch.borrow().as_ref() {
            dropdown.set_sensitive(count > 1);
        }
    }

    fn switch_row(self: &Rc<Self>) -> GtkBox {
        let mut values = vec![String::new()];
        let mut names = vec![self.text.switch_none.to_string()];
        match self.registry.as_ref().filter(|registry| !registry.switches.is_empty()) {
            Some(registry) => {
                values.extend(registry.switches.iter().map(|item| item.name.clone()));
                names.extend(registry.switches.iter().map(|item| item.description.clone()));
            }
            None => {
                values.extend(SWITCHES.iter().map(|name| name.to_string()));
                names.extend(SWITCHES.iter().map(|name| name.to_string()));
            }
        }
        let current = self.switch_value();
        if !current.is_empty() && !values.contains(&current) {
            values.push(current.clone());
            names.push(current.clone());
        }
        let selected = values.iter().position(|value| *value == current).unwrap_or(0);
        let (row, dropdown) = ui::choice_row(self.text.switch_keys, self.text.switch_hint, &names, selected, |_| {});
        dropdown.set_sensitive(self.layouts().len() > 1);
        let this = self.clone();
        dropdown.connect_selected_notify(move |dropdown| {
            let value = values.get(dropdown.selected() as usize).cloned().unwrap_or_default();
            let mut config = this.config.borrow().clone();
            this.materialize(&mut config);
            config.switch = value;
            this.change(move |current| *current = config);
        });
        *self.switch.borrow_mut() = Some(dropdown);
        row
    }

    fn switch(self: &Rc<Self>, title: &str, subtitle: &str, on: bool, set: fn(&mut InputConfig, bool)) -> GtkBox {
        let (row, switch) = ui::switch_row(title, subtitle, on, |_| {});
        let this = self.clone();
        switch.connect_active_notify(move |switch| {
            let active = switch.is_active();
            this.change(move |config| set(config, active));
        });
        row
    }

    fn slider(self: &Rc<Self>, title: &str, range: (f64, f64, f64), value: f64, set: fn(&mut InputConfig, f64)) -> GtkBox {
        let (row, scale) = ui::slider_row(title, "", range, value, |_| {});
        let this = self.clone();
        scale.connect_value_changed(move |scale| {
            let value = scale.value();
            this.change(move |config| set(config, value));
        });
        row
    }

    fn choice(self: &Rc<Self>, title: &str, subtitle: &str, names: &[&str], selected: usize, set: fn(&mut InputConfig, usize)) -> GtkBox {
        let names: Vec<String> = names.iter().map(|name| name.to_string()).collect();
        let (row, dropdown) = ui::choice_row(title, subtitle, &names, selected, |_| {});
        let this = self.clone();
        dropdown.connect_selected_notify(move |dropdown| {
            let index = dropdown.selected() as usize;
            this.change(move |config| set(config, index));
        });
        row
    }

    fn keyboard(self: &Rc<Self>) -> GtkBox {
        let text = self.text;
        let config = self.config.borrow().clone();
        let layouts = ui::section_with(text.layouts, text.layouts_hint);
        layouts.root.remove(&layouts.card);
        layouts.root.append(&self.layout_rows);
        self.layout_rows.add_css_class("fx-card");
        layouts::fill(self, &self.layout_rows);

        let keyboard = ui::section_with(text.keyboard, "");
        keyboard.card.append(&self.switch_row());
        keyboard.card.append(&self.slider(
            text.repeat_delay,
            (150.0, 1000.0, 50.0),
            f64::from(config.repeat_delay),
            |c, v| c.repeat_delay = v.round() as u32,
        ));
        keyboard.card.append(&self.slider(
            text.repeat_rate,
            (10.0, 60.0, 1.0),
            f64::from(config.repeat_rate),
            |c, v| c.repeat_rate = v.round() as u32,
        ));
        keyboard.card.append(&self.switch(text.numlock, text.numlock_hint, config.numlock, |c, on| c.numlock = on));
        let test = gtk4::Entry::builder().placeholder_text(text.test_keyboard).hexpand(true).build();
        test.add_css_class("fx-test");
        keyboard.card.append(&test);

        let both = GtkBox::new(Orientation::Vertical, 32);
        both.append(&layouts.root);
        both.append(&keyboard.root);
        both
    }

    fn touchpad(self: &Rc<Self>) -> GtkBox {
        let text = self.text;
        let config = self.config.borrow().clone();
        let section = ui::section_with(text.touchpad, "");
        section.card.append(&self.choice(
            text.scroll_direction,
            text.scroll_touchpad_hint,
            &text.scroll_directions,
            direction_index(config.touchpad_natural),
            |c, i| c.touchpad_natural = natural_at(i),
        ));
        section.card.append(&self.switch(text.tap, text.tap_hint, config.tap, |c, on| c.tap = on));
        section.card.append(&self.switch(text.dwt, text.dwt_hint, config.dwt, |c, on| c.dwt = on));
        section.card.append(&self.slider(text.speed, (-1.0, 1.0, 0.1), config.touchpad_speed, |c, v| {
            c.touchpad_speed = (v * 10.0).round() / 10.0
        }));
        let selected = ScrollMethod::ALL.iter().position(|m| *m == config.scroll).unwrap_or(0);
        section.card.append(&self.choice(text.scroll_method, "", &text.scroll_methods, selected, |c, i| {
            c.scroll = ScrollMethod::ALL.get(i).copied().unwrap_or_default()
        }));
        section.root
    }

    fn mouse(self: &Rc<Self>) -> GtkBox {
        let text = self.text;
        let config = self.config.borrow().clone();
        let section = ui::section_with(text.mouse, "");
        section.card.append(&self.choice(
            text.scroll_direction,
            text.scroll_mouse_hint,
            &text.scroll_directions,
            direction_index(config.mouse_natural),
            |c, i| c.mouse_natural = natural_at(i),
        ));
        section.card.append(&self.slider(text.speed, (-1.0, 1.0, 0.1), config.mouse_speed, |c, v| {
            c.mouse_speed = (v * 10.0).round() / 10.0
        }));
        let selected = usize::from(config.mouse_profile == AccelProfile::Flat);
        section.card.append(&self.choice(text.acceleration, "", &text.accel_profiles, selected, |c, i| {
            c.mouse_profile = if i == 1 { AccelProfile::Flat } else { AccelProfile::Adaptive }
        }));
        section.root
    }
}

struct Loaded {
    config: InputConfig,
    system: (Vec<Layout>, String),
    registry: Option<Registry>,
    touchpad: bool,
}

fn load() -> Loaded {
    Loaded {
        config: backend::load_state().input,
        system: backend::system_keyboard(),
        registry: backend::registry(),
        touchpad: backend::touchpad(),
    }
}

/// The whole Keyboard and mouse page.
pub fn page() -> gtk4::Widget {
    let text = i18n::text();
    let (scroller, column) = ui::display_page(text.input_title, text.input_description);
    let target = GtkBox::new(Orientation::Vertical, 32);
    target.append(&ui::spinner_row(text.loading_input));
    column.append(&target);
    backend::spawn(load, move |loaded| {
        let error = Label::builder().xalign(0.0).wrap(true).visible(false).css_classes(["fx-error"]).build();
        let input = Rc::new(Input {
            text,
            config: RefCell::new(loaded.config),
            system: loaded.system,
            registry: loaded.registry,
            debounce: Debounce::default(),
            error: error.clone(),
            layout_rows: GtkBox::new(Orientation::Vertical, 0),
            switch: RefCell::new(None),
        });
        ui::clear(&target);
        target.append(&error);
        if backend::preview_scene() != "pointer" {
            target.append(&input.keyboard());
        }
        if loaded.touchpad {
            target.append(&input.touchpad());
        }
        target.append(&input.mouse());
    });
    scroller.upcast()
}
