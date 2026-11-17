//! One bar: its window on one monitor, the modules on it and the timers that
//! keep them current.
//!
//! Left: the apps button, the workspace chips of this monitor's output, then
//! the close button and the title of the window in front. Middle: the date
//! and time, which open the sidebar. Right: CPU and memory, the keyboard
//! layout when there is more than one, and the control row.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::accessible::Property;
use gtk4::prelude::*;
use gtk4::{gdk, glib, Application, Box, Button, CenterBox, Label};
use gtk4_layer_shell::Layer;

use shell_common::layer::LayerWindow;
use shell_common::spawn;
use shell_common::theme::{BAR_HEIGHT, MODULE_GAP};

use crate::niri::{self, Chip, State};
use crate::popup::Popup;
use crate::{control, icons, stats, text, volume};

/// The longest window title the bar shows, in characters, before it ellipsizes.
const TITLE_CHARS: i32 = 48;

/// One monitor's bar.
pub struct Bar {
    pub monitor: gdk::Monitor,
    window: gtk4::ApplicationWindow,
    popup: Popup,
    workspaces: Box,
    chips: RefCell<Vec<(u64, Button)>>,
    front: Rc<Cell<Option<u64>>>,
    close: Button,
    title: Label,
    layout: Button,
}

impl Bar {
    /// Build and show the bar for `monitor`.
    pub fn new(app: &Application, monitor: &gdk::Monitor, painted: &Rc<RefCell<icons::Painted>>) -> Rc<Self> {
        let layer = LayerWindow::new(app).layer(Layer::Top);
        layer.anchors(true, true, true, false);
        layer.exclusive_zone(true);
        layer.monitor(Some(monitor));
        layer.window.set_height_request(BAR_HEIGHT);

        let popup = Popup::new(app, Some(monitor));
        let root = CenterBox::new();
        root.add_css_class("bar");

        let front = Rc::new(Cell::new(None));
        let close = close_button(painted, &front);
        let (left, workspaces, title) = left_module(painted, &close);
        let layout = layout_pill();
        let right = Box::new(gtk4::Orientation::Horizontal, MODULE_GAP);
        right.set_halign(gtk4::Align::End);
        right.append(&stats_row());
        right.append(&layout);
        right.append(&control::build(&popup, painted));

        root.set_start_widget(Some(&left));
        root.set_center_widget(Some(&clock()));
        root.set_end_widget(Some(&right));
        layer.window.set_child(Some(&root));
        layer.window.present();
        if preview() {
            let sample = popup.clone();
            glib::timeout_add_seconds_local_once(1, move || {
                volume::draw(&sample, volume::Level { percent: 60, muted: false });
            });
        }

        Rc::new(Self {
            monitor: monitor.clone(),
            window: layer.window,
            popup,
            workspaces,
            chips: RefCell::new(Vec::new()),
            front,
            close,
            title,
            layout,
        })
    }

    /// Draw this bar's view of the niri state. The connector is read on every
    /// draw: a monitor plugged in after login joins the list before its name
    /// arrives, and a name kept from that moment would be `None` for good.
    pub fn render(&self, state: &State) {
        let connector = self.monitor.connector();
        let view = state.view(connector.as_deref());
        self.render_chips(&view.chips);
        self.front.set(view.window);
        self.close.set_visible(view.window.is_some());
        self.title.set_text(view.title.as_deref().unwrap_or_default());
        self.title.set_visible(view.title.is_some());
        match state.layout() {
            Some((code, name)) => {
                self.layout.set_label(&code);
                self.layout.set_tooltip_text(Some(&format!("{}: {name}", text::get().layout_tooltip)));
                self.layout.set_visible(true);
            }
            None => self.layout.set_visible(false),
        }
    }

    /// Update the chips in place when the same workspaces are shown, so the
    /// focus moves with a transition; rebuild them when the set changed.
    fn render_chips(&self, chips: &[Chip]) {
        let mut current = self.chips.borrow_mut();
        let same = current.len() == chips.len()
            && current.iter().zip(chips).all(|((id, _), chip)| *id == chip.id);
        if !same {
            while let Some(child) = self.workspaces.first_child() {
                self.workspaces.remove(&child);
            }
            *current = chips
                .iter()
                .map(|chip| {
                    let button = chip_button(chip.id);
                    self.workspaces.append(&button);
                    (chip.id, button)
                })
                .collect();
        }
        for ((_, button), chip) in current.iter().zip(chips) {
            button.set_label(&chip.label);
            set_class(button, "active", chip.active);
            set_class(button, "focused", chip.focused);
            set_class(button, "urgent", chip.urgent);
        }
    }

    /// Close the bar for good, when its monitor goes away.
    pub fn destroy(&self) {
        self.popup.destroy();
        self.window.destroy();
    }
}

/// A 32 px button that shows one glyph and says what it does in a tooltip and
/// to assistive technology.
fn icon_button(painted: &Rc<RefCell<icons::Painted>>, icon: icons::Icon, class: &str, says: &str) -> Button {
    let button = Button::new();
    button.set_child(Some(&painted.borrow_mut().image(icon)));
    button.add_css_class("icon");
    button.add_css_class(class);
    button.set_valign(gtk4::Align::Center);
    button.set_tooltip_text(Some(says));
    button.update_property(&[Property::Label(says)]);
    button
}

