//! The keyboard layouts list: reorder, remove, pick a variant, and add one
//! from the xkb registry through a searchable popover.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, ListBox, Orientation, Popover, ScrolledWindow, SearchEntry};

use crate::model::config::Layout;
use crate::model::xkb::{LayoutEntry, Registry};
use crate::ui;

use super::Input;

fn icon_button(icon: &str, label: &str) -> Button {
    let button = Button::from_icon_name(icon);
    button.add_css_class("fx-icon");
    button.set_tooltip_text(Some(label));
    button.update_property(&[gtk4::accessible::Property::Label(label)]);
    button
}

fn variant_dropdown(input: &Rc<Input>, index: usize, layout: &Layout, entry: Option<&LayoutEntry>) -> gtk4::DropDown {
    let variants = entry.map(|entry| entry.variants.clone()).unwrap_or_default();
    let mut names = vec![input.text.variant_default.to_string()];
    names.extend(variants.iter().map(|variant| variant.description.clone()));
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let dropdown = gtk4::DropDown::from_strings(&refs);
    dropdown.set_enable_search(variants.len() > 8);
    dropdown.update_property(&[gtk4::accessible::Property::Label(input.text.variant)]);
    let selected = variants.iter().position(|v| v.name == layout.variant).map(|i| i + 1).unwrap_or(0);
    dropdown.set_selected(selected as u32);
    dropdown.set_sensitive(!variants.is_empty());
    let this = input.clone();
    dropdown.connect_selected_notify(move |dropdown| {
        let variant = match dropdown.selected() as usize {
            0 => String::new(),
            n => variants.get(n - 1).map(|v| v.name.clone()).unwrap_or_default(),
        };
        this.change_layouts(|layouts| {
            if let Some(layout) = layouts.get_mut(index) {
                layout.variant = variant;
            }
        });
    });
    dropdown
}

fn layout_row(input: &Rc<Input>, index: usize, count: usize, layout: &Layout) -> GtkBox {
    let registry = input.registry.as_ref();
    let entry = registry.and_then(|registry| registry.layout(&layout.layout));
    let title = entry.map(|entry| entry.item.description.clone()).unwrap_or_else(|| layout.layout.clone());
    let suffix = GtkBox::new(Orientation::Horizontal, 4);
    suffix.append(&variant_dropdown(input, index, layout, entry));
    let up = icon_button("go-up-symbolic", input.text.move_up);
    up.set_sensitive(index > 0);
    let this = input.clone();
    up.connect_clicked(move |_| this.change_layouts(|layouts| layouts.swap(index, index.saturating_sub(1))));
    let remove = icon_button("list-remove-symbolic", input.text.remove);
    remove.set_sensitive(count > 1);
    let this = input.clone();
    remove.connect_clicked(move |_| {
        this.change_layouts(|layouts| {
            if layouts.len() > 1 && index < layouts.len() {
                layouts.remove(index);
            }
        })
    });
    suffix.append(&up);
    suffix.append(&remove);
    ui::row(&title, None, &suffix)
}

fn chooser(input: &Rc<Input>, registry: &Registry, popover: &Popover) -> GtkBox {
    let body = GtkBox::new(Orientation::Vertical, 8);
    body.add_css_class("fx-chooser");
    let search = SearchEntry::builder().placeholder_text(input.text.search_layout).build();
    let list = ListBox::new();
    list.add_css_class("fx-list");
    let names: Vec<String> = registry.layouts.iter().map(|entry| entry.item.name.clone()).collect();
    for entry in &registry.layouts {
        let label = Label::builder().label(&entry.item.description).xalign(0.0).build();
        list.append(&label);
    }
    let query = search.clone();
    let descriptions: Vec<String> = registry.layouts.iter().map(|e| e.item.description.to_lowercase()).collect();
    let codes = names.clone();
    list.set_filter_func(move |row| {
        let needle = query.text().to_lowercase();
        let index = row.index().max(0) as usize;
        needle.is_empty()
            || descriptions.get(index).is_some_and(|d| d.contains(&needle))
            || codes.get(index).is_some_and(|c| c.starts_with(&needle))
    });
    let filtered = list.clone();
    search.connect_search_changed(move |_| filtered.invalidate_filter());
    let this = input.clone();
    let shown = popover.clone();
    list.connect_row_activated(move |_, row| {
        let Some(name) = names.get(row.index().max(0) as usize).cloned() else {
            return;
        };
        shown.popdown();
        this.change_layouts(|layouts| {
            if !layouts.iter().any(|l| l.layout == name && l.variant.is_empty()) {
                layouts.push(Layout { layout: name, variant: String::new() });
            }
        });
    });
    let scroller = ScrolledWindow::builder()
        .child(&list)
        .min_content_height(280)
        .min_content_width(320)
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .build();
    body.append(&search);
    body.append(&scroller);
    body
}

/// Fill `rows` with one row per layout and the add button.
pub fn fill(input: &Rc<Input>, rows: &GtkBox) {
    ui::clear(rows);
    let layouts = input.layouts();
    for (index, layout) in layouts.iter().enumerate() {
        rows.append(&layout_row(input, index, layouts.len(), layout));
    }
    let Some(registry) = input.registry.as_ref() else {
        rows.append(&ui::row(input.text.no_registry, None, &GtkBox::new(Orientation::Horizontal, 0)));
        return;
    };
    let add = gtk4::MenuButton::builder().label(input.text.add_layout).css_classes(["fx-button", "ghost"]).build();
    let popover = Popover::new();
    popover.set_child(Some(&chooser(input, registry, &popover)));
    add.set_popover(Some(&popover));
    add.set_halign(gtk4::Align::Start);
    add.add_css_class("fx-add");
    rows.append(&add);
    if crate::backend::preview_scene() == "add" {
        gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(1500), move || add.popup());
    }
}
