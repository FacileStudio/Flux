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

use crate::backend::{self, Message, AUTH_DISMISSED, AUTH_REFUSED};
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
    raw: RefCell<String>,
    guarded: RefCell<Option<gtk4::Widget>>,
    finish: Finish,
}

/// The plain sentence for a failed change, from the helper's exit `code` and its
/// `raw` output, or `None` when nothing more precise than "it failed" is known.
pub fn cause(text: &Text, code: i32, raw: &str) -> Option<&'static str> {
    let raw = raw.to_lowercase();
    let has = |needles: &[&str]| needles.iter().any(|needle| raw.contains(needle));
    if code == AUTH_REFUSED {
        Some(text.failed_auth)
    } else if code == 2 {
        Some(text.failed_unsupported)
    } else if code == 3 {
        Some(text.failed_admin)
    } else if code == 4 || has(&["another change is already running"]) {
        Some(text.failed_busy)
    } else if has(&["settings.json"]) {
        Some(text.failed_settings)
    } else if has(&["no space left on device"]) {
        Some(text.failed_space)
    } else if has(&["cannot reach the update server", "could not resolve host", "name resolution", "network is unreachable", "unable to download", "timed out", "couldn't connect"]) {
        Some(text.failed_network)
    } else {
        None
    }
}

/// What the failure panel says: the cause when known, then what state the system is in.
pub fn failure_message(text: &Text, code: i32, raw: &str) -> String {
    match cause(text, code, raw) {
        Some(cause) => format!("{cause}\n{}", text.failed_hint),
        None => text.failed_unknown.to_string(),
    }
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
            raw: RefCell::new(String::new()),
            guarded: RefCell::new(None),
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

    /// Keep `widget` from taking input while a change runs, so nothing on it can be flipped and then ignored.
    pub fn guard(&self, widget: &impl IsA<gtk4::Widget>) {
        *self.guarded.borrow_mut() = Some(widget.clone().upcast());
    }

    fn set_busy(&self, busy: bool) {
        self.busy.set(busy);
        if let Some(widget) = self.guarded.borrow().as_ref() {
            widget.set_sensitive(!busy);
        }
    }

    /// Hide the panel.
    pub fn dismiss(&self) {
        ui::show(&self.revealer, false);
    }

    /// Call `finish` each time a change ends.
    pub fn on_finish(&self, finish: impl Fn(Outcome) + 'static) {
        *self.finish.borrow_mut() = Some(std::boxed::Box::new(finish));
    }

    fn reset(&self) {
        self.set_busy(true);
        self.raw.borrow_mut().clear();
        self.details.set_expanded(false);
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
        let raw = self.raw.borrow().clone();
        backend::spawn(|| backend::log_tail(200), move |tail| buffer.set_text(&details_text(&tail, &raw)));
    }

    fn show(&self, line: &Line) {
        if line.step == Step::Error {
            return;
        }
        self.title.set_text(step_name(self.text, line.step));
        self.bar.set_fraction(f64::from(line.percent) / 100.0);
    }

    fn end(self: &Rc<Self>, outcome: Outcome, code: i32, raw: &str) {
        self.set_busy(false);
        match &outcome {
            Outcome::Dismissed => ui::show(&self.revealer, false),
            Outcome::Done { restart } => self.done(*restart),
            Outcome::Failed => self.failed(code, raw),
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

    fn failed(self: &Rc<Self>, code: i32, raw: &str) {
        self.revealer.add_css_class("fx-failed");
        self.title.set_text(self.text.failed);
        self.bar.set_visible(false);
        self.hint.set_text(&failure_message(self.text, code, raw));
        self.hint.set_visible(true);
        self.raw.replace(raw.to_string());
        self.refresh_log();
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
                self.exited(code, if error.is_empty() { last_error.clone() } else { error });
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
                    move |restart| task.end(Outcome::Done { restart }, 0, ""),
                );
            }
            AUTH_DISMISSED => self.end(Outcome::Dismissed, code, ""),
            _ => self.end(Outcome::Failed, code, &error),
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
            self.end(Outcome::Failed, 1, "");
            return;
        };
        self.show(&state.line);
        match state.line.step {
            Step::Done => self.end(Outcome::Done { restart: state.restart }, 0, ""),
            Step::Error => self.end(Outcome::Failed, 1, &state.line.message),
            _ if !running => self.end(Outcome::Failed, 1, ""),
            _ => {}
        }
    }

    /// Show the panel mid-way, for design checks.
    pub fn sample(self: &Rc<Self>, step: Step, percent: u8) {
        self.reset();
        self.show(&Line::new(step, percent, ""));
        self.set_busy(false);
    }

    /// Show the panel finished or failed, for design checks.
    pub fn sample_end(self: &Rc<Self>, outcome: Outcome, code: i32, raw: &str) {
        self.reset();
        self.end(outcome, code, raw);
    }
}

/// The Details text: the end of the operation log, then the helper's own last words.
pub fn details_text(tail: &str, raw: &str) -> String {
    [tail.trim_end(), raw.trim()].into_iter().filter(|part| !part.is_empty()).collect::<Vec<_>>().join("\n\n")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{EN, FR};

    #[test]
    fn failures_read_plainly() {
        let pkexec = "Error executing command as another user: Not authorized\n\nThis incident has been reported.";
        assert_eq!(cause(&EN, AUTH_REFUSED, pkexec), Some(EN.failed_auth));
        assert!(!failure_message(&FR, AUTH_REFUSED, pkexec).contains("incident"));
        assert_eq!(cause(&EN, 2, "flux-config: set takes hostname\n\nusage: flux-config"), Some(EN.failed_unsupported));
        assert_eq!(cause(&EN, 1, "flux-config: another change is already running; wait for it to finish"), Some(EN.failed_busy));
        assert_eq!(cause(&FR, 4, ""), Some(FR.failed_busy));
        assert_eq!(cause(&FR, 3, "flux-config: this change needs administrator rights"), Some(FR.failed_admin));
        assert_eq!(cause(&EN, 1, "flux-config: settings.json: unknown field `x`"), Some(EN.failed_settings));
        assert_eq!(cause(&EN, 1, "nixos-rebuild failed (exit status: 1): error: writing to file: No space left on device"), Some(EN.failed_space));
        assert_eq!(cause(&EN, 1, "nixos-rebuild failed (exit status: 1): error: unable to download 'https://cache.nixos.org/x.narinfo': Could not resolve host"), Some(EN.failed_network));
        assert_eq!(cause(&EN, 1, "nixos-rebuild failed (exit status: 1): error: attribute 'x' missing"), None);
        assert_eq!(failure_message(&FR, 1, "error: attribute 'x' missing"), FR.failed_unknown);
        assert_eq!(failure_message(&EN, 1, ""), EN.failed_unknown);
    }

    #[test]
    fn details_keep_the_raw_output() {
        assert_eq!(details_text("building\n", "flux-config: boom\n"), "building\n\nflux-config: boom");
        assert_eq!(details_text("", "boom"), "boom");
        assert_eq!(details_text("building", ""), "building");
    }
}
