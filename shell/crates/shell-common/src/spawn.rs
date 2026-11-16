//! Starting another shell program, or any command, without waiting on it.
//!
//! The shell binaries are installed side by side, so a widget reaches a sibling
//! (`sidebar`, `exit`) through its own directory first. That keeps a click
//! working when the unit's `PATH` lacks the system profile. Under the Nix
//! wrapper the running file is `.<name>-wrapped`, and the sibling found next to
//! it is the sibling's own wrapper.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The path of the shell program `name`: next to this executable when it is
/// there, the bare name for a `PATH` lookup otherwise.
pub fn sibling(name: &str) -> OsString {
    let dir = std::env::current_exe().ok().and_then(|exe| exe.parent().map(Path::to_path_buf));
    sibling_in(dir.as_deref(), name)
        .map(PathBuf::into_os_string)
        .unwrap_or_else(|| name.into())
}

/// `dir/name`, when that file exists.
pub fn sibling_in(dir: Option<&Path>, name: &str) -> Option<PathBuf> {
    let path = dir?.join(name);
    path.is_file().then_some(path)
}

/// Start `program` with `args` in the background, its output discarded, and
/// reap it on a thread so it never lingers as a zombie. Errors are logged.
pub fn detached(program: impl Into<OsString>, args: &[&str]) {
    let program = program.into();
    let child = Command::new(&program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    match child {
        Ok(mut child) => drop(std::thread::spawn(move || child.wait())),
        Err(error) => eprintln!("flux: cannot start {}: {error}", program.to_string_lossy()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sibling_is_found_next_to_the_executable() {
        let dir = std::env::temp_dir().join(format!("flux-spawn-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch dir");
        std::fs::write(dir.join("sidebar"), "").expect("sibling file");
        assert_eq!(sibling_in(Some(&dir), "sidebar"), Some(dir.join("sidebar")));
        assert_eq!(sibling_in(Some(&dir), "exit"), None);
        assert_eq!(sibling_in(None, "sidebar"), None);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
