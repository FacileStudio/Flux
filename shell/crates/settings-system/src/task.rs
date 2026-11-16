//! The progress panel of a long change: friendly step, bar, a Details
//! expander with the log tail, then done, restart or a calm error. It follows a
//! helper it started, or reattaches to one still running after Settings was closed.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use flux_config::progress::{Line, Step};
use gtk4::prelude::*;
use gtk4::{glib, Box, Button, Expander, Label, Orientation, ProgressBar, Revealer, ScrolledWindow, TextView};

use crate::backend::{self, Message, AUTH_DISMISSED};
use crate::i18n::Text;
use crate::ui;

/// How a change ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Done { restart: bool },
    Dismissed,
    Failed,
}

type Finish = RefCell<Option<std::boxed::Box<dyn Fn(Outcome)>>>;

/// One progress panel; a page keeps one per kind of change.
pub struct Task {
    text: &'static Text,
    revealer: Revealer,
    title: Label,
    bar: ProgressBar,
    hint: Label,
    details: Expander,
    log: TextView,
    actions: Box,
    busy: Cell<bool>,
    finish: Finish,
}

/// The friendly name of `step`.
pub fn step_name(text: &Text, step: Step) -> &'static str {
    match step {
        Step::Start => text.step_start,
        Step::Fetch => text.step_fetch,
        Step::Evaluate => text.step_evaluate,
        Step::Build => text.step_build,
        Step::Activate => text.step_activate,
        Step::Bootloader => text.step_bootloader,
        Step::Clean => text.step_clean,
        Step::Password => text.step_password,
        Step::Done => text.step_done,
        Step::Error => text.failed,
    }
}

impl Task {
    /// A hidden panel.
    pub fn new(text: &'static Text) -> Rc<Task> {
        let column = Box::new(Orientation::Vertical, 12);
        column.add_css_class("fx-task");
        let title = ui::label(text.step_start, "fx-task-title");
        let bar = ProgressBar::new();
        bar.add_css_class("fx-progress");
        let hint = ui::label(text.keep_going, "fx-subtitle");
        let log = TextView::new();
        log.add_css_class("fx-log");
        log.set_editable(false);
        log.set_cursor_visible(false);
        log.set_monospace(true);
        log.set_wrap_mode(gtk4::WrapMode::WordChar);
        let scroll = ScrolledWindow::new();
        scroll.add_css_class("fx-log-panel");
        scroll.set_min_content_height(180);
        scroll.set_child(Some(&log));
        let details = Expander::new(Some(text.details));
        details.add_css_class("fx-details");
        details.set_child(Some(&scroll));
        let actions = Box::new(Orientation::Horizontal, 8);
        actions.set_halign(gtk4::Align::End);
        column.append(&title);
        column.append(&bar);
        column.append(&hint);
        column.append(&details);
        column.append(&actions);
        let revealer = ui::reveal(&column);
        let task = Rc::new(Task {
            text,
            revealer,
            title,
            bar,
            hint,
            details,
            log,
            actions,
            busy: Cell::new(false),
            finish: RefCell::new(None),
        });
        let weak = Rc::downgrade(&task);
        task.details.connect_expanded_notify(move |_| {
            if let Some(task) = weak.upgrade() {
                task.refresh_log();
            }
        });
        task
    }

    /// The panel to place on the page.
    pub fn widget(&self) -> &Revealer {
        &self.revealer
    }

    /// Whether a change is running here.
    pub fn busy(&self) -> bool {
        self.busy.get()
    }

    /// Call `finish` each time a change ends.
    pub fn on_finish(&self, finish: impl Fn(Outcome) + 'static) {
        *self.finish.borrow_mut() = Some(std::boxed::Box::new(finish));
    }

    fn reset(&self) {
        self.busy.set(true);
        self.revealer.remove_css_class("fx-failed");
        self.title.set_text(self.text.step_start);
        self.bar.set_fraction(0.0);
        self.bar.set_visible(true);
        self.hint.set_text(self.text.keep_going);
        self.hint.set_visible(true);
        self.log.buffer().set_text("");
        ui::clear(&self.actions);
        self.actions.set_visible(false);
        ui::show(&self.revealer, true);
    }

    fn refresh_log(&self) {
        if !self.details.is_expanded() {
            return;
        }
        let buffer = self.log.buffer();
        backend::spawn(|| backend::log_tail(200), move |tail| buffer.set_text(&tail));
    }

    fn show(&self, line: &Line) {
        if line.step == Step::Error {
            return;
        }
        self.title.set_text(step_name(self.text, line.step));
        self.bar.set_fraction(f64::from(line.percent) / 100.0);
    }

    fn end(self: &Rc<Self>, outcome: Outcome, error: &str) {
        self.busy.set(false);
        match &outcome {
            Outcome::Dismissed => ui::show(&self.revealer, false),
            Outcome::Done { restart } => self.done(*restart),
            Outcome::Failed => self.failed(error),
        }
        if let Some(finish) = self.finish.borrow().as_ref() {
            finish(outcome);
        }
    }

