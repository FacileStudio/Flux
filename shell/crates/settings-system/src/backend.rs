//! What the pages ask of the system. Reads go straight to the files
//! `flux-config` keeps; changes go through `pkexec flux-config`. Everything
//! that can block runs on a worker thread. With `FLUX_SETTINGS_PREVIEW` set,
//! every call answers from sample data and touches nothing.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;

use flux_config::flake::{self, Locked};
use flux_config::generations::{self, os_release, Generation};
use flux_config::paths::Paths;
use flux_config::progress::{Line, State, Step};
use flux_config::seed;
use flux_config::settings::{self, Settings};
use gtk4::{gio, glib};

/// The environment variable that fills the pages with sample data.
pub const PREVIEW_VAR: &str = "FLUX_SETTINGS_PREVIEW";

/// Whether the pages show sample data.
pub fn preview() -> bool {
    std::env::var_os(PREVIEW_VAR).is_some()
}

/// The preview state a page should open in, such as `progress` in `updates:progress`.
pub fn preview_state(page: &str) -> Option<String> {
    let value = std::env::var(PREVIEW_VAR).ok()?;
    let (name, state) = value.split_once(':')?;
    (name == page).then(|| state.to_string())
}

/// Run `work` on a worker thread and hand its result to `done` on the main thread.
pub fn spawn<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static, done: impl FnOnce(T) + 'static) {
    let handle = gio::spawn_blocking(work);
    glib::spawn_future_local(async move {
        if let Ok(value) = handle.await {
            done(value);
        }
    });
}

fn paths() -> Paths {
    Paths::from_env()
}

fn sample() -> Settings {
    let mut settings = Settings { hostname: "ada-laptop".to_string(), timezone: Some("Europe/Paris".to_string()), ..Settings::default() };
    let _ = settings.add_user("ada", "Ada Lovelace", true, &[]);
    let _ = settings.add_user("grace", "Grace Hopper", false, &[]);
    let _ = settings.add_user("alan", "Alan Turing", false, &[]);
    settings
}

/// The machine settings: the file, or what the system says before the file exists.
pub fn settings() -> Settings {
    if preview() {
        return sample();
    }
    let paths = paths();
    match settings::load(&paths.settings(), None) {
        Ok(Some(settings)) => settings,
        _ => seed::from_system(&paths),
    }
}

/// The login name of the person using Settings.
pub fn me() -> String {
    if preview() {
        return "ada".to_string();
    }
    std::env::var("USER").unwrap_or_default()
}

/// The canonical zones, `UTC` first.
pub fn zones() -> Vec<String> {
    let dir = paths().zoneinfo();
    let tab = std::fs::read_to_string(dir.join("zone1970.tab"))
        .or_else(|_| std::fs::read_to_string(dir.join("zone.tab")))
        .unwrap_or_default();
    let mut zones: Vec<String> = tab
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split('\t').nth(2))
        .map(|zone| zone.trim().to_string())
        .filter(|zone| !zone.is_empty())
        .collect();
    zones.sort();
    zones.dedup();
    if zones.is_empty() && preview() {
        zones = ["America/New_York", "Asia/Tokyo", "Europe/Berlin", "Europe/London", "Europe/Paris"].map(String::from).to_vec();
    }
    zones.insert(0, "UTC".to_string());
    zones
}

fn capture(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).stdin(Stdio::null()).stderr(Stdio::null()).output().ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Whether network time is on, or `None` without timedated.
pub fn ntp() -> Option<bool> {
    if preview() {
        return Some(true);
    }
    capture("timedatectl", &["show", "-p", "NTP", "--value"]).map(|value| value == "yes")
}

/// Switch network time; timedated asks polkit itself.
pub fn set_ntp(on: bool) -> bool {
    preview() || capture("timedatectl", &["set-ntp", if on { "true" } else { "false" }]).is_some()
}

