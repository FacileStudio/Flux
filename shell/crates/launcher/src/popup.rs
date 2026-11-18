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

/// The installed apps and one result row for each, built once so a query only
/// reorders rows that already exist.
pub(super) struct Catalog {
    pub apps: Vec<App>,
    rows: Vec<ListBoxRow>,
}

impl Catalog {
    pub fn new(apps: Vec<App>) -> Self {
        let rows = apps.iter().map(row).collect();
        Self { apps, rows }
    }
}

/// The result rows, their scroller and the line shown when nothing matches or
/// an app could not start.
#[derive(Clone)]
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
    empty.set_wrap(true);
    empty.set_max_width_chars(48);
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
    catalog: &Rc<RefCell<Catalog>>,
) -> Rc<RefCell<Vec<usize>>> {
    let matches = Rc::new(RefCell::new(fill(results, &catalog.borrow(), "")));

    let found = matches.clone();
    let found_results = results.clone();
    let found_catalog = catalog.clone();
    entry.connect_changed(move |entry| {
        *found.borrow_mut() = fill(&found_results, &found_catalog.borrow(), &entry.text());
    });

    let row_catalog = catalog.clone();
    let row_matches = matches.clone();
    let row_results = results.clone();
    let row_surface = surface.clone();
    results.list.connect_row_activated(move |_, row| {
        launch_at(&row_surface, &row_results, &row_catalog.borrow().apps, &row_matches.borrow(), row.index());
    });

    window.add_controller(keys(surface, results, catalog, matches.clone()));
    window.add_controller(click_outside(surface));
    matches
}

/// The keys: `Esc` closes, the arrows move, `Return` launches the selection.
fn keys(
    surface: &Box,
    results: &Results,
    catalog: &Rc<RefCell<Catalog>>,
    matches: Rc<RefCell<Vec<usize>>>,
) -> gtk4::EventControllerKey {
    let controller = gtk4::EventControllerKey::new();
    controller.set_propagation_phase(PropagationPhase::Capture);
    let surface = surface.clone();
    let results = results.clone();
    let catalog = catalog.clone();
    controller.connect_key_pressed(move |_, key, _, _| match key {
        gdk::Key::Escape => {
            close(&surface);
            glib::Propagation::Stop
        }
        gdk::Key::Down => {
            move_selection(&results, matches.borrow().len(), 1);
            glib::Propagation::Stop
        }
        gdk::Key::Up => {
            move_selection(&results, matches.borrow().len(), -1);
            glib::Propagation::Stop
        }
        gdk::Key::Return | gdk::Key::KP_Enter => {
            let position = results.list.selected_row().map(|row| row.index()).unwrap_or(0);
            launch_at(&surface, &results, &catalog.borrow().apps, &matches.borrow(), position);
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

/// Fade the palette out, then hide it until the next activation.
pub(super) fn close(surface: &Box) {
    let window = surface.root().and_downcast::<gtk4::Window>();
    motion::conceal(surface, move || {
        if let Some(window) = window {
            window.set_visible(false);
        }
    });
}

/// Refill the list with the apps matching `query`, select the first, and report
/// which apps are on screen now.
pub(super) fn fill(results: &Results, catalog: &Catalog, query: &str) -> Vec<usize> {
    let list = &results.list;
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    let found = apps::matches(&catalog.apps, query);
    for &index in &found {
        list.append(&catalog.rows[index]);
    }
    if let Some(first) = list.row_at_index(0) {
        list.select_row(Some(&first));
    }
    results.scroll.vadjustment().set_value(0.0);
    results.empty.set_text(text::get().empty);
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

/// Move the selection by `delta` rows, clamped to the list, and scroll so the
/// selected row is on screen. The rows take no focus, so nothing else scrolls.
fn move_selection(results: &Results, count: usize, delta: i32) {
    if count == 0 {
        return;
    }
    let list = &results.list;
    let current = list.selected_row().map(|row| row.index()).unwrap_or(0);
    let target = (current + delta).clamp(0, count as i32 - 1);
    let Some(row) = list.row_at_index(target) else {
        return;
    };
    list.select_row(Some(&row));
    if let Some(bounds) = row.compute_bounds(list) {
        let adjustment = results.scroll.vadjustment();
        adjustment.set_value(scrolled_to_show(
            adjustment.value(),
            adjustment.page_size(),
            f64::from(bounds.y()),
            f64::from(bounds.height()),
        ));
    }
}

/// Where a view `page` tall, scrolled to `value`, has to be to show the span
/// that starts at `top` and is `height` tall: where it is when the span is in
/// view, and otherwise the nearest position that shows all of it.
fn scrolled_to_show(value: f64, page: f64, top: f64, height: f64) -> f64 {
    if top < value {
        top
    } else if top + height > value + page {
        top + height - page
    } else {
        value
    }
}

/// Launch the app shown at `position`, then hide: the launcher has done its
/// job. When the app cannot start, stay open and say so under the list.
fn launch_at(surface: &Box, results: &Results, apps: &[App], matches: &[usize], position: i32) {
    let Some(app) = usize::try_from(position)
        .ok()
        .and_then(|position| matches.get(position))
        .and_then(|&index| apps.get(index))
    else {
        return;
    };
    match exec::launch(&app.exec, app.terminal) {
        Ok(()) => close(surface),
        Err(error) => {
            eprintln!("launcher: {}: {}", app.name, error);
            results.empty.set_text(&text::get().failed.replace("{name}", &app.name));
            results.empty.set_visible(true);
        }
    }
}
