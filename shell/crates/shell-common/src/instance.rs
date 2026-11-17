//! One copy of a widget, at most.
//!
//! Widgets meant to live for a whole session (the statusbar, the sidebar
//! daemon) claim a pid file under `$XDG_RUNTIME_DIR` before they do any work.
//! A second copy, started from a compositor config or a key binding, finds the
//! file and steps aside instead of adding a second bar to the screen.
//!
//! A pid alone is not enough to trust the file: pids are recycled, and a stale
//! entry pointing at whatever inherited the number would block the widget, or
//! worse, make it signal an unrelated process. [`running`] therefore also
//! checks the process name, and reads a file whose process is gone, or is no
//! longer a copy of the widget, as stale and takes it over.
//!
//! The Nix wrapper renames the real binary to `.<name>-wrapped` and runs it
//! with `argv[0]` set to the wrapper's path, so `/proc/<pid>/comm` reads
//! `.<name>-wrappe` (the kernel keeps 15 bytes). [`is_copy`] accepts the plain
//! name, the wrapped name and its truncation, from `comm` or from `argv[0]`.

use std::path::PathBuf;

/// The longest name the kernel keeps in `/proc/<pid>/comm`.
const COMM_LEN: usize = 15;

/// The pid file a widget claims, e.g. `$XDG_RUNTIME_DIR/flux-statusbar.pid`.
pub fn pidfile(name: &str) -> PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    dir.join(format!("flux-{name}.pid"))
}

/// The pid of the running copy of `name`, if the pid file names a live one.
///
/// This process never counts as a copy of itself, so [`claim`] can call it.
pub fn running(name: &str) -> Option<i32> {
    let pid = std::fs::read_to_string(pidfile(name))
        .ok()?
        .trim()
        .parse::<i32>()
        .ok()?;
    let ours = std::process::id() as i32;
    (pid > 0 && pid != ours && is_widget(name, pid)).then_some(pid)
}

/// Take ownership of `name`, writing this process's pid.
///
/// Returns the pid of the copy that is already running, and leaves the file
/// untouched in that case; otherwise returns `None` and this process owns it
/// until [`release`].
pub fn claim(name: &str) -> Option<i32> {
    if let Some(pid) = running(name) {
        return Some(pid);
    }
    let path = pidfile(name);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, std::process::id().to_string());
    None
}

/// Remove the pid file, but only while it still describes this process.
pub fn release(name: &str) {
    let ours = std::process::id().to_string();
    if std::fs::read_to_string(pidfile(name)).ok().as_deref() == Some(ours.as_str()) {
        let _ = std::fs::remove_file(pidfile(name));
    }
}

/// Whether `pid` is currently a copy of the widget `name`.
///
/// On a system without `/proc` this is always false, which errs towards letting
/// the widget start rather than locking the user out of it.
fn is_widget(name: &str, pid: i32) -> bool {
    let comm = std::fs::read_to_string(format!("/proc/{pid}/comm")).unwrap_or_default();
    let cmdline = std::fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default();
    let argv0 = cmdline.split(|byte| *byte == 0).next().unwrap_or_default();
    is_copy(name, comm.trim_end(), &String::from_utf8_lossy(argv0))
}

/// Whether a process with this `comm` and `argv[0]` is a copy of `name`.
pub fn is_copy(name: &str, comm: &str, argv0: &str) -> bool {
    let wrapped = format!(".{name}-wrapped");
    let base = argv0.rsplit('/').next().unwrap_or_default();
    let named = |candidate: &str| candidate == name || candidate == wrapped;
    let truncated = !comm.is_empty()
        && [name, wrapped.as_str()]
            .iter()
            .any(|full| full.len() > COMM_LEN && full.get(..COMM_LEN) == Some(comm));
    named(base) || named(comm) || truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_widget_owns_its_pid_file() {
        let dir = std::env::temp_dir().join("flux-instance-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp runtime dir");
        std::env::set_var("XDG_RUNTIME_DIR", &dir);

        assert_eq!(claim("statusbar"), None);
        assert_eq!(
            std::fs::read_to_string(pidfile("statusbar")).expect("pid file"),
            std::process::id().to_string()
        );
        assert_eq!(running("statusbar"), None);

        release("statusbar");
        assert!(!pidfile("statusbar").exists());
        release("statusbar");

        std::fs::write(pidfile("statusbar"), "2147483646").expect("stale pid file");
        assert_eq!(running("statusbar"), None);
        assert_eq!(claim("statusbar"), None);

        std::fs::write(pidfile("statusbar"), "1").expect("foreign pid file");
        assert_eq!(running("statusbar"), None);

        release("statusbar");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn wrapped_binaries_count_as_copies() {
        assert!(is_copy("sidebar", "sidebar", "sidebar"));
        assert!(is_copy("sidebar", ".sidebar-wrappe", "/nix/store/x-flux-shell/bin/sidebar"));
        assert!(is_copy("sidebar", ".sidebar-wrappe", ""));
        assert!(is_copy("statusbar", "x", "/nix/store/x/bin/.statusbar-wrapped"));
        assert!(is_copy("statusbar", ".statusbar-wrap", ""));
        assert!(!is_copy("sidebar", "bash", "/bin/bash"));
        assert!(!is_copy("sidebar", "", ""));
        assert!(!is_copy("sidebar", ".sidebar-wrap", "/bin/sh"));
        assert!(!is_copy("sidebar", "sidebar-extra", "/bin/sidebar-extra"));
    }
}
