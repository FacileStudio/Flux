//! The Displays page: one card per output, the arrangement canvas when there
//! are several, and night light.
//!
//! Safe changes (scale, variable refresh rate, position) are written to
//! `outputs.kdl` at once and niri reloads them. A change that can leave the
//! screen black (mode, rotation, turning an output off or on) is applied to
//! the session only, with `niri msg output`, behind a 15 second "Keep these
//! settings?" countdown; it is written only when kept, and undone otherwise.

mod arrange;
mod night;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Revealer, RevealerTransitionType, Stack, StackTransitionType};

use crate::backend;
use crate::i18n::{self, fill, Text};
use crate::model::config::{ModeChoice, OutputConfig};
use crate::model::geometry::Rect;
use crate::model::outputs::{
    format_mode, format_refresh, format_resolution, refresh_rates, resolutions, scale_choices, Output,
    Transform,
};
use crate::ui;

pub const ID: &str = "display";
pub const ICON: &str = "video-display-symbolic";
const COUNTDOWN: u32 = 15;

pub fn title() -> String {
    i18n::text().display_title.to_string()
}

/// Runs the latest of a burst of calls once the burst has been quiet for a
/// moment, so a dragged slider writes its file once.
#[derive(Default)]
pub(crate) struct Debounce {
    generation: Rc<Cell<u64>>,
}

impl Debounce {
    pub(crate) fn run(&self, millis: u64, work: impl FnOnce() + 'static) {
        let generation = self.generation.get() + 1;
        self.generation.set(generation);
        let current = self.generation.clone();
        glib::timeout_add_local_once(Duration::from_millis(millis), move || {
            if current.get() == generation {
                work();
            }
        });
    }
}

struct Pending {
    old: OutputConfig,
    new: OutputConfig,
    identity: String,
    left: u32,
    generation: u64,
}

struct Page {
    text: &'static Text,
    stack: Stack,
    content: GtkBox,
    banner: Revealer,
    banner_body: Label,
    error: Label,
    outputs: RefCell<Vec<Output>>,
    pending: RefCell<Option<Pending>>,
    generation: Cell<u64>,
}

fn percent(scale: f64) -> String {
    let value = (scale * 100.0).round();
    if i18n::french() {
        format!("{value}\u{a0}%")
    } else {
        format!("{value}%")
    }
}

impl Page {
    fn show_error(&self, template: &str, message: &str) {
        self.error.set_label(&fill(template, "error", message));
        self.error.set_visible(true);
    }

    fn refresh(self: &Rc<Self>) {
        let this = self.clone();
        backend::spawn(backend::outputs, move |result| match result {
            Ok(list) if !list.is_empty() => {
                *this.outputs.borrow_mut() = list;
                this.rebuild();
                this.stack.set_visible_child_name("content");
                this.preview_keep();
            }
            _ => this.stack.set_visible_child_name("empty"),
        });
    }

    fn preview_keep(self: &Rc<Self>) {
        if backend::preview_scene() != "keep" || self.generation.get() > 0 {
            return;
        }
        let Some(output) = self.outputs.borrow().get(1).cloned() else {
            return;
        };
        let mut config = OutputConfig::from_live(&output);
        config.mode = Some(ModeChoice { width: 2560, height: 1440, refresh: 143998 });
        self.propose(&output, config);
    }

    fn outputs_changed(self: &Rc<Self>) {
        if self.pending.borrow().is_none() {
            self.refresh();
        }
    }

    fn refresh_soon(self: &Rc<Self>) {
        if backend::preview() {
            return;
        }
        let this = self.clone();
        glib::timeout_add_local_once(Duration::from_millis(500), move || this.refresh());
    }

    fn save(self: &Rc<Self>, configs: Vec<(String, OutputConfig)>) {
        let this = self.clone();
        backend::spawn(
            move || backend::save_outputs(move |state| state.outputs.extend(configs)),
            move |result| match result {
                Ok(_) => {
                    this.error.set_visible(false);
                    this.refresh_soon();
                }
                Err(message) => this.show_error(this.text.save_failed, &message),
            },
        );
    }

    fn propose(self: &Rc<Self>, output: &Output, new: OutputConfig) {
        let old = OutputConfig::from_live(output);
        if old == new || self.pending.borrow().is_some() {
            return;
        }
        let generation = self.generation.get() + 1;
        self.generation.set(generation);
        *self.pending.borrow_mut() =
            Some(Pending { old, new: new.clone(), identity: output.identity(), left: COUNTDOWN, generation });
        self.content.set_sensitive(false);
        let this = self.clone();
        backend::spawn(move || backend::apply_live(&new), move |result| match result {
            Ok(()) => {
                this.error.set_visible(false);
                this.start_countdown(generation);
            }
            Err(message) => {
                this.show_error(this.text.apply_failed, &message);
                this.revert();
            }
        });
    }

