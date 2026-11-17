//! niri, through its IPC socket.
//!
//! The bar talks to `$NIRI_SOCKET` directly rather than through `niri msg`, so
//! it needs nothing on `PATH`. One thread holds the event stream: niri opens it
//! with the full state (workspaces, windows, keyboard layouts) and then sends
//! one JSON event per line. The thread parses each line and hands it to the
//! main loop, where [`State`] folds it in and every bar draws its own output's
//! view of it.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use serde::Deserialize;

/// How long the stream thread waits before reconnecting to a socket that went away.
const RECONNECT: Duration = Duration::from_secs(2);

/// One workspace, as niri reports it.
#[derive(Clone, Debug, Deserialize)]
pub struct Workspace {
    pub id: u64,
    pub idx: u8,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default)]
    pub is_urgent: bool,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub is_focused: bool,
    #[serde(default)]
    pub active_window_id: Option<u64>,
}

/// One window, reduced to what the bar shows.
#[derive(Clone, Debug, Deserialize)]
pub struct Window {
    pub id: u64,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub app_id: Option<String>,
    #[serde(default)]
    pub is_focused: bool,
}

/// The configured keyboard layouts and the one in use.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct KeyboardLayouts {
    pub names: Vec<String>,
    pub current_idx: u8,
}

/// The events the bar acts on. Any other event fails to parse and is skipped.
#[derive(Debug, Deserialize)]
pub enum Event {
    WorkspacesChanged { workspaces: Vec<Workspace> },
    WorkspaceUrgencyChanged { id: u64, urgent: bool },
    WorkspaceActivated { id: u64, focused: bool },
    WorkspaceActiveWindowChanged { workspace_id: u64, active_window_id: Option<u64> },
    WindowsChanged { windows: Vec<Window> },
    WindowOpenedOrChanged { window: Window },
    WindowClosed { id: u64 },
    WindowFocusChanged { id: Option<u64> },
    KeyboardLayoutsChanged { keyboard_layouts: KeyboardLayouts },
    KeyboardLayoutSwitched { idx: u8 },
}

/// Everything the bars draw from, kept current by [`State::apply`].
#[derive(Default)]
pub struct State {
    workspaces: Vec<Workspace>,
    windows: HashMap<u64, Window>,
    layouts: KeyboardLayouts,
}

/// One chip on a bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chip {
    pub id: u64,
    pub label: String,
    pub active: bool,
    pub focused: bool,
    pub urgent: bool,
}

/// What one bar shows: its output's workspaces in order, and the title of the
/// window in front on that output.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct View {
    pub chips: Vec<Chip>,
    pub title: Option<String>,
}

impl State {
    /// Fold one event in, returning whether anything a bar draws changed.
    pub fn apply(&mut self, event: Event) -> bool {
        match event {
            Event::WorkspacesChanged { workspaces } => self.workspaces = workspaces,
            Event::WorkspaceUrgencyChanged { id, urgent } => {
                let Some(workspace) = self.workspace(id) else {
                    return false;
                };
                workspace.is_urgent = urgent;
            }
            Event::WorkspaceActivated { id, focused } => self.activate(id, focused),
            Event::WorkspaceActiveWindowChanged { workspace_id, active_window_id } => {
                let Some(workspace) = self.workspace(workspace_id) else {
                    return false;
                };
                workspace.active_window_id = active_window_id;
            }
            Event::WindowsChanged { windows } => {
                self.windows = windows.into_iter().map(|window| (window.id, window)).collect();
            }
            Event::WindowOpenedOrChanged { window } => self.open(window),
            Event::WindowClosed { id } => {
                self.windows.remove(&id);
            }
            Event::WindowFocusChanged { id } => {
                for window in self.windows.values_mut() {
                    window.is_focused = Some(window.id) == id;
                }
            }
            Event::KeyboardLayoutsChanged { keyboard_layouts } => self.layouts = keyboard_layouts,
            Event::KeyboardLayoutSwitched { idx } => self.layouts.current_idx = idx,
        }
        true
    }

    fn workspace(&mut self, id: u64) -> Option<&mut Workspace> {
        self.workspaces.iter_mut().find(|workspace| workspace.id == id)
    }

    /// `id` becomes the active workspace of its output, and the focused one
    /// when `focused` says so. The other outputs keep their active workspace.
    fn activate(&mut self, id: u64, focused: bool) {
        let output = self
            .workspaces
            .iter()
            .find(|workspace| workspace.id == id)
            .map(|workspace| workspace.output.clone());
        let Some(output) = output else {
            return;
        };
        for workspace in &mut self.workspaces {
            if workspace.output == output {
                workspace.is_active = workspace.id == id;
            }
            if focused {
                workspace.is_focused = workspace.id == id;
            }
        }
    }

    /// A window that opened or changed. A newly focused one takes the focus
    /// from every other window, as niri's own model does.
    fn open(&mut self, window: Window) {
        if window.is_focused {
            for other in self.windows.values_mut() {
                other.is_focused = false;
            }
        }
        self.windows.insert(window.id, window);
    }

