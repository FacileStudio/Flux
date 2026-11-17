//! The `flux` input of the machine flake: which Flux revision is installed and
//! where to look for a newer one.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::settings::copy_atomic;

/// The input name the machine flake gives Flux.
pub const INPUT: &str = "flux";

/// The locked `flux` input of a `flake.lock`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Locked {
    pub rev: String,
    pub last_modified: u64,
    pub kind: String,
    pub owner: String,
    pub repo: String,
    pub url: String,
    pub branch: Option<String>,
}

impl Locked {
    /// The short revision people see.
    pub fn short(&self) -> &str {
        self.rev.get(..7).unwrap_or(&self.rev)
    }

    /// The git URL `git ls-remote` reads, when the input has one.
    pub fn git_url(&self) -> Option<String> {
        match self.kind.as_str() {
            "github" => Some(format!("https://github.com/{}/{}", self.owner, self.repo)),
            "gitlab" => Some(format!("https://gitlab.com/{}/{}", self.owner, self.repo)),
            "git" => Some(self.url.trim_start_matches("git+").to_string()).filter(|url| !url.is_empty()),
            _ => None,
        }
    }

    /// The ref to compare against: the branch the input follows, else `HEAD`.
    pub fn remote_ref(&self) -> String {
        self.branch.as_ref().map(|branch| format!("refs/heads/{branch}")).unwrap_or_else(|| "HEAD".to_string())
    }

    /// The flake reference `nix flake metadata` resolves to the remote tip.
    pub fn flake_ref(&self) -> Option<String> {
        match self.kind.as_str() {
            "github" | "gitlab" => Some(match &self.branch {
                Some(branch) => format!("{}:{}/{}/{branch}", self.kind, self.owner, self.repo),
                None => format!("{}:{}/{}", self.kind, self.owner, self.repo),
            }),
            "git" => self.git_url().map(|url| match &self.branch {
                Some(branch) => format!("git+{url}?ref={branch}"),
                None => format!("git+{url}"),
            }),
            _ => None,
        }
    }
}

fn text(value: &Value, key: &str) -> String {
    value.get(key).and_then(Value::as_str).unwrap_or_default().to_string()
}

/// The `flux` input of a `flake.lock` document.
pub fn parse_lock(document: &str) -> Option<Locked> {
    let root: Value = serde_json::from_str(document).ok()?;
    let nodes = root.get("nodes")?;
    let root_name = root.get("root").and_then(Value::as_str).unwrap_or("root");
    let node_name = match nodes.get(root_name)?.get("inputs")?.get(INPUT)? {
        Value::String(name) => name.clone(),
        _ => return None,
    };
    let node = nodes.get(&node_name)?;
    let locked = node.get("locked")?;
    let original = node.get("original");
    let branch = original
        .and_then(|original| original.get("ref"))
        .and_then(Value::as_str)
        .map(String::from);
    let rev = text(locked, "rev");
    if rev.is_empty() {
        return None;
    }
    Some(Locked {
        rev,
        last_modified: locked.get("lastModified").and_then(Value::as_u64).unwrap_or(0),
        kind: text(locked, "type"),
        owner: text(locked, "owner"),
        repo: text(locked, "repo"),
        url: text(locked, "url"),
        branch,
    })
}

/// The revision `git ls-remote <url> <ref>` printed.
pub fn parse_ls_remote(output: &str) -> Option<String> {
    output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .find(|rev| rev.len() == 40 && rev.chars().all(|c| c.is_ascii_hexdigit()))
        .map(String::from)
}

/// The revision in `nix flake metadata --json` output.
pub fn parse_metadata(output: &str) -> Option<String> {
    let value: Value = serde_json::from_str(output).ok()?;
    value
        .get("revision")
        .or_else(|| value.get("locked").and_then(|locked| locked.get("rev")))
        .and_then(Value::as_str)
        .map(String::from)
}

fn kept(locks: &Path, generation: &str) -> PathBuf {
    locks.join(format!("{generation}.lock"))
}

/// Keep a copy of `lock_file` as the lock generation `generation` (its store name) was built from.
pub fn remember(locks: &Path, generation: &str, lock_file: &Path) -> Result<(), String> {
    copy_atomic(lock_file, &kept(locks, generation))
}

/// Whether a lock is kept for `generation`.
pub fn remembers(locks: &Path, generation: &str) -> bool {
    kept(locks, generation).is_file()
}

/// Put the lock kept for `generation` back as `lock_file`: `Ok(false)` when none was kept.
pub fn recall(locks: &Path, generation: &str, lock_file: &Path) -> Result<bool, String> {
    if !remembers(locks, generation) {
        return Ok(false);
    }
    copy_atomic(&kept(locks, generation), lock_file).map(|()| true)
}

/// Delete the kept locks of every generation not named in `alive`.
pub fn forget_except(locks: &Path, alive: &[String]) {
    let Ok(entries) = fs::read_dir(locks) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !alive.iter().any(|generation| name == format!("{generation}.lock")) {
            let _ = fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kept_locks() {
        let dir = std::env::temp_dir().join(format!("flux-config-locks-{}", std::process::id()));
        let locks = dir.join("locks");
        let lock_file = dir.join("flake.lock");
        let _ = fs::create_dir_all(&dir);
        let _ = fs::write(&lock_file, "old");
        assert_eq!(recall(&locks, "gen1", &lock_file), Ok(false));
        assert_eq!(remember(&locks, "gen1", &lock_file), Ok(()));
        let _ = fs::write(&lock_file, "new");
        assert_eq!(remember(&locks, "gen2", &lock_file), Ok(()));
        assert_eq!(recall(&locks, "gen1", &lock_file), Ok(true));
        assert_eq!(fs::read_to_string(&lock_file).unwrap_or_default(), "old");
        forget_except(&locks, &["gen2".to_string()]);
        assert!(!remembers(&locks, "gen1") && remembers(&locks, "gen2"));
        assert!(remember(&locks, "gen3", &dir.join("missing.lock")).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    const LOCK: &str = include_str!("../tests/fixtures/flake.lock");

    #[test]
    fn lock() {
        let locked = parse_lock(LOCK);
        let Some(locked) = locked else {
            panic!("fixture lock did not parse");
        };
        assert_eq!(locked.rev, "b00595e4a1c0d2b3c4d5e6f708192a3b4c5d6e7f");
        assert_eq!(locked.short(), "b00595e");
        assert_eq!(locked.kind, "github");
        assert_eq!(locked.branch.as_deref(), Some("stable"));
        assert_eq!(locked.git_url().as_deref(), Some("https://github.com/FacileStudio/Flux"));
        assert_eq!(locked.remote_ref(), "refs/heads/stable");
        assert_eq!(locked.flake_ref().as_deref(), Some("github:FacileStudio/Flux/stable"));
        assert_eq!(locked.last_modified, 1_763_300_000);
        assert_eq!(parse_lock("{}"), None);
        assert_eq!(parse_lock("not json"), None);
    }

    #[test]
    fn remote_outputs() {
        let rev = "0123456789abcdef0123456789abcdef01234567";
        assert_eq!(parse_ls_remote(&format!("{rev}\trefs/heads/stable\n")).as_deref(), Some(rev));
        assert_eq!(parse_ls_remote("fatal: unable to access"), None);
        assert_eq!(parse_metadata(&format!(r#"{{"revision":"{rev}"}}"#)).as_deref(), Some(rev));
        assert_eq!(parse_metadata(&format!(r#"{{"locked":{{"rev":"{rev}"}}}}"#)).as_deref(), Some(rev));
        assert_eq!(parse_metadata("{}"), None);
    }
}