    fn start_countdown(self: &Rc<Self>, generation: u64) {
        self.update_banner();
        self.banner.set_visible(true);
        self.banner.set_reveal_child(true);
        let this = self.clone();
        glib::timeout_add_local(Duration::from_secs(1), move || {
            let expired = {
                let mut pending = this.pending.borrow_mut();
                match pending.as_mut() {
                    Some(p) if p.generation == generation => {
                        p.left = p.left.saturating_sub(1);
                        p.left == 0
                    }
                    _ => return glib::ControlFlow::Break,
                }
            };
            if expired {
                this.revert();
                return glib::ControlFlow::Break;
            }
            this.update_banner();
            glib::ControlFlow::Continue
        });
    }

    fn update_banner(&self) {
        if let Some(pending) = self.pending.borrow().as_ref() {
            self.banner_body.set_label(&fill(self.text.keep_body, "seconds", &pending.left.to_string()));
        }
    }

    fn finish(self: &Rc<Self>) -> Option<Pending> {
        self.banner.set_reveal_child(false);
        self.content.set_sensitive(true);
        self.pending.borrow_mut().take()
    }

    fn keep(self: &Rc<Self>) {
        if let Some(pending) = self.finish() {
            self.save(vec![(pending.identity, pending.new)]);
        }
    }

    fn revert(self: &Rc<Self>) {
        let Some(pending) = self.finish() else {
            return;
        };
        let this = self.clone();
        backend::spawn(move || backend::apply_live(&pending.old), move |_| this.refresh_soon());
    }

    fn rebuild(self: &Rc<Self>) {
        ui::clear(&self.content);
        let outputs = self.outputs.borrow().clone();
        let enabled: Vec<&Output> = outputs.iter().filter(|output| output.enabled()).collect();
        if enabled.len() > 1 {
            self.content.append(&self.arrangement(&enabled));
        }
        for output in &outputs {
            self.content.append(&self.card(output, enabled.len()));
        }
    }

    fn arrangement(self: &Rc<Self>, enabled: &[&Output]) -> GtkBox {
        let section = ui::section_with(self.text.arrangement, self.text.arrangement_hint);
        let plates: Vec<arrange::Plate> = enabled
            .iter()
            .filter_map(|output| {
                let logical = output.logical?;
                Some(arrange::Plate {
                    identity: output.identity(),
                    label: output.display_name(),
                    rect: Rect::new(logical.x, logical.y, logical.width as i32, logical.height as i32),
                })
            })
            .collect();
        let outputs: Vec<Output> = enabled.iter().map(|output| (*output).clone()).collect();
        let this = self.clone();
        let canvas = arrange::canvas(plates, move |plates| {
            let configs = plates
                .iter()
                .filter_map(|plate| {
                    let output = outputs.iter().find(|output| output.identity() == plate.identity)?;
                    let mut config = OutputConfig::from_live(output);
                    config.position = Some((plate.rect.x, plate.rect.y));
                    Some((plate.identity.clone(), config))
                })
                .collect();
            this.save(configs);
        });
        section.card.append(&canvas);
        section.card.add_css_class("fx-canvas-card");
        section.root
    }

    fn card(self: &Rc<Self>, output: &Output, enabled_count: usize) -> GtkBox {
        let text = self.text;
        let mut about = vec![if output.internal() { text.built_in.to_string() } else { output.name.clone() }];
        if let Some(inches) = output.diagonal_inches() {
            let size = if i18n::french() { inches.to_string().replace('.', ",") } else { inches.to_string() };
            about.push(fill(text.inches, "size", &size));
        }
        if !output.enabled() {
            about.push(text.off.to_string());
        }
        let section = ui::section_with(&output.display_name(), &about.join(" · "));

        if self.outputs.borrow().len() > 1 {
            section.card.append(&self.enable_row(output, enabled_count));
        }
        if let (Some(mode), Some(logical)) = (output.mode(), output.logical) {
            section.card.append(&self.resolution_row(output, mode.width, mode.height));
            section.card.append(&self.refresh_row(output, mode.width, mode.height, mode.refresh_rate));
            let horizontal = if logical.transform.sideways() { mode.height } else { mode.width };
            section.card.append(&self.scale_row(output, logical.scale, horizontal));
            section.card.append(&self.rotation_row(output, logical.transform));
            if output.vrr_supported {
                section.card.append(&self.vrr_row(output));
            }
        }
        section.root
    }

