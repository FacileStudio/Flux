use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box, ListBox, Orientation, Widget};

use super::{button, label, list_panel, page, row, row_at};
use crate::app::{App, Screen};
use crate::model::{human_size, Disk};
use crate::system;

fn disk_row(app: &Rc<App>, disk: &Disk) -> gtk4::ListBoxRow {
    let text = app.text();
    let mut tags = Vec::new();
    if disk.removable {
        tags.push(text.tag_removable);
    }
    if disk.live {
        tags.push(text.tag_live);
    }
    let title = if disk.model.is_empty() { disk.name.as_str() } else { disk.model.as_str() };
    let detail = [human_size(disk.size), disk.transport.to_uppercase(), disk.path.clone()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    let glyph = if disk.removable { "drive-removable-media-symbolic" } else { "drive-harddisk-symbolic" };
    let item = row(Some(glyph), title, Some(&detail), &tags);
    if disk.live {
        item.set_selectable(false);
        item.set_activatable(false);
        item.add_css_class("unavailable");
    }
    item
}

fn disk_list(app: &Rc<App>, disks: &[Disk], next: &gtk4::Button) -> ListBox {
    let list = ListBox::new();
    for disk in disks {
        list.append(&disk_row(app, disk));
    }
    let chosen = app.state.borrow().disk.clone();
    let index = chosen.and_then(|chosen| disks.iter().position(|disk| disk.path == chosen.path && !disk.live));
    match index {
        Some(index) => list.select_row(row_at(&list, index).as_ref()),
        None => app.state.borrow_mut().disk = None,
    }
    next.set_sensitive(index.is_some());
    let weak = Rc::downgrade(app);
    let disks = disks.to_vec();
    let next = next.clone();
    list.connect_row_selected(move |_, picked| {
        let (Some(app), Some(picked)) = (weak.upgrade(), picked) else {
            return;
        };
        let disk = usize::try_from(picked.index())
            .ok()
            .and_then(|index| disks.get(index))
            .filter(|disk| !disk.live)
            .cloned();
        next.set_sensitive(disk.is_some());
        app.state.borrow_mut().disk = disk;
    });
    list
}

pub fn build(app: &Rc<App>) -> Widget {
    let text = app.text();
    let disks = if app.state.borrow().preview { crate::app::preview_disks() } else { system::disks() };
    let next = super::next(text.next);
    let weak = Rc::downgrade(app);
    next.connect_clicked(move |_| {
        if let Some(app) = weak.upgrade() {
            app.go(Screen::Confirm);
        }
    });
    let body: Widget = if disks.is_empty() {
        next.set_sensitive(false);
        let empty = Box::new(Orientation::Vertical, 0);
        empty.add_css_class("panel");
        empty.add_css_class("empty");
        empty.append(&label(text.disk_none, "subtitle"));
        empty.upcast()
    } else {
        list_panel(&disk_list(app, &disks, &next), 280).upcast()
    };
    super::default(app, &next);
    page(app, text.disk_title, text.disk_subtitle, &body, &[&next])
}

pub fn confirm(app: &Rc<App>) -> Widget {
    let text = app.text();
    let disk = app.state.borrow().disk.as_ref().map(Disk::label).unwrap_or_default();
    let warning = Box::new(Orientation::Vertical, 8);
    warning.add_css_class("danger-panel");
    warning.set_valign(gtk4::Align::Start);
    warning.append(&label(&disk, "danger-title"));
    warning.append(&label(&text.confirm_body.replace("{disk}", &disk), "danger-body"));
    let erase = button("edit-delete-symbolic", text.confirm_button, "danger");
    let weak = Rc::downgrade(app);
    erase.connect_clicked(move |_| {
        if let Some(app) = weak.upgrade() {
            app.go(Screen::Account);
        }
    });
    page(app, text.confirm_title, text.disk_subtitle, &warning, &[&erase])
}
