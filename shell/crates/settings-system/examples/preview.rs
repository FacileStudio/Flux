//! A stand-in Settings window around the three pages, for design checks:
//! `FLUX_SETTINGS_PREVIEW=updates:progress cargo run -p settings-system --example preview`.
//! With `FLUX_SETTINGS_PREVIEW` set the pages show sample data and run nothing.

use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box, Image, Label, ListBox, Orientation, Stack, StackTransitionType, Widget};
use settings_system::{system, updates, users};

const NAV_CSS: &str = "
window.fx-window, window.fx-window.backdrop { background-color: @fc_page; }
window.fx-window scrolledwindow, window.fx-window viewport { background-color: transparent; }
.fx-nav { background-color: @fc_component; padding: 16px 8px; min-width: 220px; }
.fx-nav row { padding: 8px 12px; border-radius: 8px; color: @fc_fg; transition: background-color 150ms ease-out; }
.fx-nav row:hover { background-color: @fc_surface; }
.fx-nav row:selected { background-color: @fc_surface; color: @fc_fg; }
.fx-nav list { background-color: transparent; }
";

struct Entry {
    id: &'static str,
    icon: &'static str,
    title: String,
    build: fn() -> Widget,
}

fn entries() -> [Entry; 3] {
    [
        Entry { id: system::ID, icon: system::ICON, title: system::title(), build: system::page },
        Entry { id: users::ID, icon: users::ICON, title: users::title(), build: users::page },
        Entry { id: updates::ID, icon: updates::ICON, title: updates::title(), build: updates::page },
    ]
}

fn nav_row(entry: &Entry) -> gtk4::ListBoxRow {
    let line = Box::new(Orientation::Horizontal, 10);
    line.append(&Image::from_icon_name(entry.icon));
    let label = Label::new(Some(&entry.title));
    label.set_halign(Align::Start);
    line.append(&label);
    let row = gtk4::ListBoxRow::new();
    row.set_child(Some(&line));
    row
}

fn activate(application: &Application) {
    let css: &'static str = std::boxed::Box::leak(format!("{}\n{}\n{NAV_CSS}", shell_common::ui::CSS, settings_system::CSS).into_boxed_str());
    shell_common::theme::init(css, "settings-preview");
    let window = ApplicationWindow::builder().application(application).title("Settings").default_width(1000).default_height(760).build();
    window.add_css_class("fx-window");
    let stack = Stack::new();
    stack.set_transition_type(StackTransitionType::Crossfade);
    stack.set_transition_duration(180);
    stack.set_hexpand(true);
    let nav = ListBox::new();
    let side = Box::new(Orientation::Vertical, 0);
    side.add_css_class("fx-nav");
    side.append(&nav);
    let entries = entries();
    for entry in &entries {
        nav.append(&nav_row(entry));
        stack.add_named(&(entry.build)(), Some(entry.id));
    }
    let wanted = std::env::var("FLUX_SETTINGS_PREVIEW").unwrap_or_default();
    let start = entries.iter().position(|entry| wanted.split(':').next() == Some(entry.id)).unwrap_or(0);
    let ids: Vec<&'static str> = entries.iter().map(|entry| entry.id).collect();
    let pages = stack.clone();
    nav.connect_row_selected(move |_, row| {
        if let Some(id) = row.and_then(|row| ids.get(usize::try_from(row.index()).ok()?)) {
            pages.set_visible_child_name(id);
        }
    });
    if let Some(row) = nav.row_at_index(i32::try_from(start).unwrap_or(0)) {
        nav.select_row(Some(&row));
    }
    let layout = Box::new(Orientation::Horizontal, 0);
    layout.append(&side);
    layout.append(&stack);
    window.set_child(Some(&layout));
    window.present();
}

fn main() {
    if std::env::var_os("FLUX_SETTINGS_PREVIEW").is_none() {
        std::env::set_var("FLUX_SETTINGS_PREVIEW", "system");
    }
    let application = Application::builder().application_id("studio.facile.FluxSettingsPreview").build();
    application.connect_activate(activate);
    let _ = shell_common::app::run(&application, "settings-preview");
}
