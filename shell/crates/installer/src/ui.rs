//! The pieces every screen is made of, and the screen builders.

mod account;
mod disk;
mod finish;
mod language;
mod progress;
mod summary;
mod timezone;
mod welcome;

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box, Button, DrawingArea, Label, ListBox, Orientation, ScrolledWindow, Widget};
use shell_common::ui::icon;

use crate::app::{App, Screen, STEPS};

pub const WIDTH: i32 = 720;

/// The widget tree for `screen`.
pub fn build(app: &Rc<App>, screen: Screen) -> Widget {
    match screen {
        Screen::Welcome => welcome::build(app),
        Screen::Language => language::build(app),
        Screen::Disk => disk::build(app),
        Screen::Confirm => disk::confirm(app),
        Screen::Account => account::build(app),
        Screen::TimeZone => timezone::build(app),
        Screen::Summary => summary::build(app),
        Screen::Progress => progress::build(app),
        Screen::Error => finish::error(app),
        Screen::Done => finish::done(app),
    }
}

/// A label with `class`, aligned to the start and wrapping.
pub fn label(text: &str, class: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class(class);
    label.set_xalign(0.0);
    label.set_wrap(true);
    label.set_max_width_chars(1);
    label
}

/// A button with `label` and the emphasis `class`: primary, secondary, ghost or danger.
pub fn button(text: &str, class: &str) -> Button {
    let button = Button::with_label(text);
    button.add_css_class("action");
    button.add_css_class(class);
    button
}

/// A list in a filled panel that scrolls on its own.
pub fn list_panel(list: &ListBox, height: i32) -> ScrolledWindow {
    list.add_css_class("choices");
    let scroll = ScrolledWindow::new();
    scroll.add_css_class("panel");
    scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scroll.set_min_content_height(height);
    scroll.set_vexpand(true);
    scroll.set_child(Some(list));
    scroll
}

/// A screen: title, subtitle, body, then Back and the actions with the primary one last.
pub fn page(app: &Rc<App>, title: &str, subtitle: &str, body: &impl IsA<Widget>, actions: &[&Button]) -> Widget {
    let column = Box::new(Orientation::Vertical, 24);
    column.add_css_class("page");
    column.set_halign(Align::Center);
    column.set_width_request(WIDTH);
    let header = Box::new(Orientation::Vertical, 8);
    if let Some(step) = app.current().step() {
        let counter = app.text().step.replace("{n}", &step.to_string()).replace("{total}", &STEPS.to_string());
        header.append(&label(&counter, "step"));
    }
    header.append(&label(title, "title"));
    header.append(&label(subtitle, "subtitle"));
    column.append(&header);
    body.set_vexpand(true);
    column.append(body);
    column.append(&footer(app, actions));
    let scroll = ScrolledWindow::new();
    scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scroll.set_child(Some(&column));
    scroll.upcast()
}

/// A centred screen led by the Flux mark: the welcome and the finish line.
pub fn hero(title: &str, subtitle: &str, content: &impl IsA<Widget>) -> Widget {
    let column = Box::new(Orientation::Vertical, 0);
    column.add_css_class("page");
    column.set_halign(Align::Center);
    column.set_valign(Align::Center);
    column.set_width_request(WIDTH);
    column.append(&mark(72));
    let header = Box::new(Orientation::Vertical, 8);
    header.add_css_class("hero-header");
    header.append(&label(title, "hero"));
    header.append(&label(subtitle, "subtitle"));
    column.append(&header);
    column.append(content);
    column.upcast()
}

fn footer(app: &Rc<App>, actions: &[&Button]) -> Box {
    let row = Box::new(Orientation::Horizontal, 8);
    row.add_css_class("footer");
    let back = button(app.text().back, "ghost");
    let weak = Rc::downgrade(app);
    back.connect_clicked(move |_| {
        if let Some(app) = weak.upgrade() {
            app.back();
        }
    });
    if app.can_go_back() {
        row.append(&back);
    }
    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    row.append(&spacer);
    for action in actions {
        row.append(*action);
    }
    row
}

/// Make `button` the one Enter presses.
pub fn default(app: &Rc<App>, button: &Button) {
    app.window.set_default_widget(Some(button));
}

/// The Flux mark, drawn in the current text colour.
pub fn mark(size: i32) -> DrawingArea {
    let area = DrawingArea::new();
    area.add_css_class("mark");
    area.set_content_width(size);
    area.set_content_height(size);
    area.set_halign(Align::Start);
    area.set_draw_func(|area, cr, width, height| {
        let color = area.color();
        let scale = f64::from(width.min(height)) / 24.0;
        cr.scale(scale, scale);
        for (x, y, alpha) in [(8.0, 16.0, 0.5), (16.0, 16.0, 0.5), (12.0, 8.0, 1.0)] {
            cr.set_source_rgba(
                f64::from(color.red()),
                f64::from(color.green()),
                f64::from(color.blue()),
                f64::from(color.alpha()) * alpha,
            );
            cr.arc(x, y, 6.0, 0.0, std::f64::consts::TAU);
            let _ = cr.fill();
        }
    });
    area
}

/// The `ListBoxRow` at `index` once it exists, to select or focus it.
pub fn row_at(list: &ListBox, index: usize) -> Option<gtk4::ListBoxRow> {
    list.row_at_index(i32::try_from(index).ok()?)
}

/// A row with an optional leading icon, a main label, an optional muted line and trailing tags.
pub fn row(glyph: Option<&str>, title: &str, detail: Option<&str>, tags: &[&str]) -> gtk4::ListBoxRow {
    let line = Box::new(Orientation::Horizontal, 8);
    if let Some(name) = glyph {
        let image = icon(name, 24);
        image.add_css_class("row-icon");
        image.set_valign(Align::Center);
        line.append(&image);
    }
    let text = Box::new(Orientation::Vertical, 2);
    text.set_hexpand(true);
    text.set_valign(Align::Center);
    let title = label(title, "row-title");
    title.set_wrap(false);
    title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    text.append(&title);
    if let Some(detail) = detail {
        text.append(&label(detail, "dim"));
    }
    line.append(&text);
    for tag in tags {
        let tag = label(tag, "tag");
        tag.set_wrap(false);
        tag.set_valign(Align::Center);
        line.append(&tag);
    }
    let row = gtk4::ListBoxRow::new();
    row.set_child(Some(&line));
    row
}
