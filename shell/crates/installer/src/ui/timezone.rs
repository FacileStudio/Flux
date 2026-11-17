use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box, ListBox, Orientation, SearchEntry, Widget};

use super::{list_panel, map, page, row, row_at};
use crate::app::{App, Screen};

/// A zone the way a person reads it: `America / Sao Paulo`.
pub fn readable(zone: &str) -> String {
    zone.replace('_', " ").replace('/', " / ")
}

fn zone_list(app: &Rc<App>, chosen: Rc<RefCell<String>>, redraw: gtk4::DrawingArea) -> ListBox {
    let list = ListBox::new();
    let state = app.state.borrow();
    for zone in &state.zones {
        let item = row(None, &readable(zone), None, &[]);
        item.set_widget_name(zone);
        list.append(&item);
    }
    if let Some(index) = state.zones.iter().position(|zone| *zone == state.timezone) {
        list.select_row(row_at(&list, index).as_ref());
    }
    drop(state);
    let weak = Rc::downgrade(app);
    list.connect_row_selected(move |_, picked| {
        let (Some(app), Some(picked)) = (weak.upgrade(), picked) else {
            return;
        };
        let zone = picked.widget_name().to_string();
        let mut state = app.state.borrow_mut();
        state.timezone.clone_from(&zone);
        state.timezone_chosen = true;
        drop(state);
        *chosen.borrow_mut() = zone;
        redraw.queue_draw();
    });
    list
}

fn show(list: &ListBox, entry: &SearchEntry, zones: &[String], zone: &str) {
    if !entry.text().is_empty() {
        entry.set_text("");
        list.invalidate_filter();
    }
    if let Some(item) = zones.iter().position(|name| name == zone).and_then(|index| row_at(list, index)) {
        list.select_row(Some(&item));
        item.grab_focus();
    }
}

fn search(list: &ListBox, placeholder: &str) -> SearchEntry {
    let entry = SearchEntry::new();
    entry.set_placeholder_text(Some(placeholder));
    let filtered = list.clone();
    let query = entry.clone();
    list.set_filter_func(move |item| {
        let needle = query.text().to_lowercase();
        needle.is_empty() || readable(&item.widget_name()).to_lowercase().contains(needle.trim())
    });
    entry.connect_search_changed(move |_| filtered.invalidate_filter());
    let first = list.clone();
    entry.connect_activate(move |_| {
        let mut index = 0;
        while let Some(item) = first.row_at_index(index) {
            if item.is_child_visible() {
                first.select_row(Some(&item));
                item.grab_focus();
                return;
            }
            index += 1;
        }
    });
    entry
}

pub fn build(app: &Rc<App>) -> Widget {
    let text = app.text();
    let (points, zones, current) = {
        let state = app.state.borrow();
        (Rc::new(state.points.clone()), state.zones.clone(), state.timezone.clone())
    };
    let chosen = Rc::new(RefCell::new(current));
    let picked: Rc<RefCell<Option<(ListBox, SearchEntry)>>> = Rc::new(RefCell::new(None));
    let target = picked.clone();
    let area = map::build(points, chosen.clone(), move |zone| {
        if let Some((list, entry)) = target.borrow().as_ref() {
            show(list, entry, &zones, &zone);
        }
    });
    let list = zone_list(app, chosen, area.clone());
    let entry = search(&list, text.timezone_search);
    *picked.borrow_mut() = Some((list.clone(), entry.clone()));
    let frame = Box::new(Orientation::Vertical, 0);
    frame.add_css_class("panel");
    frame.add_css_class("map-panel");
    frame.append(&area);
    let body = Box::new(Orientation::Vertical, 8);
    body.append(&frame);
    body.append(&entry);
    body.append(&list_panel(&list, 160));
    let next = super::next(text.next);
    let weak = Rc::downgrade(app);
    next.connect_clicked(move |_| {
        if let Some(app) = weak.upgrade() {
            app.go(Screen::Software);
        }
    });
    super::default(app, &next);
    if let Some(selected) = list.selected_row() {
        gtk4::glib::idle_add_local_once(move || {
            selected.grab_focus();
        });
    }
    page(app, text.timezone_title, text.timezone_subtitle, &body, &[&next])
}
