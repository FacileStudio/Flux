//! The Updates page: the installed version, checking and installing updates,
//! automatic updates, and going back to a previous version.

use std::rc::Rc;

use flux_config::generations::Generation;
use flux_config::progress::Step;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation, Revealer, Spinner, Widget};

use crate::backend::{self, Check, Installed};
use crate::i18n::{self, fill, Text};
use crate::task::{self, Outcome, Task};
use crate::ui;

pub const ID: &str = "updates";
pub const ICON: &str = "software-update-available-symbolic";

/// The page name in the navigation.
pub fn title() -> String {
    i18n::text().updates_title.to_string()
}

/// The second line under the version: release day and short revision.
pub fn released(text: &Text, installed: &Installed) -> String {
    match &installed.locked {
        Some(locked) => fill(text.installed, &[("date", &i18n::date(text, locked.last_modified)), ("rev", locked.short())]),
        None => String::new(),
    }
}

/// The line under a previous version: its day and kernel.
pub fn generation_detail(text: &Text, generation: &Generation) -> String {
    let day = i18n::date(text, generation.date);
    if generation.kernel.is_empty() {
        day
    } else {
        format!("{day} · {}", fill(text.kernel, &[("kernel", &generation.kernel)]))
    }
}

struct Page {
    text: &'static Text,
    body: Box,
    task: Rc<Task>,
    status: Label,
    spinner: Spinner,
    check: Button,
    update: Revealer,
}

/// The whole page.
pub fn page() -> Widget {
    let text = i18n::text();
    let (scroll, column) = ui::system_page(text.updates_title, text.updates_subtitle);
    let task = Task::new(text);
    let status = ui::label(text.check_hint, "fx-status");
    status.set_hexpand(true);
    let spinner = Spinner::new();
    let check = ui::secondary_button(text.check);
    let update_button = ui::primary_button(text.update_now);
    let update = ui::reveal(&update_button);
    update.set_transition_type(gtk4::RevealerTransitionType::Crossfade);
    let body = Box::new(Orientation::Vertical, 24);
    body.append(&ui::spinner_row(text.updates_title));
    let page = Rc::new(Page { text, body, task, status, spinner, check, update });
    column.append(page.task.widget());
    column.append(&page.body);
    let weak = Rc::downgrade(&page);
    page.task.on_finish(move |outcome| {
        if let (Some(page), Outcome::Done { .. }) = (weak.upgrade(), outcome) {
            ui::show(&page.update, false);
            page.reload();
        }
    });
    let start = page.clone();
    update_button.connect_clicked(move |_| {
        start.check.set_sensitive(false);
        start.task.start(vec!["update".to_string()], None);
    });
    let checker = page.clone();
    page.check.connect_clicked(move |_| checker.run_check());
    page.reload();
    let attach = page.clone();
    backend::spawn(backend::running, move |state| {
        if let Some(state) = state.filter(|state| backend::owner(&state.operation) == ID) {
            attach.task.attach(&state.line);
        }
    });
    page.preview();
    scroll.upcast()
}

impl Page {
    fn preview(self: &Rc<Self>) {
        match backend::preview_state(ID).as_deref() {
            Some("progress") => self.task.sample(Step::Build, 54),
            Some("error") => self.task.sample_end(Outcome::Failed, "cannot reach the update server"),
            _ => {}
        }
    }

    fn reload(self: &Rc<Self>) {
        let page = self.clone();
        backend::spawn(|| (backend::installed(), backend::generations()), move |(installed, generations)| {
            page.fill(&installed, &generations);
        });
    }

    fn run_check(self: &Rc<Self>) {
        self.check.set_sensitive(false);
        self.spinner.set_visible(true);
        self.spinner.start();
        self.status.set_text(self.text.checking);
        let page = self.clone();
        backend::spawn(backend::check, move |result| {
            page.spinner.stop();
            page.spinner.set_visible(false);
            page.check.set_sensitive(true);
            let text = page.text;
            page.status.set_text(match result {
                Check::UpToDate => text.up_to_date,
                Check::Available => text.available,
                Check::Failed => text.check_failed,
            });
            ui::show(&page.update, result == Check::Available);
        });
    }

    fn fill(self: &Rc<Self>, installed: &Installed, generations: &[Generation]) {
        ui::clear(&self.body);
        self.body.append(&self.hero(installed));
        self.check.set_sensitive(!self.task.busy());
        if installed.restart && !self.task.busy() {
            self.body.append(&self.restart_banner());
        }
        self.body.append(&self.automatic(installed.automatic));
        self.body.append(&self.previous(generations));
        if !self.task.busy() && backend::preview_state(ID).is_none() {
            self.run_check();
        }
    }

