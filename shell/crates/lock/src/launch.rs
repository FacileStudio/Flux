//! The command line, the single-instance guard, and `--daemon`.
//!
//! `--daemon` and `--suspend-then-lock` re-run this binary with `--notify` and
//! return as soon as the child reports the session locked, so a caller such as
//! swayidle's `before-sleep` hook (which waits for its command) lets the
//! machine sleep only once the lock surface is up. The child keeps running
//! until the user unlocks.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// How long a launcher waits for the lock before it gives up. logind's
/// default sleep delay is five seconds.
const LOCK_WAIT: Duration = Duration::from_secs(4);

/// What the command line asks for.
#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Version,
    Help,
    Foreground,
    Notify,
    Daemon,
    LockThenSuspend,
}

pub const USAGE: &str = "usage: flux-lock [--daemon | --suspend-then-lock | --version]\n\
\n\
  (none)               lock now and stay in the foreground until unlocked\n\
  --daemon             lock, and return once the session is locked\n\
  --suspend-then-lock  lock, then suspend once the session is locked\n";

/// The mode for the arguments after the program name, or the unknown one.
pub fn parse(args: &[String]) -> Result<Mode, String> {
    match args {
        [] => Ok(Mode::Foreground),
        [one] => match one.as_str() {
            "--version" => Ok(Mode::Version),
            "--help" | "-h" => Ok(Mode::Help),
            "--notify" => Ok(Mode::Notify),
            "--daemon" => Ok(Mode::Daemon),
            "--suspend-then-lock" => Ok(Mode::LockThenSuspend),
            other => Err(other.to_string()),
        },
        [_, second, ..] => Err(second.to_string()),
    }
}

fn guard_path() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("flux-lock.lock")
}

/// Take the single-instance lock. `Ok(None)` means another flux-lock holds it.
///
/// The guard is an `flock` on a file under `$XDG_RUNTIME_DIR`, released by the
/// kernel when the process ends however it ends, so it never goes stale.
pub fn claim() -> std::io::Result<Option<File>> {
    let file = File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .open(guard_path())?;
    let taken = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if taken == 0 {
        return Ok(Some(file));
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::EWOULDBLOCK) {
        Ok(None)
    } else {
        Err(error)
    }
}

/// Run a locking child and return once it is locked. The exit code is 0 when
/// the session is locked, non-zero otherwise; `suspend` asks logind to sleep
/// only in the first case.
pub fn launch(suspend: bool) -> i32 {
    let Ok(exe) = std::env::current_exe() else {
        eprintln!("flux-lock: cannot find its own binary");
        return 1;
    };
    let spawned = Command::new(exe)
        .arg("--notify")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(error) => {
            eprintln!("flux-lock: cannot start the locker: {error}");
            return 1;
        }
    };
    let Some(stdout) = child.stdout.take() else {
        return 1;
    };
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        let read = BufReader::new(stdout).read_line(&mut line).is_ok();
        let _ = sender.send(read && line.trim() == "locked");
    });
    match receiver.recv_timeout(LOCK_WAIT) {
        Ok(true) => {
            if suspend {
                return suspend_now();
            }
            0
        }
        Ok(false) => child.wait().ok().and_then(|status| status.code()).filter(|code| *code != 0).unwrap_or(1),
        Err(_) => {
            eprintln!("flux-lock: the session did not lock in time");
            1
        }
    }
}

fn suspend_now() -> i32 {
    match Command::new("systemctl").arg("suspend").status() {
        Ok(status) if status.success() => 0,
        _ => {
            eprintln!("flux-lock: locked, but systemctl suspend failed");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn the_command_line_parses() {
        assert_eq!(parse(&args(&[])), Ok(Mode::Foreground));
        assert_eq!(parse(&args(&["--daemon"])), Ok(Mode::Daemon));
        assert_eq!(parse(&args(&["--suspend-then-lock"])), Ok(Mode::LockThenSuspend));
        assert_eq!(parse(&args(&["--version"])), Ok(Mode::Version));
        assert_eq!(parse(&args(&["--unlock"])), Err("--unlock".into()));
        assert_eq!(parse(&args(&["--daemon", "--x"])), Err("--x".into()));
    }

    #[test]
    fn a_second_claim_is_refused_while_the_first_holds() {
        let dir = std::env::temp_dir().join(format!("flux-lock-claim-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        std::env::set_var("XDG_RUNTIME_DIR", &dir);
        let first = claim().expect("claim").expect("free");
        let path = guard_path();
        let other = File::options().write(true).open(&path).expect("open");
        let taken = unsafe { libc::flock(other.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        assert_ne!(taken, 0);
        drop(first);
        let taken = unsafe { libc::flock(other.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        assert_eq!(taken, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
