//! The window: the searchable navigation on the left, the pages on the right.
//!
//! Pages are built the first time they are shown and kept in the stack, so a
//! page's background work starts only when someone opens it.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, glib, Application, ApplicationWindow, Orientation, Stack, StackTransitionType};
use shell_common::{theme, ui};

use crate::i18n;
use crate::registry::{self, PAGES};

const STYLE: &str = include_str!("../style.css");

struct App {
    window: ApplicationWindow,
    stack: Stack,
    list: gtk4::ListBox,
    built: RefCell<HashSet<&'static str>>,
}

thread_local! {
    static APP: RefCell<Option<Rc<App>>> = const { RefCell::new(None) };
}

impl App {
    /// Show the page called `id`, building it on its first visit.
    fn show(&self, index: usize) {
        let Some(page) = PAGES.get(index) else {
            return;
        };
        if self.built.borrow_mut().insert(page.id) {
            self.stack.add_named(&(page.build)(), Some(page.id));
        }
        self.stack.set_visible_child_name(page.id);
        let row = i32::try_from(index)
            .ok()
            .and_then(|index| self.list.row_at_index(index));
        if row.is_some() && self.list.selected_row() != row {
            self.list.select_row(row.as_ref());
        }
    }

    /// Show `id`, or the first page when it names none.
    fn show_id(&self, id: Option<&str>) {
        let index = id
            .and_then(|id| PAGES.iter().position(|page| page.id == id))
            .unwrap_or(0);
        self.show(index);
    }
}

/// Open the window on `page`, building it on the first call.
pub fn open(application: &Application, page: Option<&str>) {
    let app = APP.with(|slot| slot.borrow().clone()).unwrap_or_else(|| {
        let app = build(application);
        APP.with(|slot| *slot.borrow_mut() = Some(app.clone()));
        app
    });
    app.show_id(page);
    app.window.present();
}

/// One navigation row: the page's icon and title.
fn nav_row(page: &registry::Page) -> gtk4::ListBoxRow {
    let line = gtk4::Box::new(Orientation::Horizontal, 12);
    line.append(&ui::icon(page.icon, 16));
    let label = gtk4::Label::new(Some(&(page.title)()));
    label.set_xalign(0.0);
    line.append(&label);
    let row = gtk4::ListBoxRow::new();
    row.set_child(Some(&line));
    row
}

/// The navigation column: the search field over the page list.
fn navigation(app: &Rc<App>, search: &gtk4::SearchEntry) -> gtk4::Box {
    let column = gtk4::Box::new(Orientation::Vertical, 12);
    column.add_css_class("fx-nav");
    column.append(search);
    let empty = gtk4::Label::new(Some(i18n::text().no_results));
    empty.add_css_class("fx-subtitle");
    empty.set_visible(false);
    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scroll.set_vexpand(true);
    scroll.set_child(Some(&app.list));
    column.append(&scroll);
    column.append(&empty);
    filter(app, search, &empty);
    column
}

/// Narrow the page list to what `search` matches, and pick the first match on Enter.
fn filter(app: &Rc<App>, search: &gtk4::SearchEntry, empty: &gtk4::Label) {
    let query = Rc::new(RefCell::new(String::new()));
    let shown = query.clone();
    app.list.set_filter_func(move |row| {
        let page = usize::try_from(row.index())
            .ok()
            .and_then(|index| PAGES.get(index));
        page.is_some_and(|page| registry::matches(&shown.borrow(), &(page.title)(), page.id))
    });
    let weak = Rc::downgrade(app);
    let empty = empty.clone();
    search.connect_search_changed(move |search| {
        let Some(app) = weak.upgrade() else {
            return;
        };
        *query.borrow_mut() = search.text().to_string();
        app.list.invalidate_filter();
        let any = PAGES
            .iter()
            .any(|page| registry::matches(&query.borrow(), &(page.title)(), page.id));
        empty.set_visible(!any);
    });
    let weak = Rc::downgrade(app);
    search.connect_activate(move |_| {
        if let Some(app) = weak.upgrade() {
            open_first_match(&app);
        }
    });
}

/// Show the first page the search left in the list and move focus into it.
fn open_first_match(app: &App) {
    let first = (0..PAGES.len()).find(|&index| {
        i32::try_from(index)
            .ok()
            .and_then(|index| app.list.row_at_index(index))
            .is_some_and(|row| row.is_child_visible())
    });
    if let Some(index) = first {
        app.show(index);
        app.stack.child_focus(gtk4::DirectionType::TabForward);
    }
}

/// Ctrl+F searches, Ctrl+W and Ctrl+Q close.
fn shortcuts(window: &ApplicationWindow, search: &gtk4::SearchEntry) {
    let controller = gtk4::EventControllerKey::new();
    controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let search = search.clone();
    let target = window.clone();
    controller.connect_key_pressed(move |_, key, _, modifiers| {
        if !modifiers.contains(gdk::ModifierType::CONTROL_MASK) {
            return glib::Propagation::Proceed;
        }
        match key.to_lower() {
            gdk::Key::f => {
                search.grab_focus();
                glib::Propagation::Stop
            }
            gdk::Key::w | gdk::Key::q => {
                target.close();
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        }
    });
    window.add_controller(controller);
}

/// The window, the page stack and the navigation list, not yet wired together.
fn parts(application: &Application) -> App {
    let window = ApplicationWindow::builder()
        .application(application)
        .title(i18n::text().window_title)
        .default_width(980)
        .default_height(680)
        .build();
    window.add_css_class("fx-settings");
    window.set_decorated(false);
    let stack = Stack::new();
    stack.set_transition_type(StackTransitionType::Crossfade);
    stack.set_transition_duration(160);
    stack.set_hexpand(true);
    let list = gtk4::ListBox::new();
    list.add_css_class("fx-nav-list");
    PAGES.iter().for_each(|page| list.append(&nav_row(page)));
    App {
        window,
        stack,
        list,
        built: RefCell::new(HashSet::new()),
    }
}

fn build(application: &Application) -> Rc<App> {
    let css: &'static str = Box::leak(format!("{}\n{}", ui::CSS, STYLE).into_boxed_str());
    theme::init(css, "settings");
    let text = i18n::text();
    let app = Rc::new(parts(application));
    let weak = Rc::downgrade(&app);
    app.list.connect_row_selected(move |_, row| {
        if let (Some(app), Some(row)) = (weak.upgrade(), row) {
            if let Ok(index) = usize::try_from(row.index()) {
                app.show(index);
            }
        }
    });
    let search = gtk4::SearchEntry::new();
    search.set_placeholder_text(Some(text.search));
    search.add_css_class("fx-search");
    let root = gtk4::Box::new(Orientation::Horizontal, 0);
    root.append(&navigation(&app, &search));
    root.append(&app.stack);
    app.window.set_child(Some(&root));
    shortcuts(&app.window, &search);
    app
}
