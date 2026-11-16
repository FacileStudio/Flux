//! The launcher's surface: the query field, the result rows underneath it, and
//! the keys and clicks that act on them.

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    gdk, glib, ApplicationWindow, Box, Entry, Image, Label, ListBox, ListBoxRow, Orientation,
    PolicyType, PropagationPhase, ScrolledWindow, SelectionMode,
};
use shell_common::motion;

use crate::apps::{self, App};
use crate::{exec, text};

#[cfg(test)]
mod tests;

/// The pixel size of a result's icon.
const ICON_SIZE: i32 = 24;

/// The result rows, their scroller and the line shown when nothing matches.
pub(super) struct Results {
    pub list: ListBox,
    pub scroll: ScrolledWindow,
    pub empty: Label,
}

/// The query field, and the results that go under it.
pub(super) fn build() -> (Entry, Results) {
    let entry = Entry::new();
    entry.set_placeholder_text(Some(text::get().search));

    let list = ListBox::new();
    list.set_selection_mode(SelectionMode::Single);
    list.set_activate_on_single_click(true);
    list.set_focusable(false);

    let scroll = ScrolledWindow::new();
    scroll.set_child(Some(&list));
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
    scroll.set_propagate_natural_height(true);
    scroll.set_max_content_height(480);

    let empty = Label::new(Some(text::get().empty));
    empty.add_css_class("dim");
    empty.add_css_class("empty");
    empty.set_visible(false);
    (entry, Results { list, scroll, empty })
}

/// Install the launcher's behaviour: typing refills the list, the arrow keys,
/// `Return` and `Esc` act on it, a click on a row launches it and a click
/// beside the palette closes it. The key controller sits on the window and
/// captures, so the keys reach it before the field consumes them.
pub(super) fn install(
    window: &ApplicationWindow,
    surface: &Box,
    entry: &Entry,
    results: &Results,
    apps: &Rc<Vec<App>>,
) {
    let matches = Rc::new(RefCell::new(fill(results, apps, "")));

    let found = matches.clone();
    let found_results = Results {
        list: results.list.clone(),
        scroll: results.scroll.clone(),
        empty: results.empty.clone(),
    };
    let found_apps = apps.clone();
    entry.connect_changed(move |entry| {
        *found.borrow_mut() = fill(&found_results, &found_apps, &entry.text());
    });

    let row_apps = apps.clone();
    let row_matches = matches.clone();
    results.list.connect_row_activated(move |_, row| {
        launch_at(&row_apps, &row_matches.borrow(), row.index());
    });

    window.add_controller(keys(surface, &results.list, apps, matches));
    window.add_controller(click_outside(surface));
}

/// The keys: `Esc` closes, the arrows move, `Return` launches the selection.
fn keys(
    surface: &Box,
    list: &ListBox,
    apps: &Rc<Vec<App>>,
    matches: Rc<RefCell<Vec<usize>>>,
) -> gtk4::EventControllerKey {
    let controller = gtk4::EventControllerKey::new();
    controller.set_propagation_phase(PropagationPhase::Capture);
    let surface = surface.clone();
    let list = list.clone();
    let apps = apps.clone();
    controller.connect_key_pressed(move |_, key, _, _| match key {
        gdk::Key::Escape => {
            close(&surface);
            glib::Propagation::Stop
        }
        gdk::Key::Down => {
            move_selection(&list, matches.borrow().len(), 1);
            glib::Propagation::Stop
        }
        gdk::Key::Up => {
            move_selection(&list, matches.borrow().len(), -1);
            glib::Propagation::Stop
        }
        gdk::Key::Return | gdk::Key::KP_Enter => {
            let position = list.selected_row().map(|row| row.index()).unwrap_or(0);
            launch_at(&apps, &matches.borrow(), position);
            glib::Propagation::Stop
        }
        _ => glib::Propagation::Proceed,
    });
    controller
}

/// A click that lands outside the palette closes the launcher.
fn click_outside(surface: &Box) -> gtk4::GestureClick {
    let click = gtk4::GestureClick::new();
    let surface = surface.clone();
    click.connect_pressed(move |gesture, _, x, y| {
        let Some(window) = gesture.widget() else {
            return;
        };
        let inside = surface
            .compute_bounds(&window)
            .is_some_and(|bounds| bounds.contains_point(&gtk4::graphene::Point::new(x as f32, y as f32)));
        if !inside {
            close(&surface);
        }
    });
    click
}

/// Fade the palette out, then leave.
fn close(surface: &Box) {
    motion::conceal(surface, || std::process::exit(0));
}

/// Refill the list with the apps matching `query`, select the first, and report
/// which apps are on screen now.
pub(super) fn fill(results: &Results, apps: &[App], query: &str) -> Vec<usize> {
    let list = &results.list;
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    let found = apps::matches(apps, query);
    for &index in &found {
        list.append(&row(&apps[index]));
    }
    if let Some(first) = list.row_at_index(0) {
        list.select_row(Some(&first));
    }
    results.scroll.set_visible(!found.is_empty());
    results.empty.set_visible(found.is_empty());
    found
}

/// One result row: the app's icon and name, and its generic name when it has one.
fn row(app: &App) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.set_focusable(false);

    let line = Box::new(Orientation::Horizontal, 12);
    line.append(&icon(app.icon.as_deref()));

    let name = Label::new(Some(&app.name));
    name.set_xalign(0.0);
    name.set_hexpand(true);
    name.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    line.append(&name);

    if let Some(generic) = &app.generic {
        let suffix = Label::new(Some(generic));
        suffix.add_css_class("dim");
        suffix.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        line.append(&suffix);
    }
    row.set_child(Some(&line));
    row
}

/// The icon an entry names: a file when `Icon` is a path, a theme icon
/// otherwise, the generic application icon when it names nothing.
fn icon(name: Option<&str>) -> Image {
    let image = match name {
        Some(path) if path.starts_with('/') && Path::new(path).is_file() => Image::from_file(path),
        Some(name) => Image::from_icon_name(name),
        None => Image::from_icon_name("application-x-executable"),
    };
    image.set_pixel_size(ICON_SIZE);
    image
}

/// Move the selection by `delta` rows, clamped to the list.
fn move_selection(list: &ListBox, count: usize, delta: i32) {
    if count == 0 {
        return;
    }
    let current = list.selected_row().map(|row| row.index()).unwrap_or(0);
    let target = (current + delta).clamp(0, count as i32 - 1);
    if let Some(row) = list.row_at_index(target) {
        list.select_row(Some(&row));
    }
}

/// Launch the app shown at `position`, then leave: the launcher has done its job.
fn launch_at(apps: &[App], matches: &[usize], position: i32) {
    let Some(app) = usize::try_from(position)
        .ok()
        .and_then(|position| matches.get(position))
        .and_then(|&index| apps.get(index))
    else {
        return;
    };
    if let Err(error) = exec::launch(&app.exec, app.terminal) {
        eprintln!("launcher: {}: {}", app.name, error);
        std::process::exit(1);
    }
    std::process::exit(0);
}