/// Whether the clock shows 24 hours, or `None` without the GNOME interface schema.
pub fn clock_24h() -> Option<bool> {
    if preview() {
        return Some(true);
    }
    capture("gsettings", &["get", "org.gnome.desktop.interface", "clock-format"]).map(|value| value.contains("24h"))
}

/// Set the clock format for this person.
pub fn set_clock_24h(on: bool) -> bool {
    preview() || capture("gsettings", &["set", "org.gnome.desktop.interface", "clock-format", if on { "24h" } else { "12h" }]).is_some()
}

/// What the Updates page shows at the top.
#[derive(Clone, Debug, Default)]
pub struct Installed {
    pub version: String,
    pub locked: Option<Locked>,
    pub restart: bool,
    pub automatic: bool,
}

fn link(path: PathBuf) -> Option<String> {
    std::fs::read_link(path).ok().map(|target| target.to_string_lossy().into_owned())
}

/// The installed version, the locked Flux revision and whether a restart is pending.
pub fn installed() -> Installed {
    if preview() {
        return Installed {
            version: "Flux 0.1.0".to_string(),
            locked: flake::parse_lock(include_str!("../../flux-config/tests/fixtures/flake.lock")),
            restart: preview_state("updates").as_deref() == Some("restart"),
            automatic: true,
        };
    }
    let paths = paths();
    let version = std::fs::read_to_string(paths.os_release())
        .ok()
        .and_then(|document| os_release(&document, "PRETTY_NAME"))
        .unwrap_or_else(|| "Flux".to_string());
    let locked = std::fs::read_to_string(paths.lock_file()).ok().and_then(|document| flake::parse_lock(&document));
    let parts = ["kernel", "kernel-modules", "initrd"];
    let booted: Vec<Option<String>> = parts.iter().map(|part| link(paths.booted().join(part))).collect();
    let current: Vec<Option<String>> = parts.iter().map(|part| link(paths.current().join(part))).collect();
    Installed {
        version,
        locked,
        restart: flux_config::progress::needs_restart(&booted, &current),
        automatic: settings().updates.automatic,
    }
}

/// The result of looking for a newer Flux.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Check {
    UpToDate,
    Available,
    Failed,
}

/// Ask `flux-config check`, which needs no administrator rights.
pub fn check() -> Check {
    if preview() {
        std::thread::sleep(std::time::Duration::from_millis(600));
        return Check::Available;
    }
    let Some(output) = capture(&helper(), &["check"]) else {
        return Check::Failed;
    };
    match serde_json::from_str::<serde_json::Value>(&output).ok().and_then(|value| value.get("available")?.as_bool()) {
        Some(true) => Check::Available,
        Some(false) => Check::UpToDate,
        None => Check::Failed,
    }
}

/// The system versions, newest first.
pub fn generations() -> Vec<Generation> {
    if preview() {
        let day = 86_400;
        let base = 1_794_787_200;
        return vec![
            Generation { number: 14, date: base, version: "Flux 0.2.0".into(), kernel: "6.12.8".into(), current: true },
            Generation { number: 13, date: base - 9 * day, version: "Flux 0.1.1".into(), kernel: "6.12.8".into(), current: false },
            Generation { number: 12, date: base - 23 * day, version: "Flux 0.1.0".into(), kernel: "6.12.1".into(), current: false },
        ];
    }
    let paths = paths();
    generations::list(&paths.profiles(), &paths.system_profile())
}

/// The running operation, if a helper is still working on one.
pub fn running() -> Option<State> {
    if preview() {
        return None;
    }
    let state = State::parse(&std::fs::read_to_string(paths().state()).ok()?)?;
    let alive = PathBuf::from("/proc").join(state.pid.to_string()).exists();
    state.running(alive).then_some(state)
}

/// The page that shows `operation`: `system`, `users` or `updates`.
pub fn owner(operation: &str) -> &'static str {
    if operation.starts_with("user-") || operation == "set-password" {
        "users"
    } else if operation.starts_with("set-") {
        "system"
    } else {
        "updates"
    }
}