    fn done(self: &Rc<Self>, restart: bool) {
        self.title.set_text(if restart { self.text.restart_hint } else { self.text.step_done });
        self.bar.set_fraction(1.0);
        self.hint.set_visible(false);
        self.add_dismiss();
        if restart {
            let button = ui::primary_button(self.text.restart);
            button.connect_clicked(|_| backend::reboot());
            self.actions.append(&button);
            return;
        }
        let weak = Rc::downgrade(self);
        glib::timeout_add_local_once(Duration::from_millis(2400), move || {
            if let Some(task) = weak.upgrade().filter(|task| !task.busy()) {
                ui::show(&task.revealer, false);
            }
        });
    }

    fn failed(self: &Rc<Self>, error: &str) {
        self.revealer.add_css_class("fx-failed");
        self.title.set_text(self.text.failed);
        self.bar.set_visible(false);
        let message = if error.is_empty() { self.text.failed_hint.to_string() } else { format!("{error}\n{}", self.text.failed_hint) };
        self.hint.set_text(&message);
        self.hint.set_visible(true);
        self.add_dismiss();
    }

    fn add_dismiss(self: &Rc<Self>) {
        let dismiss = ui::ghost_button(self.text.dismiss);
        let weak = Rc::downgrade(self);
        dismiss.connect_clicked(move |_| {
            if let Some(task) = weak.upgrade() {
                ui::show(&task.revealer, false);
            }
        });
        self.actions.prepend(&dismiss);
        self.actions.set_visible(true);
    }

    /// Run `pkexec flux-config <args>`, with `input` on its stdin, and follow it.
    pub fn start(self: &Rc<Self>, args: Vec<String>, input: Option<String>) {
        if self.busy() {
            return;
        }
        self.reset();
        let (sender, receiver) = mpsc::channel();
        backend::run(args, input, sender);
        self.follow(receiver);
    }

    fn follow(self: &Rc<Self>, receiver: Receiver<Message>) {
        let task = self.clone();
        let mut ticks = 0u32;
        let mut last_error = String::new();
        glib::timeout_add_local(Duration::from_millis(100), move || {
            ticks = ticks.wrapping_add(1);
            if ticks.is_multiple_of(10) {
                task.refresh_log();
            }
            for message in receiver.try_iter() {
                if task.receive(message, &mut last_error).is_break() {
                    return glib::ControlFlow::Break;
                }
            }
            glib::ControlFlow::Continue
        });
    }

    fn receive(self: &Rc<Self>, message: Message, last_error: &mut String) -> glib::ControlFlow {
        match message {
            Message::Line(line) => {
                if line.step == Step::Error {
                    last_error.clone_from(&line.message);
                }
                self.show(&line);
                glib::ControlFlow::Continue
            }
            Message::Exit { code, error } => {
                self.exited(code, if last_error.is_empty() { error } else { last_error.clone() });
                glib::ControlFlow::Break
            }
        }
    }

    fn exited(self: &Rc<Self>, code: i32, error: String) {
        match code {
            0 => {
                let task = self.clone();
                backend::spawn(
                    || backend::state().map(|state| state.restart).unwrap_or(false) || backend::installed().restart,
                    move |restart| task.end(Outcome::Done { restart }, ""),
                );
            }
            AUTH_DISMISSED => self.end(Outcome::Dismissed, ""),
            _ => self.end(Outcome::Failed, &error),
        }
    }

    /// Follow an operation a helper started before this page opened.
    pub fn attach(self: &Rc<Self>, line: &Line) {
        self.reset();
        self.show(line);
        let task = self.clone();
        let mut ticks = 0u32;
        glib::timeout_add_local(Duration::from_millis(500), move || {
            ticks = ticks.wrapping_add(1);
            if ticks.is_multiple_of(2) {
                task.refresh_log();
            }
            let watcher = task.clone();
            backend::spawn(
                || (backend::state(), backend::running().is_some()),
                move |(state, running)| watcher.attached(state, running),
            );
            if task.busy() {
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });
    }

    fn attached(self: &Rc<Self>, state: Option<flux_config::progress::State>, running: bool) {
        if !self.busy() {
            return;
        }
        let Some(state) = state else {
            self.end(Outcome::Failed, "");
            return;
        };
        self.show(&state.line);
        match state.line.step {
            Step::Done => self.end(Outcome::Done { restart: state.restart }, ""),
            Step::Error => self.end(Outcome::Failed, &state.line.message),
            _ if !running => self.end(Outcome::Failed, ""),
            _ => {}
        }
    }

    /// Show the panel mid-way, for design checks.
    pub fn sample(self: &Rc<Self>, step: Step, percent: u8) {
        self.reset();
        self.show(&Line::new(step, percent, ""));
        self.busy.set(false);
    }

    /// Show the panel finished or failed, for design checks.
    pub fn sample_end(self: &Rc<Self>, outcome: Outcome, error: &str) {
        self.reset();
        self.end(outcome, error);
    }
}

/// A danger-toned confirmation: title, body, Cancel and the danger action.
pub fn confirm(title: &str, body: &str, cancel: &str, action: &str) -> (Box, Button, Button) {
    let column = Box::new(Orientation::Vertical, 8);
    column.add_css_class("fx-confirm");
    column.append(&ui::label(title, "fx-confirm-title"));
    column.append(&ui::label(body, "fx-confirm-body"));
    let no = ui::ghost_button(cancel);
    let yes = ui::danger_button(action);
    column.append(&ui::actions(&[&no, &yes]));
    (column, no, yes)
}
