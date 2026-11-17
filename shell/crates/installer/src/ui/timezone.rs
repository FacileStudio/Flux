use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box, ListBox, Orientation, SearchEntry, Widget};

use super::{button, list_panel, page, row, row_at};
use crate::app::{App, Screen};

/// A zone the way a person reads it: `America / Sao Paulo`.
pub fn readable(zone: &str) -> String {
    zone.replace('_', " ").replace('/', " / ")
}

fn zone_list(app: &Rc<App>) -> ListBox {
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
        let mut state = app.state.borrow_mut();
        state.timezone = picked.widget_name().to_string();
        state.timezone_chosen = true;
    });
    list
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
    let list = zone_list(app);
    let entry = search(&list, text.timezone_search);
    let body = Box::new(Orientation::Vertical, 8);
    body.append(&entry);
    body.append(&list_panel(&list, 360));
    let next = button(text.next, "primary");
    let weak = Rc::downgrade(app);
    next.connect_clicked(move |_| {
        if let Some(app) = weak.upgrade() {
            app.go(Screen::Summary);
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