/// The latest state line, while reattached to an operation.
pub fn state() -> Option<State> {
    State::parse(&std::fs::read_to_string(paths().state()).ok()?)
}

/// The last `count` lines of the operation log.
pub fn log_tail(count: usize) -> String {
    if preview() {
        return "building the system configuration...\ncopying path '/nix/store/…-flux-shell-0.2.0' from 'https://cache.nixos.org'...\ncopying path '/nix/store/…-linux-6.12.8' from 'https://cache.nixos.org'...".to_string();
    }
    let log = std::fs::read_to_string(paths().log()).unwrap_or_default();
    let lines: Vec<&str> = log.lines().collect();
    lines[lines.len().saturating_sub(count)..].join("\n")
}

/// The helper binary by its real store path, which the polkit action names.
pub fn helper() -> String {
    let system = PathBuf::from("/run/current-system/sw/bin/flux-config");
    std::fs::canonicalize(&system)
        .ok()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| "flux-config".to_string())
}

/// What a running helper reports.
pub enum Message {
    Line(Line),
    Exit { code: i32, error: String },
}

/// pkexec's exit code when the person dismissed the password prompt.
pub const AUTH_DISMISSED: i32 = 126;

fn preview_run(sender: &Sender<Message>) {
    let steps = [
        (Step::Start, 0),
        (Step::Evaluate, 8),
        (Step::Build, 30),
        (Step::Build, 62),
        (Step::Activate, 88),
        (Step::Bootloader, 95),
        (Step::Done, 100),
    ];
    for (step, percent) in steps {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let _ = sender.send(Message::Line(Line::new(step, percent, "")));
    }
    let _ = sender.send(Message::Exit { code: 0, error: String::new() });
}

/// Run `pkexec flux-config <args>` on a worker thread, writing `input` to its
/// stdin and forwarding each progress line to `sender`.
pub fn run(args: Vec<String>, input: Option<String>, sender: Sender<Message>) {
    std::thread::spawn(move || {
        if preview() {
            preview_run(&sender);
            return;
        }
        let child = Command::new("pkexec")
            .arg(helper())
            .args(&args)
            .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let mut child = match child {
            Ok(child) => child,
            Err(error) => {
                let _ = sender.send(Message::Exit { code: -1, error: error.to_string() });
                return;
            }
        };
        if let (Some(input), Some(mut stdin)) = (input, child.stdin.take()) {
            let _ = stdin.write_all(input.as_bytes());
            let _ = stdin.write_all(b"\n");
        }
        let errors = child.stderr.take().map(|mut stderr| {
            std::thread::spawn(move || {
                let mut text = String::new();
                let _ = stderr.read_to_string(&mut text);
                text
            })
        });
        if let Some(stdout) = child.stdout.take() {
            for line in BufReader::new(stdout).lines().map_while(Result::ok).filter_map(|text| Line::parse(&text)) {
                let _ = sender.send(Message::Line(line));
            }
        }
        let code = child.wait().ok().and_then(|status| status.code()).unwrap_or(-1);
        let error = errors.and_then(|handle| handle.join().ok()).unwrap_or_default();
        let error = error.lines().rev().find(|line| !line.trim().is_empty()).unwrap_or("").trim_start_matches("flux-config: ").to_string();
        let _ = sender.send(Message::Exit { code, error });
    });
}

/// Run a quick `pkexec flux-config <args>` to its end and say whether it worked. Blocks: call it through [`spawn`].
pub fn run_quick(args: &[&str]) -> bool {
    if preview() {
        return true;
    }
    Command::new("pkexec")
        .arg(helper())
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Restart the machine.
pub fn reboot() {
    if !preview() {
        let _ = Command::new("systemctl").arg("reboot").spawn();
    }
}
