use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box, Orientation, Switch, Widget};
use shell_common::ui::icon;

use super::{label, network, page};
use crate::app::{App, Screen};
use crate::model::{self, Software};

fn card(glyph: &str, title: &str, body: &str, tag: Option<&str>, on: bool, set: impl Fn(bool) + 'static) -> Box {
    let line = Box::new(Orientation::Horizontal, 16);
    line.add_css_class("software");
    let image = icon(glyph, 24);
    image.add_css_class("software-icon");
    image.set_valign(Align::Start);
    line.append(&image);
    let words = Box::new(Orientation::Vertical, 4);
    words.set_hexpand(true);
    let heading = Box::new(Orientation::Horizontal, 8);
    let name = label(title, "row-title");
    name.set_wrap(false);
    heading.append(&name);
    if let Some(tag) = tag {
        let tag = label(tag, "tag");
        tag.set_wrap(false);
        heading.append(&tag);
    }
    words.append(&heading);
    words.append(&label(body, "dim"));
    line.append(&words);
    let switch = Switch::new();
    switch.set_active(on);
    switch.set_valign(Align::Center);
    switch.connect_active_notify(move |switch| set(switch.is_active()));
    line.append(&switch);
    line
}

fn choose(app: &Rc<App>, redraw: &Rc<dyn Fn()>, change: fn(&mut Software, bool)) -> impl Fn(bool) + 'static {
    let weak = Rc::downgrade(app);
    let redraw = redraw.clone();
    move |on| {
        if let Some(app) = weak.upgrade() {
            change(&mut app.state.borrow_mut().software, on);
            redraw();
        }
    }
}

pub fn build(app: &Rc<App>) -> Widget {
    let text = app.text();
    let (software, nvidia_gpu) = {
        let state = app.state.borrow();
        (state.software, state.nvidia_gpu)
    };
    let (notice, redraw) = network::build(app, true, |_| ());
    let panel = Box::new(Orientation::Vertical, 4);
    panel.add_css_class("panel");
    panel.set_valign(Align::Start);
    panel.append(&card(
        "applications-engineering-symbolic",
        text.software_facile,
        text.software_facile_body,
        None,
        software.facile,
        choose(app, &redraw, |software, on| software.facile = on),
    ));
    panel.append(&card(
        "folder-open-symbolic",
        text.software_apps,
        text.software_apps_body,
        None,
        software.apps,
        choose(app, &redraw, |software, on| software.apps = on),
    ));
    if model::X86 {
        panel.append(&card(
            "input-gaming-symbolic",
            text.software_gaming,
            text.software_gaming_body,
            None,
            software.gaming,
            choose(app, &redraw, |software, on| software.gaming = on),
        ));
    }
    if nvidia_gpu {
        panel.append(&card(
            "video-display-symbolic",
            text.software_nvidia,
            text.software_nvidia_body,
            Some(text.recommended),
            software.nvidia,
            choose(app, &redraw, |software, on| software.nvidia = on),
        ));
    }
    let body = Box::new(Orientation::Vertical, 12);
    body.append(&panel);
    body.append(&notice);
    let next = super::next(text.next);
    let weak = Rc::downgrade(app);
    next.connect_clicked(move |_| {
        if let Some(app) = weak.upgrade() {
            app.go(Screen::Summary);
        }
    });
    super::default(app, &next);
    page(app, text.software_title, text.software_subtitle, &body, &[&next])
}