    fn enable_row(self: &Rc<Self>, output: &Output, enabled_count: usize) -> GtkBox {
        let (row, switch) = ui::switch_row(self.text.use_display, self.text.use_display_hint, output.enabled(), |_| {});
        switch.set_sensitive(!(output.enabled() && enabled_count <= 1));
        let this = self.clone();
        let output = output.clone();
        switch.connect_active_notify(move |switch| {
            let mut config = OutputConfig::from_live(&output);
            config.off = !switch.is_active();
            if !config.off {
                config.mode = output.preferred().map(|m| ModeChoice { width: m.width, height: m.height, refresh: m.refresh_rate });
                config.scale = None;
            }
            this.propose(&output, config);
        });
        row
    }

    fn resolution_row(self: &Rc<Self>, output: &Output, width: u16, height: u16) -> GtkBox {
        let list = resolutions(&output.modes);
        let at = self.text.at;
        let names: Vec<String> = list
            .iter()
            .map(|r| {
                let name = format_resolution(r.width, r.height);
                if r.preferred {
                    fill(self.text.preferred, "mode", &name)
                } else {
                    name
                }
            })
            .collect();
        let selected = list.iter().position(|r| r.width == width && r.height == height).unwrap_or(0);
        let (row, dropdown) = ui::choice_row(self.text.resolution, "", &names, selected, |_| {});
        if let Some(mode) = output.mode() {
            dropdown.set_tooltip_text(Some(&format_mode(&mode, at)));
        }
        let this = self.clone();
        let output = output.clone();
        dropdown.connect_selected_notify(move |dropdown| {
            let Some(choice) = list.get(dropdown.selected() as usize) else {
                return;
            };
            let rates = refresh_rates(&output.modes, choice.width, choice.height);
            let pick = rates.iter().find(|m| m.is_preferred).or_else(|| rates.first()).copied();
            let Some(mode) = pick else {
                return;
            };
            let mut config = OutputConfig::from_live(&output);
            config.mode = Some(ModeChoice { width: mode.width, height: mode.height, refresh: mode.refresh_rate });
            this.propose(&output, config);
        });
        row
    }

    fn refresh_row(self: &Rc<Self>, output: &Output, width: u16, height: u16, current: u32) -> GtkBox {
        let rates = refresh_rates(&output.modes, width, height);
        let names: Vec<String> = rates.iter().map(|m| format!("{} Hz", format_refresh(m.refresh_rate))).collect();
        let selected = rates.iter().position(|m| m.refresh_rate == current).unwrap_or(0);
        let (row, dropdown) = ui::choice_row(self.text.refresh, "", &names, selected, |_| {});
        dropdown.set_sensitive(rates.len() > 1);
        let this = self.clone();
        let output = output.clone();
        dropdown.connect_selected_notify(move |dropdown| {
            let Some(mode) = rates.get(dropdown.selected() as usize) else {
                return;
            };
            let mut config = OutputConfig::from_live(&output);
            config.mode = Some(ModeChoice { width: mode.width, height: mode.height, refresh: mode.refresh_rate });
            this.propose(&output, config);
        });
        row
    }

    fn scale_row(self: &Rc<Self>, output: &Output, current: f64, horizontal: u16) -> GtkBox {
        let choices = scale_choices(Some(current), horizontal);
        let names: Vec<String> = choices.iter().map(|scale| percent(*scale)).collect();
        let selected = choices.iter().position(|s| (s - current).abs() < 0.005).unwrap_or(0);
        let (row, dropdown) = ui::choice_row(self.text.scale, self.text.scale_hint, &names, selected, |_| {});
        let this = self.clone();
        let output = output.clone();
        dropdown.connect_selected_notify(move |dropdown| {
            let Some(scale) = choices.get(dropdown.selected() as usize).copied() else {
                return;
            };
            let mut config = OutputConfig::from_live(&output);
            config.scale = Some(scale);
            this.save(vec![(output.identity(), config)]);
        });
        row
    }

    fn rotation_row(self: &Rc<Self>, output: &Output, current: Transform) -> GtkBox {
        let names: Vec<String> = self.text.rotations.iter().map(|name| name.to_string()).collect();
        let selected = Transform::ALL.iter().position(|t| *t == current).unwrap_or(0);
        let (row, dropdown) = ui::choice_row(self.text.rotation, "", &names, selected, |_| {});
        let this = self.clone();
        let output = output.clone();
        dropdown.connect_selected_notify(move |dropdown| {
            let Some(transform) = Transform::ALL.get(dropdown.selected() as usize).copied() else {
                return;
            };
            let mut config = OutputConfig::from_live(&output);
            config.transform = transform;
            this.propose(&output, config);
        });
        row
    }