    /// The view for the bar on `output`. `None` stands for a monitor whose
    /// connector is unknown: it shows every workspace, ordered by output and
    /// index, and the focused window.
    pub fn view(&self, output: Option<&str>) -> View {
        let mut shown: Vec<&Workspace> = self
            .workspaces
            .iter()
            .filter(|workspace| output.is_none() || workspace.output.as_deref() == output)
            .collect();
        shown.sort_by(|a, b| (&a.output, a.idx).cmp(&(&b.output, b.idx)));
        let chips = shown
            .iter()
            .map(|workspace| Chip {
                id: workspace.id,
                label: workspace
                    .name
                    .clone()
                    .unwrap_or_else(|| workspace.idx.to_string()),
                active: workspace.is_active,
                focused: workspace.is_focused,
                urgent: workspace.is_urgent,
            })
            .collect();
        View {
            chips,
            title: self.title(output, &shown),
        }
    }

    /// The title of the window in front: the active window of the output's
    /// active workspace, or the focused window when the output is unknown.
    fn title(&self, output: Option<&str>, shown: &[&Workspace]) -> Option<String> {
        let window = match output {
            Some(_) => shown
                .iter()
                .find(|workspace| workspace.is_active)
                .and_then(|workspace| workspace.active_window_id)
                .and_then(|id| self.windows.get(&id)),
            None => self.windows.values().find(|window| window.is_focused),
        }?;
        window
            .title
            .clone()
            .or_else(|| window.app_id.clone())
            .filter(|title| !title.trim().is_empty())
    }

    /// The layout in use as a short code and its full name, when more than one
    /// layout is configured. One layout needs no indicator.
    pub fn layout(&self) -> Option<(String, String)> {
        if self.layouts.names.len() < 2 {
            return None;
        }
        let name = self.layouts.names.get(usize::from(self.layouts.current_idx))?;
        Some((short_layout(name), name.clone()))
    }
}

/// A two or three letter code for an xkb layout name: the country in brackets
/// when there is one ("English (US)" is `US`), the first two letters otherwise
/// ("French" is `FR`).
pub fn short_layout(name: &str) -> String {
    let bracket = name
        .split_once('(')
        .and_then(|(_, rest)| rest.split_once(')'))
        .map(|(inner, _)| inner.trim())
        .filter(|inner| (2..=3).contains(&inner.chars().count()) && inner.chars().all(char::is_alphabetic));
    match bracket {
        Some(code) => code.to_uppercase(),
        None => name.chars().filter(|c| c.is_alphabetic()).take(2).collect::<String>().to_uppercase(),
    }
}

/// The socket niri listens on, from `$NIRI_SOCKET`.
fn socket() -> Option<String> {
    std::env::var("NIRI_SOCKET").ok().filter(|path| !path.is_empty())
}

/// Whether this session runs under niri.
pub fn present() -> bool {
    socket().is_some()
}

/// Follow the event stream on a thread of its own, handing every event to `tx`,
/// and reconnect when the socket goes away. Ends when nobody listens.
pub fn follow(tx: async_channel::Sender<Event>) {
    let Some(path) = socket() else {
        return;
    };
    std::thread::spawn(move || loop {
        if let Err(error) = stream(&path, &tx) {
            eprintln!("statusbar: niri event stream: {error}");
        }
        if tx.is_closed() {
            return;
        }
        std::thread::sleep(RECONNECT);
    });
}

/// One connection's worth of events.
fn stream(path: &str, tx: &async_channel::Sender<Event>) -> std::io::Result<()> {
    let mut socket = UnixStream::connect(path)?;
    socket.write_all(b"\"EventStream\"\n")?;
    let mut lines = BufReader::new(socket).lines();
    lines.next().transpose()?;
    for line in lines {
        let Ok(event) = serde_json::from_str::<Event>(&line?) else {
            continue;
        };
        if tx.send_blocking(event).is_err() {
            return Ok(());
        }
    }
    Ok(())
}

/// The request that focuses the workspace with niri id `id`, on its own output.
pub fn focus_request(id: u64) -> String {
    format!(r#"{{"Action":{{"FocusWorkspace":{{"reference":{{"Id":{id}}}}}}}}}"#)
}

/// The request that moves to the next keyboard layout.
pub const NEXT_LAYOUT: &str = r#"{"Action":{"SwitchLayout":{"layout":"Next"}}}"#;

/// Send one request on a fresh connection, off the main loop.
pub fn send(request: String) {
    let Some(path) = socket() else {
        return;
    };
    std::thread::spawn(move || {
        let result = UnixStream::connect(&path).and_then(|mut socket| {
            socket.write_all(request.as_bytes())?;
            socket.write_all(b"\n")?;
            let mut reply = String::new();
            BufReader::new(socket).read_line(&mut reply)?;
            Ok(reply)
        });
        match result {
            Ok(reply) if reply.starts_with("{\"Ok\"") => {}
            Ok(reply) => eprintln!("statusbar: niri refused {request}: {}", reply.trim()),
            Err(error) => eprintln!("statusbar: niri request failed: {error}"),
        }
    });
}

#[cfg(test)]
mod tests;
