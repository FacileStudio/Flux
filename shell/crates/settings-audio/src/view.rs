//! The Sound page: GTK only. State comes from the hub; every change goes back through it.

mod apps;
mod controls;
mod device;
mod lists;
mod quick;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    gdk, Box as GtkBox, CssProvider, Orientation, PolicyType, ScrolledWindow, Stack,
    StackTransitionType, Switch, Widget,
};

use crate::backend::{Command, Event, Target};
use crate::i18n::{self, Text};
use crate::model::{self, Direction, Options, Snapshot};
use crate::{hub, motion, ui};

const STYLE: &str = include_str!("../style.css");

thread_local! {
    static STYLED: Cell<bool> = const { Cell::new(false) };
}

fn style() {
    if STYLED.with(|styled| styled.replace(true)) {
        return;
    }
    let Some(display) = gdk::Display::default() else {
        return;
    };
    let provider = CssProvider::new();
    #[allow(deprecated)]
    provider.load_from_data(STYLE);
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

struct Page {
    stack: Stack,
    output: Rc<device::DeviceSection>,
    input: Rc<device::DeviceSection>,
    apps: Rc<apps::AppsSection>,
    boost: Switch,
    guard: Cell<bool>,
    options: Cell<Options>,
    last: RefCell<Option<Snapshot>>,
}

impl Page {
    fn show(&self, snapshot: &Snapshot) {
        let cap = model::max_percent(self.options.get().allow_boost);
        self.output.update(snapshot, cap);
        self.input.update(snapshot, cap);
        self.apps.update(snapshot, cap);
        self.stack.set_visible_child_name("ready");
    }

    fn options(&self, options: Options) {
        self.options.set(options);
        self.guard.set(true);
        self.boost.set_active(options.allow_boost);
        self.guard.set(false);
        let last = self.last.borrow().clone();
        if let Some(snapshot) = last {
            self.show(&snapshot);
        }
    }

    fn event(&self, event: &Event) {
        match event {
            Event::Snapshot(snapshot) => {
                self.show(snapshot);
                self.last.replace(Some((**snapshot).clone()));
            }
            Event::Level(Direction::Output, peak) => self.output.level(*peak),
            Event::Level(Direction::Input, peak) => self.input.level(*peak),
            Event::Unavailable => self.stack.set_visible_child_name("unavailable"),
            Event::Options(options) => self.options(*options),
        }
    }
}

/// The whole Sound page.
pub fn page() -> Widget {
    style();
    let text = i18n::text();
    let content = GtkBox::new(Orientation::Vertical, 32);
    content.add_css_class("fx-page");
    let output = device::DeviceSection::new(text, Direction::Output);
    let input = device::DeviceSection::new(text, Direction::Input);
    let apps = apps::AppsSection::new(text);
    let (options_section, options_card) = ui::section(text.options);
    let (boost_row, boost) = ui::switch_row(text.boost_title, Some(text.boost_body));
    options_card.append(&boost_row);
    content.append(&output.root);
    content.append(&input.root);
    content.append(&apps.root);
    content.append(&options_section);
    let page = Rc::new(Page {
        stack: frame(text, &content),
        output,
        input,
        apps,
        boost,
        guard: Cell::new(false),
        options: Cell::new(Options::default()),
        last: RefCell::new(None),
    });
    let weak = Rc::downgrade(&page);
    page.boost.connect_active_notify(move |switch| {
        if weak.upgrade().is_some_and(|page| !page.guard.get()) {
            set_boost(switch.is_active());
        }
    });
    let weak = Rc::downgrade(&page);
    let id = hub::get().listen(move |event| {
        if let Some(page) = weak.upgrade() {
            page.event(event);
        }
    });
    let stack = page.stack.clone();
    stack.connect_map(|_| hub::get().want_meters(true));
    stack.connect_unmap(|_| hub::get().want_meters(false));
    stack.connect_destroy(move |_| {
        let _ = &page;
        hub::get().forget(id);
    });
    stack.upcast()
}

fn frame(text: &Text, content: &GtkBox) -> Stack {
    let scroller = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .child(content)
        .vexpand(true)
        .build();
    let stack = Stack::new();
    stack.add_css_class("fx-audio");
    stack.add_css_class("fx-sound");
    stack.set_transition_type(StackTransitionType::Crossfade);
    stack.set_transition_duration(if motion::enabled() {
        motion::DURATION
    } else {
        0
    });
    stack.add_named(&loading(text), Some("loading"));
    stack.add_named(
        &ui::empty_state(
            "audio-volume-muted-symbolic",
            text.unavailable_title,
            text.unavailable_body,
        ),
        Some("unavailable"),
    );
    stack.add_named(&scroller, Some("ready"));
    stack.set_visible_child_name("loading");
    stack
}

fn set_boost(allow_boost: bool) {
    let hub = hub::get();
    hub.set_options(Options { allow_boost });
    if allow_boost {
        return;
    }
    let loud = hub.snapshot().and_then(|snapshot| {
        snapshot
            .default_device(Direction::Output)
            .filter(|device| device.percent() > 100)
            .map(|device| device.index)
    });
    if let Some(index) = loud {
        hub.send(Command::SetVolume(
            Target::Device(Direction::Output, index),
            model::raw(100),
            None,
        ));
    }
}

fn loading(text: &Text) -> Widget {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("fx-page");
    root.append(&ui::spinner_row(text.connecting));
    root.upcast()
}

/// The statusbar popover content.
pub fn quick_menu() -> Widget {
    style();
    quick::build(i18n::text())
}