    fn vrr_row(self: &Rc<Self>, output: &Output) -> GtkBox {
        let (row, switch) = ui::switch_row(self.text.vrr, self.text.vrr_hint, output.vrr_enabled, |_| {});
        let this = self.clone();
        let output = output.clone();
        switch.connect_active_notify(move |switch| {
            let mut config = OutputConfig::from_live(&output);
            config.vrr = switch.is_active();
            this.save(vec![(output.identity(), config)]);
        });
        row
    }
}

fn banner(text: &'static Text) -> (Revealer, Label, gtk4::Button, gtk4::Button) {
    let body = Label::builder().xalign(0.0).css_classes(["fx-subtitle"]).build();
    let words = GtkBox::new(Orientation::Vertical, 2);
    words.set_hexpand(true);
    words.append(&Label::builder().label(text.keep_title).xalign(0.0).css_classes(["fx-title"]).build());
    words.append(&body);
    let revert = ui::ghost_button(text.revert);
    let keep = ui::primary_button(text.keep);
    let bar = GtkBox::new(Orientation::Horizontal, 8);
    bar.add_css_class("fx-banner");
    bar.append(&words);
    bar.append(&revert);
    bar.append(&keep);
    let revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideDown)
        .transition_duration(180)
        .child(&bar)
        .visible(false)
        .build();
    revealer.connect_child_revealed_notify(|revealer| {
        if !revealer.reveals_child() && !revealer.is_child_revealed() {
            revealer.set_visible(false);
        }
    });
    (revealer, body, revert, keep)
}

fn empty(text: &'static Text, page: &Rc<Page>) -> GtkBox {
    let state = ui::empty_state(ICON, text.no_niri_title, text.no_niri_body);
    let retry = ui::ghost_button(text.retry);
    retry.set_halign(gtk4::Align::Center);
    let this = page.clone();
    retry.connect_clicked(move |_| {
        this.stack.set_visible_child_name("loading");
        this.refresh();
    });
    state.append(&retry);
    state
}

/// The whole Displays page.
pub fn page() -> gtk4::Widget {
    let text = i18n::text();
    let (scroller, column) = ui::display_page(text.display_title, text.display_description);
    let (banner, banner_body, revert, keep) = banner(text);
    let error = Label::builder().xalign(0.0).wrap(true).visible(false).css_classes(["fx-error"]).build();
    let content = GtkBox::new(Orientation::Vertical, 32);
    let stack = Stack::builder()
        .transition_type(StackTransitionType::Crossfade)
        .transition_duration(160)
        .vhomogeneous(false)
        .build();
    stack.add_named(&ui::spinner_row(text.loading), Some("loading"));
    stack.add_named(&content, Some("content"));
    let page = Rc::new(Page {
        text,
        stack: stack.clone(),
        content,
        banner: banner.clone(),
        banner_body,
        error: error.clone(),
        outputs: RefCell::new(Vec::new()),
        pending: RefCell::new(None),
        generation: Cell::new(0),
    });
    stack.add_named(&empty(text, &page), Some("empty"));
    stack.set_visible_child_name("loading");
    let this = page.clone();
    keep.connect_clicked(move |_| this.keep());
    let this = page.clone();
    revert.connect_clicked(move |_| this.revert());

    column.append(&banner);
    column.append(&error);
    column.append(&stack);
    let night_slot = GtkBox::new(Orientation::Vertical, 0);
    column.append(&night_slot);
    backend::spawn(
        || (backend::load_state().night, backend::coordinates().is_some()),
        move |(night_light, located)| night_slot.append(&night::section(text, night_light, located)),
    );
    let weak = Rc::downgrade(&page);
    scroller.connect_map(move |_| {
        if let Some(page) = weak.upgrade() {
            page.outputs_changed();
        }
    });
    if let Some(display) = gtk4::gdk::Display::default() {
        let weak = Rc::downgrade(&page);
        display.monitors().connect_items_changed(move |_, _, _, _| {
            let weak = weak.clone();
            glib::timeout_add_local_once(Duration::from_millis(800), move || {
                if let Some(page) = weak.upgrade() {
                    page.outputs_changed();
                }
            });
        });
    }
    scroller.upcast()
}
