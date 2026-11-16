//! Finding the applications installed on this machine.
//!
//! The launcher reads the XDG data directories in precedence order and takes
//! the first `.desktop` file that claims each name, which is what the desktop
//! entry specification asks for. On NixOS the applications live under
//! `$XDG_DATA_DIRS` (`/run/current-system/sw/share`, the user profiles), so
//! that variable is what the list is built from.

use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// How many results the launcher shows at once.
const MAX_MATCHES: usize = 10;

/// The data directories the specification falls back to when `$XDG_DATA_DIRS`
/// is unset or empty.
const DEFAULT_DATA_DIRS: &str = "/usr/local/share:/usr/share";

/// One launchable application, as its `.desktop` file describes it.
pub struct App {
    pub name: String,
    pub generic: Option<String>,
    pub exec: String,
    pub icon: Option<String>,
    pub terminal: bool,
}

/// What the process environment says about where and for whom to look.
pub struct Env {
    pub data_home: Option<OsString>,
    pub home: Option<OsString>,
    pub data_dirs: Option<OsString>,
    pub desktops: Vec<String>,
    pub path: Option<OsString>,
}

impl Env {
    /// The environment of this process.
    pub fn current() -> Self {
        Self {
            data_home: std::env::var_os("XDG_DATA_HOME"),
            home: std::env::var_os("HOME"),
            data_dirs: std::env::var_os("XDG_DATA_DIRS"),
            desktops: std::env::var("XDG_CURRENT_DESKTOP")
                .unwrap_or_default()
                .split(':')
                .filter(|name| !name.is_empty())
                .map(str::to_ascii_lowercase)
                .collect(),
            path: std::env::var_os("PATH"),
        }
    }
}

/// Every application found on this machine, sorted by name.
pub fn discover(env: &Env) -> Vec<App> {
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for dir in data_dirs(env) {
        for path in unclaimed(&dir, &mut seen) {
            if let Some(app) = parse(&path, env) {
                found.push(app);
            }
        }
    }
    found.sort_by_key(|app| app.name.to_lowercase());
    found
}

/// The `.desktop` files in one directory, minus the names an earlier directory
/// already contributed: the earlier directory wins.
fn unclaimed(dir: &Path, seen: &mut HashSet<String>) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("desktop"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| seen.insert(name.to_string_lossy().into_owned()))
        })
        .collect()
}

/// The indices of the apps matching `query`, capped at [`MAX_MATCHES`]. An
/// empty query matches everything, so the launcher opens on the top of the list.
pub fn matches(apps: &[App], query: &str) -> Vec<usize> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return (0..apps.len()).take(MAX_MATCHES).collect();
    }
    apps.iter()
        .enumerate()
        .filter(|(_, app)| {
            app.name.to_lowercase().contains(&needle)
                || app
                    .generic
                    .as_deref()
                    .is_some_and(|generic| generic.to_lowercase().contains(&needle))
        })
        .map(|(index, _)| index)
        .take(MAX_MATCHES)
        .collect()
}

/// Where `.desktop` files live, most specific first: `$XDG_DATA_HOME` (or
/// `~/.local/share`), then every entry of `$XDG_DATA_DIRS`, each with
/// `applications` appended and each named once.
pub fn data_dirs(env: &Env) -> Vec<PathBuf> {
    let home = env
        .data_home
        .clone()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| env.home.clone().map(|home| PathBuf::from(home).join(".local/share")));
    let system = env
        .data_dirs
        .clone()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_DATA_DIRS.into());
    let mut dirs: Vec<PathBuf> = Vec::new();
    for base in home.into_iter().chain(std::env::split_paths(&system)) {
        let dir = base.join("applications");
        if base.is_absolute() && !dirs.contains(&dir) {
            dirs.push(dir);
        }
    }
    dirs
}

/// Read one `.desktop` file. `None` when the entry is not an application, is
/// hidden, is not meant for this desktop, names a missing `TryExec`, or names no
/// program: a row that cannot be launched is worse than no row.
fn parse(path: &Path, env: &Env) -> Option<App> {
    let text = std::fs::read_to_string(path).ok()?;
    entry_from(&text).into_app(env)
}

