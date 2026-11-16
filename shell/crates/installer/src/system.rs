//! What the installer asks of the running system: disks, zones, the live
//! keyboard layout, the password hash and the backend process.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Sender;

use crate::model::{self, Choices, Disk, Event, Step};

pub const LOG: &str = "/tmp/flux-install.log";

/// What the install thread reports to the window.
pub enum Message {
    Event(Event),
    Failed(Failure),
}

/// Why the install did not reach the backend's own error line.
pub enum Failure {
    Start(String),
    Hash(String),
    Exit(i32, String),
}

/// The whole disks on this machine, with the live medium flagged.
pub fn disks() -> Vec<Disk> {
    let output = Command::new("lsblk")
        .args(["-J", "-b", "-d", "-o", "NAME,PATH,SIZE,MODEL,TYPE,RM,TRAN"])
        .output();
    let json = output.map(|out| String::from_utf8_lossy(&out.stdout).into_owned()).unwrap_or_default();
    model::parse_disks(&json, &live_disks())
}

fn live_disks() -> Vec<String> {
    let mounts = std::fs::read_to_string("/proc/mounts").unwrap_or_default();
    let Some(source) = model::iso_source(&mounts) else {
        return Vec::new();
    };
    let device = std::fs::canonicalize(&source).unwrap_or_else(|_| PathBuf::from(&source));
    let Some(name) = device.file_name().and_then(|name| name.to_str()) else {
        return Vec::new();
    };
    let block = Path::new("/sys/class/block").join(name);
    if !block.join("partition").exists() {
        return vec![name.to_string()];
    }
    std::fs::canonicalize(&block)
        .ok()
        .and_then(|path| path.parent()?.file_name()?.to_str().map(String::from))
        .into_iter()
        .collect()
}

fn zoneinfo() -> Option<PathBuf> {
    std::env::var_os("TZDIR")
        .map(PathBuf::from)
        .into_iter()
        .chain(["/etc/zoneinfo", "/usr/share/zoneinfo"].map(PathBuf::from))
        .find(|dir| dir.is_dir())
}

/// Every canonical time zone the system knows, `UTC` first.
pub fn zones() -> Vec<String> {
    let Some(dir) = zoneinfo() else {
        return vec!["UTC".to_string()];
    };
    let tab = std::fs::read_to_string(dir.join("zone1970.tab"))
        .or_else(|_| std::fs::read_to_string(dir.join("zone.tab")))
        .unwrap_or_default();
    model::parse_zone_tab(&tab)
}

/// Try `layout` in the live niri session through a user config niri reloads.
pub fn try_layout(layout: &str) {
    let Ok(system) = std::fs::read_to_string("/etc/niri/config.kdl") else {
        return;
    };
    let Some(config) = model::niri_with_layout(&system, layout) else {
        return;
    };
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let dir = PathBuf::from(home).join(".config/niri");
    if std::fs::create_dir_all(&dir).is_ok() && std::fs::write(dir.join("config.kdl"), config).is_ok() {
        let _ = Command::new("niri")
            .args(["msg", "action", "load-config-file"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

/// The sha-512 crypt hash of `password`, from `mkpasswd` reading stdin.
fn hash(password: &str) -> Result<String, String> {
    let mut child = Command::new("mkpasswd")
        .args(["-s", "-m", "sha-512"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    child
        .stdin
        .take()
        .ok_or("no stdin")?
        .write_all(password.as_bytes())
        .map_err(|error| error.to_string())?;
    let output = child.wait_with_output().map_err(|error| error.to_string())?;
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output.status.success() && hash.starts_with("$6$") {
        Ok(hash)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn backend() -> PathBuf {
    std::env::var_os("PATH")
        .into_iter()
        .flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
        .map(|dir| dir.join("flux-install"))
        .find(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from("/run/current-system/sw/bin/flux-install"))
}

/// Forward every progress line of `child` and say whether the backend reached
/// its own end: the `done` step or an `error` line.
fn relay(child: &mut Child, sender: &Sender<Message>) -> bool {
    let Some(stdout) = child.stdout.take() else {
        return false;
    };
    let mut finished = false;
    let events = BufReader::new(stdout).lines().map_while(Result::ok);
    for event in events.filter_map(|line| model::parse_progress(&line)) {
        finished |= matches!(event, Event::Error(_) | Event::Progress { step: Step::Done, .. });
        let _ = sender.send(Message::Event(event));
    }
    finished
}

fn run(choices: &Choices, password: String, sender: &Sender<Message>) -> Result<(), Failure> {
    let hash = hash(&password).map_err(Failure::Hash)?;
    drop(password);
    let mut child = Command::new("sudo")
        .arg("-n")
        .arg(backend())
        .args(model::install_args(choices, &hash))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| Failure::Start(error.to_string()))?;
    let finished = relay(&mut child, sender);
    let mut errors = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        let _ = stderr.read_to_string(&mut errors);
    }
    let code = child.wait().ok().and_then(|status| status.code()).unwrap_or(-1);
    if finished {
        Ok(())
    } else {
        Err(Failure::Exit(code, errors.trim().to_string()))
    }
}

/// Run the install off the main thread, reporting every progress line on `sender`.
pub fn install(choices: Choices, password: String, sender: Sender<Message>) {
    std::thread::spawn(move || {
        if let Err(failure) = run(&choices, password, &sender) {
            let _ = sender.send(Message::Failed(failure));
        }
    });
}


/// The install log, or an empty string before the backend writes it.
pub fn log() -> String {
    std::fs::read_to_string(LOG).unwrap_or_default()
}

/// Restart the machine.
pub fn reboot() {
    let _ = Command::new("systemctl").arg("reboot").spawn();
}