    fn hero(self: &Rc<Self>, installed: &Installed) -> Box {
        let hero = Box::new(Orientation::Vertical, 16);
        hero.add_css_class("fx-hero");
        let names = Box::new(Orientation::Vertical, 4);
        names.append(&ui::label(&installed.version, "fx-version"));
        let line = released(self.text, installed);
        if !line.is_empty() {
            names.append(&ui::label(&line, "fx-subtitle"));
        }
        hero.append(&names);
        let status = Box::new(Orientation::Horizontal, 8);
        self.spinner.set_visible(false);
        detach(&self.spinner);
        detach(&self.status);
        status.append(&self.spinner);
        status.append(&self.status);
        hero.append(&status);
        let buttons = Box::new(Orientation::Horizontal, 8);
        detach(&self.check);
        detach(&self.update);
        buttons.append(&self.check);
        buttons.append(&self.update);
        hero.append(&buttons);
        hero
    }

    fn restart_banner(&self) -> Box {
        let banner = Box::new(Orientation::Horizontal, 12);
        banner.add_css_class("fx-banner");
        let label = ui::label(self.text.restart_hint, "fx-title");
        label.set_hexpand(true);
        label.set_valign(Align::Center);
        banner.append(&label);
        let restart = ui::primary_button(self.text.restart);
        restart.connect_clicked(|_| backend::reboot());
        banner.append(&restart);
        banner
    }

    fn automatic(self: &Rc<Self>, active: bool) -> Box {
        let text = self.text;
        let (section, card) = ui::section("");
        let page = self.clone();
        let (row, _) = ui::switch_row(text.auto_updates, text.auto_updates_hint, active, move |on| {
            let page = page.clone();
            let value = if on { "on" } else { "off" };
            backend::spawn(move || backend::run_quick(&["set", "auto-update", value]), move |ok| {
                if !ok {
                    page.reload();
                }
            });
        });
        card.append(&row);
        section
    }

    fn previous(self: &Rc<Self>, generations: &[Generation]) -> Box {
        let text = self.text;
        let (section, card) = ui::section(text.previous);
        let hint = ui::label(text.previous_hint, "fx-subtitle");
        hint.add_css_class("fx-section-hint");
        section.insert_child_after(&hint, section.first_child().as_ref());
        if generations.len() <= 1 {
            card.append(&ui::empty_state("document-open-recent-symbolic", text.previous, text.no_generations));
        }
        let open_confirm = backend::preview_state(ID).as_deref() == Some("confirm");
        for (index, generation) in generations.iter().enumerate() {
            let row = self.generation_row(generation);
            if open_confirm && index == 1 {
                if let Some(reveal) = row.last_child().and_downcast::<Revealer>() {
                    ui::show(&reveal, true);
                }
            }
            card.append(&row);
        }
        let clean = ui::secondary_button(text.clean_button);
        let page = self.clone();
        clean.connect_clicked(move |_| page.task.start(vec!["gc".to_string()], None));
        clean.set_sensitive(generations.len() > 3);
        card.append(&ui::row(text.clean, text.clean_hint, &clean));
        section
    }

    fn generation_row(self: &Rc<Self>, generation: &Generation) -> Box {
        let text = self.text;
        let column = Box::new(Orientation::Vertical, 0);
        let detail = generation_detail(text, generation);
        let name = if generation.version.is_empty() { format!("#{}", generation.number) } else { generation.version.clone() };
        let suffix: Widget = if generation.current {
            let tag = Label::new(Some(text.current));
            tag.add_css_class("fx-tag");
            tag.upcast()
        } else {
            ui::secondary_button(text.go_back).upcast()
        };
        column.append(&ui::row(&name, &detail, &suffix));
        if let Some(button) = suffix.downcast_ref::<Button>() {
            let title = fill(text.go_back_title, &[("version", &name)]);
            let (confirm, no, yes) = task::confirm(&title, text.go_back_body, text.cancel, text.go_back);
            confirm.add_css_class("fx-confirm-calm");
            let reveal = ui::reveal(&confirm);
            let open = reveal.clone();
            button.connect_clicked(move |_| ui::show(&open, true));
            let close = reveal.clone();
            no.connect_clicked(move |_| ui::show(&close, false));
            let page = self.clone();
            let number = generation.number.to_string();
            let hide = reveal.clone();
            yes.connect_clicked(move |_| {
                ui::show(&hide, false);
                page.task.start(vec!["rollback".to_string(), number.clone()], None);
            });
            column.append(&reveal);
        }
        column
    }
}

fn detach(widget: &impl IsA<Widget>) {
    if let Some(parent) = widget.parent().and_downcast::<Box>() {
        parent.remove(widget);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{EN, FR};

    #[test]
    fn lines() {
        let installed = Installed {
            locked: flux_config::flake::parse_lock(include_str!("../../flux-config/tests/fixtures/flake.lock")),
            ..Installed::default()
        };
        assert_eq!(released(&EN, &installed), "Released 16 Nov 2025 · revision b00595e");
        assert_eq!(released(&EN, &Installed::default()), "");
        let generation = Generation { number: 3, date: 0, version: "Flux 0.1.0".into(), kernel: "6.12.1".into(), current: false };
        assert_eq!(generation_detail(&FR, &generation), "1 janv. 1970 · noyau 6.12.1");
    }
}
