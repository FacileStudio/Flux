//! The window, what the person chose so far, and moving between screens.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{gdk, glib, Application, ApplicationWindow, Stack, StackTransitionType};
use shell_common::theme;

use crate::i18n::{self, Text};
use crate::model::{self, Choices, Disk, LANGUAGES};
use crate::ui;

const STYLE: &str = include_str!("../style.css");

/// A screen of the installer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Language,
    Disk,
    Confirm,
    Account,
    TimeZone,
    Software,
    Summary,
    Progress,
    Error,
    Done,
}

/// How many numbered steps the person walks through before the install starts.
pub const STEPS: u8 = 6;

impl Screen {
    /// The numbered step this screen belongs to, for the screens that have one.
    pub fn step(self) -> Option<u8> {
        match self {
            Screen::Language => Some(1),
            Screen::Disk | Screen::Confirm => Some(2),
            Screen::Account => Some(3),
            Screen::TimeZone => Some(4),
            Screen::Software => Some(5),
            Screen::Summary => Some(6),
            _ => None,
        }
    }

    fn has_back(self) -> bool {
        !matches!(self, Screen::Welcome | Screen::Progress | Screen::Done)
    }
}

/// Everything the person picked or typed.
#[derive(Default)]
pub struct State {
    pub language: usize,
    pub keymap: String,
    pub keymap_chosen: bool,
    pub disk: Option<Disk>,
    pub full_name: String,
    pub username: String,
    pub username_edited: bool,
    pub hostname: String,
    pub hostname_edited: bool,
    pub password: String,
    pub password_again: String,
    pub timezone: String,
    pub timezone_chosen: bool,
    pub zones: Vec<String>,
    pub points: Vec<model::ZonePoint>,
    pub software: model::Software,
    pub nvidia_gpu: bool,
    pub error: String,
    pub error_log: String,
    pub preview: bool,
}

impl State {
    fn new() -> Self {
        let language = &LANGUAGES[0];
        let nvidia_gpu = crate::system::nvidia_gpu();
        State {
            keymap: language.keymap.to_string(),
            timezone: language.timezone.to_string(),
            hostname: "flux".to_string(),
            zones: crate::system::zones(),
            points: crate::system::zone_points(),
            nvidia_gpu,
            software: model::Software { nvidia: nvidia_gpu, ..model::Software::default() },
            ..State::default()
        }
    }

    /// Pick language `index`, carrying its layout and zone unless the person chose others.
    pub fn set_language(&mut self, index: usize) {
        let Some(language) = LANGUAGES.get(index) else {
            return;
        };
        self.language = index;
        if !self.keymap_chosen {
            self.keymap = language.keymap.to_string();
        }
        if !self.timezone_chosen && self.zones.iter().any(|zone| zone == language.timezone) {
            self.timezone = language.timezone.to_string();
        }
    }

    /// What the backend gets.
    pub fn choices(&self) -> Choices {
        Choices {
            disk: self.disk.as_ref().map(|disk| disk.path.clone()).unwrap_or_default(),
            hostname: self.hostname.clone(),
            full_name: self.full_name.trim().to_string(),
            username: self.username.clone(),
            timezone: self.timezone.clone(),
            keymap: self.keymap.clone(),
            locale: LANGUAGES[self.language].locale.to_string(),
            software: model::Software { nvidia: self.software.nvidia && self.nvidia_gpu, ..self.software },
        }
    }

    /// Whether every choice the backend needs is made: a disk, a valid account and a zone.
    pub fn ready(&self) -> bool {
        self.disk.as_ref().is_some_and(|disk| !disk.path.is_empty() && !disk.live)
            && self.account_ready()
            && !self.timezone.is_empty()
    }

    /// Whether the account screen holds values the backend accepts.
    pub fn account_ready(&self) -> bool {
        model::valid_full_name(&self.full_name)
            && model::valid_username(&self.username)
            && model::valid_hostname(&self.hostname)
            && !self.password.is_empty()
            && self.password == self.password_again
    }
}

/// The installer window and its navigation.
pub struct App {
    pub window: ApplicationWindow,
    stack: Stack,
    pub state: RefCell<State>,
    history: RefCell<Vec<Screen>>,
    current: Cell<Screen>,
}

impl App {
    /// The strings for the chosen language.
    pub fn text(&self) -> &'static Text {
        i18n::text(LANGUAGES[self.state.borrow().language].code)
    }

    /// Open `screen`, remembering this one for Back.
    pub fn go(self: &Rc<Self>, screen: Screen) {
        self.history.borrow_mut().push(self.current.get());
        self.show(screen);
    }

    /// The screen on show.
    pub fn current(&self) -> Screen {
        self.current.get()
    }

    /// Whether the open screen offers Back.
    pub fn can_go_back(&self) -> bool {
        self.current.get().has_back()
    }

    /// Open `screen` in place of this one.
    pub fn replace(self: &Rc<Self>, screen: Screen) {
        self.show(screen);
    }

    /// Return to the previous screen, when this one allows it.
    pub fn back(self: &Rc<Self>) {
        if !self.current.get().has_back() {
            return;
        }
        let previous = self.history.borrow_mut().pop();
        if let Some(screen) = previous {
            self.show(screen);
        }
    }

    fn show(self: &Rc<Self>, screen: Screen) {
        self.current.set(screen);
        self.window.set_default_widget(None::<&gtk4::Widget>);
        let old = self.stack.visible_child();
        let page = ui::build(self, screen);
        self.stack.add_child(&page);
        self.stack.set_visible_child(&page);
        self.window.set_title(Some(self.text().window_title));
        if let Some(old) = old {
            let stack = self.stack.clone();
            glib::timeout_add_local_once(Duration::from_millis(400), move || {
                if old.parent().is_some() {
                    stack.remove(&old);
                }
            });
        }
    }
}