/// The `[Desktop Entry]` group of a `.desktop` file. Everything after it is a
/// different matter (the actions, and the localized keys like `Name[fr]`) and
/// is skipped.
fn entry_from(text: &str) -> Entry {
    let mut entry = Entry::default();
    let mut in_entry_group = false;
    for line in text.lines() {
        if line.starts_with('[') {
            in_entry_group = line.trim() == "[Desktop Entry]";
            continue;
        }
        if line.starts_with('#') || !in_entry_group {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            entry.apply(key.trim(), value.trim());
        }
    }
    entry
}

/// The fields the launcher reads out of a `.desktop` file.
#[derive(Default)]
struct Entry {
    kind: Option<String>,
    name: Option<String>,
    generic: Option<String>,
    exec: Option<String>,
    try_exec: Option<String>,
    icon: Option<String>,
    terminal: bool,
    hidden: bool,
    only_show_in: Option<Vec<String>>,
    not_show_in: Vec<String>,
}

impl Entry {
    /// Fold one `key=value` line in. Keys we do not render are not an error:
    /// the specification defines many more than the launcher shows.
    fn apply(&mut self, key: &str, value: &str) {
        match key {
            "Type" => self.kind = Some(value.to_string()),
            "Name" => self.name = Some(unescape(value)),
            "GenericName" => self.generic = Some(unescape(value)),
            "Exec" => self.exec = Some(value.to_string()),
            "TryExec" => self.try_exec = Some(unescape(value)),
            "Icon" => self.icon = Some(unescape(value)).filter(|icon| !icon.is_empty()),
            "Terminal" => self.terminal = value.eq_ignore_ascii_case("true"),
            "NoDisplay" | "Hidden" => self.hidden |= value.eq_ignore_ascii_case("true"),
            "OnlyShowIn" => self.only_show_in = Some(list(value)),
            "NotShowIn" => self.not_show_in = list(value),
            _ => {}
        }
    }

    /// The application this entry describes, when it is one the launcher shows.
    fn into_app(self, env: &Env) -> Option<App> {
        if self.hidden || self.kind.as_deref().is_some_and(|kind| kind != "Application") {
            return None;
        }
        if !shown_in(self.only_show_in.as_deref(), &self.not_show_in, &env.desktops) {
            return None;
        }
        if let Some(program) = &self.try_exec {
            if !on_path(program, env.path.as_deref()) {
                return None;
            }
        }
        Some(App {
            name: self.name?,
            generic: self.generic,
            exec: self.exec.filter(|exec| !exec.trim().is_empty())?,
            icon: self.icon,
            terminal: self.terminal,
        })
    }
}

/// Whether an entry with these `OnlyShowIn` and `NotShowIn` lists belongs on
/// one of `desktops`, compared without case.
fn shown_in(only: Option<&[String]>, not: &[String], desktops: &[String]) -> bool {
    if not.iter().any(|name| desktops.contains(name)) {
        return false;
    }
    only.is_none_or(|only| only.iter().any(|name| desktops.contains(name)))
}

/// A `;`-separated list value, lowercased, empty items dropped.
fn list(value: &str) -> Vec<String> {
    value
        .split(';')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

/// Whether `program` is an executable file: as given when it is a path, or in
/// one of the `path` directories otherwise.
fn on_path(program: &str, path: Option<&std::ffi::OsStr>) -> bool {
    if program.contains('/') {
        return is_executable(Path::new(program));
    }
    path.is_some_and(|path| std::env::split_paths(path).any(|dir| is_executable(&dir.join(program))))
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

/// Resolve the escapes the desktop entry specification defines in string
/// values.
fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(current) = chars.next() {
        if current != '\\' {
            out.push(current);
            continue;
        }
        match chars.next() {
            Some('s') => out.push(' '),
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some(other) => out.push(other),
            None => break,
        }
    }
    out
}

#[cfg(test)]
mod tests;