/// The apps button: it opens the launcher, so an app can be started without
/// knowing a key.
fn apps_button(painted: &Rc<RefCell<icons::Painted>>) -> Button {
    let button = icon_button(painted, icons::Icon::Apps, "apps", text::get().apps_tooltip);
    button.connect_clicked(|_| spawn::detached(spawn::sibling("launcher"), &[]));
    button
}

/// The close button: it closes the window in front on this bar's output,
/// which `front` names. Hidden while there is no window.
fn close_button(painted: &Rc<RefCell<icons::Painted>>, front: &Rc<Cell<Option<u64>>>) -> Button {
    let button = icon_button(painted, icons::Icon::Close, "close", text::get().close_tooltip);
    button.set_visible(false);
    let front = front.clone();
    button.connect_clicked(move |_| {
        if let Some(id) = front.get() {
            niri::send(niri::close_request(id));
        }
    });
    button
}

/// The left module: the apps button, the workspace chips, then the close
/// button and the title of the window in front.
fn left_module(painted: &Rc<RefCell<icons::Painted>>, close: &Button) -> (Box, Box, Label) {
    let workspaces = Box::new(gtk4::Orientation::Horizontal, MODULE_GAP);
    workspaces.add_css_class("workspaces");
    workspaces.set_visible(niri::present());
    let title = Label::new(None);
    title.add_css_class("title");
    title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    title.set_max_width_chars(TITLE_CHARS);
    let window = Box::new(gtk4::Orientation::Horizontal, MODULE_GAP / 2);
    window.append(close);
    window.append(&title);
    let left = Box::new(gtk4::Orientation::Horizontal, MODULE_GAP);
    left.set_halign(gtk4::Align::Start);
    left.append(&apps_button(painted));
    left.append(&workspaces);
    left.append(&window);
    (left, workspaces, title)
}

/// The keyboard layout pill, hidden until niri reports several layouts. A
/// click moves to the next one.
fn layout_pill() -> Button {
    let layout = Button::new();
    layout.add_css_class("pill");
    layout.add_css_class("layout");
    layout.set_visible(false);
    layout.set_tooltip_text(Some(text::get().layout_tooltip));
    layout.connect_clicked(|_| niri::send(niri::NEXT_LAYOUT.to_string()));
    layout
}

/// Whether the bar opens its volume card with a sample level, for a
/// screenshot: `FLUX_SHELL_PREVIEW=popup`.
fn preview() -> bool {
    std::env::var("FLUX_SHELL_PREVIEW").is_ok_and(|value| value == "popup")
}

fn set_class(widget: &impl IsA<gtk4::Widget>, class: &str, on: bool) {
    if on {
        widget.add_css_class(class);
    } else {
        widget.remove_css_class(class);
    }
}

/// A workspace chip that asks niri to focus workspace `id` on its own output.
fn chip_button(id: u64) -> Button {
    let button = Button::new();
    button.add_css_class("ws");
    button.set_valign(gtk4::Align::Center);
    button.connect_clicked(move |_| niri::send(niri::focus_request(id)));
    button
}

/// The clock: the date and the time, checked every second so the minute turns
/// on time, and opening the sidebar on click.
fn clock() -> Button {
    let label = Label::new(None);
    let button = Button::new();
    button.set_child(Some(&label));
    button.add_css_class("clock");
    button.set_tooltip_text(Some(text::get().clock_tooltip));
    button.connect_clicked(|_| spawn::detached(spawn::sibling("sidebar"), &["toggle"]));

    update_clock(&label);
    let weak = label.downgrade();
    glib::timeout_add_seconds_local(1, move || {
        let Some(label) = weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        update_clock(&label);
        glib::ControlFlow::Continue
    });
    button
}

fn update_clock(label: &Label) {
    let Some(now) = glib::DateTime::now_local()
        .ok()
        .and_then(|now| now.format(text::get().clock_format).ok())
    else {
        return;
    };
    if label.text() != now {
        label.set_text(&now);
    }
}

/// The CPU and memory readings, as two pills on a two-second timer.
fn stats_row() -> Box {
    let ram = Label::new(None);
    ram.add_css_class("stat");
    let cpu = Label::new(None);
    cpu.add_css_class("stat");
    let row = Box::new(gtk4::Orientation::Horizontal, MODULE_GAP);
    row.add_css_class("stats");
    row.append(&cpu);
    row.append(&ram);

    let mut sampler = stats::Sampler::new();
    update_stats(&ram, &cpu, &mut sampler);
    let (ram, cpu) = (ram.downgrade(), cpu.downgrade());
    glib::timeout_add_seconds_local(2, move || {
        let (Some(ram), Some(cpu)) = (ram.upgrade(), cpu.upgrade()) else {
            return glib::ControlFlow::Break;
        };
        update_stats(&ram, &cpu, &mut sampler);
        glib::ControlFlow::Continue
    });
    row
}

fn update_stats(ram: &Label, cpu: &Label, sampler: &mut stats::Sampler) {
    if let Some(percent) = stats::memory_percent() {
        ram.set_text(&format!("RAM {percent}%"));
    }
    if let Some(percent) = sampler.cpu_percent() {
        cpu.set_text(&format!("CPU {percent}%"));
    }
}
