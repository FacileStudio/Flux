use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{glib, Box, Expander, Label, Orientation, ProgressBar, ScrolledWindow, TextView, Widget};

use super::{label, page};
use crate::app::{App, Screen};
use crate::i18n::Text;
use crate::model::{self, Event, Step};
use crate::system::{self, Failure, Message};

fn step_name(text: &Text, step: Step) -> &'static str {
    match step {
        Step::Partition => text.step_partition,
        Step::Format => text.step_format,
        Step::Config => text.step_config,
        Step::Install | Step::Done => text.step_install,
        Step::Bootloader => text.step_bootloader,
    }
}

fn failure_text(text: &Text, failure: Failure) -> String {
    match failure {
        Failure::Start(error) => text.error_start.replace("{error}", &error),
        Failure::Hash(error) => text.error_hash.replace("{error}", &error),
        Failure::Exit(code, errors) => {
            let message = text.error_exit.replace("{code}", &code.to_string());
            if errors.is_empty() {
                message
            } else {
                format!("{message}\n{errors}")
            }
        }
    }
}

struct View {
    step: Label,
    bar: ProgressBar,
    details: Expander,
    log: TextView,
}

fn fail(app: &Rc<App>, message: String, log_tail: String) {
    let mut state = app.state.borrow_mut();
    state.error = message;
    state.error_log = log_tail;
    drop(state);
    app.replace(Screen::Error);
}

fn handle(app: &Rc<App>, view: &View, message: Message) -> glib::ControlFlow {
    let text = app.text();
    match message {
        Message::Event(Event::Progress { step: Step::Done, .. }) => {
            app.state.borrow_mut().password.clear();
            app.replace(Screen::Done);
            glib::ControlFlow::Break
        }
        Message::Event(Event::Progress { step, percent, .. }) => {
            view.step.set_text(step_name(text, step));
            view.bar.set_fraction(f64::from(percent) / 100.0);
            glib::ControlFlow::Continue
        }
        Message::Event(Event::Error { message, log_tail }) => {
            fail(app, message, log_tail);
            glib::ControlFlow::Break
        }
        Message::Failed(failure) => {
            fail(app, failure_text(text, failure), String::new());
            glib::ControlFlow::Break
        }
    }
}

fn poll(app: &Rc<App>, view: View, receiver: Receiver<Message>) {
    let app = app.clone();
    let mut ticks = 0u32;
    glib::timeout_add_local(Duration::from_millis(100), move || {
        ticks = ticks.wrapping_add(1);
        if view.details.is_expanded() && ticks.is_multiple_of(10) {
            view.log.buffer().set_text(&model::tail(&system::log(), 200));
        }
        for message in receiver.try_iter() {
            if handle(&app, &view, message).is_break() {
                return glib::ControlFlow::Break;
            }
        }
        glib::ControlFlow::Continue
    });
}

/// The Details expander over the install log. It shows the log's tail when opened, unless  already holds text.
pub(super) fn details(text: &Text) -> (Expander, TextView) {
    let log = TextView::new();
    log.add_css_class("log");
    log.set_editable(false);
    log.set_cursor_visible(false);
    log.set_monospace(true);
    log.set_wrap_mode(gtk4::WrapMode::WordChar);
    let scroll = ScrolledWindow::new();
    scroll.add_css_class("panel");
    scroll.set_min_content_height(240);
    scroll.set_child(Some(&log));
    let expander = Expander::new(Some(text.details));
    expander.add_css_class("advanced");
    expander.set_child(Some(&scroll));
    let view = log.clone();
    expander.connect_expanded_notify(move |expander| {
        if expander.is_expanded() && view.buffer().char_count() == 0 {
            view.buffer().set_text(&model::tail(&system::log(), 200));
        }
    });
    (expander, log)
}

pub fn build(app: &Rc<App>) -> Widget {
    let text = app.text();
    let body = Box::new(Orientation::Vertical, 12);
    let step = label(text.step_start, "heading");
    let bar = ProgressBar::new();
    bar.add_css_class("install");
    let (expander, log) = details(text);
    body.append(&step);
    body.append(&bar);
    body.append(&expander);
    if !app.state.borrow().preview {
        let (sender, receiver) = mpsc::channel();
        let state = app.state.borrow();
        system::install(state.choices(), state.password.clone(), sender);
        drop(state);
        poll(app, View { step, bar, details: expander, log }, receiver);
    } else {
        bar.set_fraction(0.42);
        step.set_text(text.step_install);
    }
    page(app, text.progress_title, text.progress_subtitle, &body, &[])
}