fn keys(app: &Rc<App>) {
    let controller = gtk4::EventControllerKey::new();
    let weak = Rc::downgrade(app);
    controller.connect_key_pressed(move |_, key, _, modifiers| {
        let Some(app) = weak.upgrade() else {
            return glib::Propagation::Proceed;
        };
        let alt_left = key == gdk::Key::Left && modifiers.contains(gdk::ModifierType::ALT_MASK);
        if key == gdk::Key::Escape || alt_left {
            app.back();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    app.window.add_controller(controller);
}

/// Build the window, or bring back the one already open.
pub fn activate(application: &Application) {
    if let Some(window) = application.active_window() {
        window.present();
        return;
    }
    theme::init(STYLE, "installer");
    let window = ApplicationWindow::builder()
        .application(application)
        .title(i18n::EN.window_title)
        .default_width(1280)
        .default_height(800)
        .build();
    window.add_css_class("installer");
    let stack = Stack::new();
    stack.set_transition_type(StackTransitionType::Crossfade);
    stack.set_transition_duration(200);
    window.set_child(Some(&stack));
    let app = Rc::new(App {
        window,
        stack,
        state: RefCell::new(State::new()),
        history: RefCell::new(Vec::new()),
        current: Cell::new(Screen::Welcome),
    });
    keys(&app);
    let owner = app.clone();
    app.window.connect_destroy(move |_| {
        let _ = &owner;
    });
    let start = std::env::var("FLUX_INSTALLER_PREVIEW").ok().and_then(|name| preview(&app, &name));
    app.replace(start.unwrap_or(Screen::Welcome));
    app.window.fullscreen();
    app.window.present();
}

/// The disks a preview lists in place of this machine's.
pub fn preview_disks() -> Vec<Disk> {
    let lsblk = r#"{"blockdevices":[
        {"name":"nvme0n1","path":"/dev/nvme0n1","size":512110190592,"model":"Samsung SSD 980 PRO","type":"disk","rm":false,"tran":"nvme"},
        {"name":"sdb","path":"/dev/sdb","size":2000398934016,"model":"WD Elements","type":"disk","rm":true,"tran":"usb"},
        {"name":"sda","path":"/dev/sda","size":31914983424,"model":"SanDisk Ultra","type":"disk","rm":true,"tran":"usb"}
    ]}"#;
    model::parse_disks(lsblk, &["sda".to_string()])
}

/// Open a screen with sample choices, without running anything, for design checks:
/// `FLUX_INSTALLER_PREVIEW=fr:summary flux-installer`.
fn preview(app: &Rc<App>, name: &str) -> Option<Screen> {
    let (language, screen) = name.split_once(':').unwrap_or(("en", name));
    let screen = match screen {
        "welcome" => Screen::Welcome,
        "language" => Screen::Language,
        "disk" => Screen::Disk,
        "confirm" => Screen::Confirm,
        "account" => Screen::Account,
        "timezone" => Screen::TimeZone,
        "software" => Screen::Software,
        "summary" => Screen::Summary,
        "progress" => Screen::Progress,
        "error" => Screen::Error,
        "done" => Screen::Done,
        _ => return None,
    };
    let mut state = app.state.borrow_mut();
    state.preview = true;
    state.nvidia_gpu = true;
    state.software.nvidia = true;
    state.set_language(LANGUAGES.iter().position(|item| item.code == language).unwrap_or(0));
    state.full_name = "Ada Lovelace".to_string();
    state.username = "ada".to_string();
    state.hostname = "ada-flux".to_string();
    state.password = "flux".to_string();
    state.password_again = "flux".to_string();
    state.error = "the install step failed, see /tmp/flux-install.log".to_string();
    state.error_log = "copying path '/nix/store/7d0x-linux-6.18-modules' from 'auto?trusted=1'\nerror: writing to file: No space left on device".to_string();
    state.disk = Some(Disk {
        name: "nvme0n1".to_string(),
        path: "/dev/nvme0n1".to_string(),
        size: 512_110_190_592,
        model: "Samsung SSD 980 PRO".to_string(),
        removable: false,
        transport: "nvme".to_string(),
        live: false,
    });
    drop(state);
    app.history.borrow_mut().push(Screen::Welcome);
    Some(screen)
}

#[cfg(test)]
mod tests;
