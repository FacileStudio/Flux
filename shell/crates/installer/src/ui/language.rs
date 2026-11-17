use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box, Entry, ListBox, Orientation, Widget};

use super::{label, list_panel, page, row, row_at};
use crate::app::{App, Screen};
use crate::model::{KEYMAPS, LANGUAGES};
use crate::system;

fn languages(app: &Rc<App>) -> ListBox {
    let list = ListBox::new();
    for language in LANGUAGES {
        list.append(&row(None, language.name, None, &[]));
    }
    let selected = app.state.borrow().language;
    if let Some(current) = row_at(&list, selected) {
        list.select_row(Some(&current));
        let focus = current.clone();
        gtk4::glib::idle_add_local_once(move || {
            focus.grab_focus();
        });
    }
    let weak = Rc::downgrade(app);
    list.connect_row_selected(move |_, picked| {
        let (Some(app), Some(picked)) = (weak.upgrade(), picked) else {
            return;
        };
        let index = usize::try_from(picked.index()).unwrap_or(0);
        if index == app.state.borrow().language {
            return;
        }
        app.state.borrow_mut().set_language(index);
        let keymap = app.state.borrow().keymap.clone();
        system::try_layout(&keymap);
        app.replace(Screen::Language);
    });
    list
}

fn keymaps(app: &Rc<App>) -> ListBox {
    let list = ListBox::new();
    for (code, name) in KEYMAPS {
        let item = row(None, name, None, &[]);
        item.set_widget_name(code);
        list.append(&item);
    }
    let current = app.state.borrow().keymap.clone();
    if let Some(index) = KEYMAPS.iter().position(|(code, _)| *code == current) {
        list.select_row(row_at(&list, index).as_ref());
    }
    let weak = Rc::downgrade(app);
    list.connect_row_selected(move |_, picked| {
        let (Some(app), Some(picked)) = (weak.upgrade(), picked) else {
            return;
        };
        let code = picked.widget_name().to_string();
        let mut state = app.state.borrow_mut();
        state.keymap.clone_from(&code);
        state.keymap_chosen = true;
        drop(state);
        system::try_layout(&code);
    });
    list
}

fn column(heading: &str, content: &impl IsA<Widget>) -> Box {
    let column = Box::new(Orientation::Vertical, 8);
    column.set_hexpand(true);
    column.append(&label(heading, "heading"));
    column.append(content);
    column
}

pub fn build(app: &Rc<App>) -> Widget {
    let text = app.text();
    let body = Box::new(Orientation::Horizontal, 16);
    body.set_homogeneous(true);
    body.append(&column(text.language_heading, &list_panel(&languages(app), 360)));
    let keyboard = Box::new(Orientation::Vertical, 8);
    keyboard.append(&list_panel(&keymaps(app), 304));
    let test = Entry::new();
    test.set_placeholder_text(Some(text.keyboard_test));
    keyboard.append(&test);
    body.append(&column(text.keyboard_heading, &keyboard));
    let next = super::next(text.next);
    let weak = Rc::downgrade(app);
    next.connect_clicked(move |_| {
        if let Some(app) = weak.upgrade() {
            app.go(Screen::Disk);
        }
    });
    super::default(app, &next);
    page(app, text.language_title, text.language_subtitle, &body, &[&next])
}
