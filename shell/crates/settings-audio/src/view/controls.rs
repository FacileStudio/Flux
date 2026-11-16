//! The volume control: mute button, slider, value and optional live meter.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use gtk4::prelude::*;
use gtk4::{
    glib, Align, Box as GtkBox, Label, LevelBar, Orientation, PositionType, Scale, TickCallbackId,
    ToggleButton,
};

use crate::i18n::Text;
use crate::{model, motion};

const HANDS_OFF: Duration = Duration::from_millis(500);

/// What a person did to a volume control.
#[derive(Clone, Copy, Debug)]
pub enum Change {
    Volume(u32),
    Mute(bool),
}

#[derive(Default)]
struct Glide {
    from: f64,
    to: f64,
    start: i64,
    tick: Option<TickCallbackId>,
}

/// A mute button, a slider, the value, and optionally a live meter under the slider.
pub struct VolumeControl {
    pub root: GtkBox,
    mute: ToggleButton,
    scale: Scale,
    value: Label,
    meter: Option<LevelBar>,
    guard: Rc<Cell<bool>>,
    touched: Rc<Cell<Option<Instant>>>,
    glide: Rc<RefCell<Glide>>,
    level: Cell<f64>,
    cap: Cell<u32>,
    text: &'static Text,
}

impl VolumeControl {
    pub fn new(
        text: &'static Text,
        label: &str,
        with_meter: bool,
        on_change: impl Fn(Change) + 'static,
    ) -> Rc<Self> {
        let root = GtkBox::new(Orientation::Vertical, 4);
        root.add_css_class("fx-volume");
        let (line, mute, scale, value) = slider(text, label);
        root.append(&line);
        let meter = with_meter.then(|| meter(&root));
        let control = Rc::new(VolumeControl {
            root,
            mute,
            scale,
            value,
            meter,
            guard: Rc::new(Cell::new(false)),
            touched: Rc::new(Cell::new(None)),
            glide: Rc::new(RefCell::new(Glide::default())),
            level: Cell::new(0.0),
            cap: Cell::new(100),
            text,
        });
        control.connect(Rc::new(on_change));
        control
    }

    fn connect(self: &Rc<Self>, on_change: Rc<dyn Fn(Change)>) {
        let weak = Rc::downgrade(self);
        let changed = Rc::clone(&on_change);
        self.scale.connect_value_changed(move |scale| {
            let Some(control) = weak.upgrade().filter(|control| !control.guard.get()) else {
                return;
            };
            control.touched.set(Some(Instant::now()));
            let percent = scale.value().round().max(0.0) as u32;
            control.show(percent, control.mute.is_active());
            changed(Change::Volume(model::raw(percent)));
        });
        let weak = Rc::downgrade(self);
        self.mute.connect_toggled(move |button| {
            let Some(control) = weak.upgrade().filter(|control| !control.guard.get()) else {
                return;
            };
            control.show(control.scale.value().round() as u32, button.is_active());
            on_change(Change::Mute(button.is_active()));
        });
    }

    fn show(&self, percent: u32, muted: bool) {
        self.value.set_text(&self.text.percent(percent));
        self.mute.set_icon_name(model::volume_icon(percent, muted));
        self.mute.set_tooltip_text(Some(if muted {
            self.text.unmute
        } else {
            self.text.mute
        }));
        if muted {
            self.root.add_css_class("muted");
        } else {
            self.root.remove_css_class("muted");
        }
    }

    /// Show the server's volume, gliding the slider there unless the person is moving it.
    pub fn set(&self, raw: u32, muted: bool, cap: u32) {
        let percent = model::percent(raw);
        let top = cap.max(percent);
        if self.cap.replace(top) != top || self.scale.adjustment().upper() as u32 != top {
            self.guard.set(true);
            self.scale.set_range(0.0, f64::from(top));
            self.scale.clear_marks();
            if top > 100 {
                self.scale.add_mark(100.0, PositionType::Bottom, None);
            }
            self.guard.set(false);
        }
        self.guard.set(true);
        self.mute.set_active(muted);
        self.guard.set(false);
        let busy = self
            .touched
            .get()
            .is_some_and(|at| at.elapsed() < HANDS_OFF);
        if busy {
            return;
        }
        self.show(percent, muted);
        self.glide_to(f64::from(percent));
    }

    fn glide_to(&self, target: f64) {
        let current = self.scale.value();
        let mut glide = self.glide.borrow_mut();
        let clock = self.scale.frame_clock();
        let (Some(clock), true) = (
            clock,
            motion::enabled() && self.scale.is_mapped() && (current - target).abs() >= 1.0,
        ) else {
            if let Some(tick) = glide.tick.take() {
                tick.remove();
            }
            self.guard.set(true);
            self.scale.set_value(target);
            self.guard.set(false);
            return;
        };
        glide.from = current;
        glide.to = target;
        glide.start = clock.frame_time();
        if glide.tick.is_some() {
            return;
        }
        let state = Rc::clone(&self.glide);
        let guard = Rc::clone(&self.guard);
        let span = f64::from(motion::DURATION) * 1000.0;
        glide.tick = Some(self.scale.add_tick_callback(move |scale, clock| {
            let mut glide = state.borrow_mut();
            let progress = (clock.frame_time() - glide.start) as f64 / span;
            let value = glide.from + (glide.to - glide.from) * motion::ease_out(progress);
            guard.set(true);
            scale.set_value(value);
            guard.set(false);
            if progress >= 1.0 {
                glide.tick = None;
                return glib::ControlFlow::Break;
            }
            glib::ControlFlow::Continue
        }));
    }

    /// Feed a peak (linear, 0 to 1) to the meter.
    pub fn level(&self, peak: f32) {
        let Some(meter) = &self.meter else {
            return;
        };
        let next = model::smooth(self.level.get(), model::meter(peak));
        self.level.set(next);
        meter.set_value(next);
    }

    pub fn set_sensitive(&self, sensitive: bool) {
        self.root.set_sensitive(sensitive);
        if !sensitive {
            self.level(0.0);
        }
    }
}

fn meter(root: &GtkBox) -> LevelBar {
    let meter = LevelBar::for_interval(0.0, 1.0);
    meter.add_css_class("fx-meter");
    for name in ["low", "high", "full"] {
        meter.remove_offset_value(Some(name));
    }
    meter.set_margin_start(44);
    meter.set_margin_end(72);
    root.append(&meter);
    meter
}

fn slider(text: &Text, label: &str) -> (GtkBox, ToggleButton, Scale, Label) {
    let line = GtkBox::new(Orientation::Horizontal, 12);
    let mute = ToggleButton::new();
    mute.add_css_class("fx-icon-button");
    mute.set_icon_name(model::volume_icon(0, false));
    mute.set_valign(Align::Center);
    let scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    scale.set_draw_value(false);
    scale.set_hexpand(true);
    scale.set_increments(1.0, 5.0);
    scale.update_property(&[gtk4::accessible::Property::Label(label)]);
    let value = Label::new(Some(&text.percent(0)));
    value.add_css_class("fx-value");
    value.set_width_chars(5);
    value.set_xalign(1.0);
    line.append(&mute);
    line.append(&scale);
    line.append(&value);
    (line, mute, scale, value)
}
