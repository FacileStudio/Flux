//! The system generations under `/nix/var/nix/profiles`: one per apply, the
//! versions a person can go back to.

use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

/// One generation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Generation {
    pub number: u32,
    pub date: u64,
    pub version: String,
    pub kernel: String,
    pub current: bool,
}

/// The generation number in a profile link name such as `system-42-link`.
pub fn link_number(name: &str) -> Option<u32> {
    name.strip_prefix("system-")?.strip_suffix("-link")?.parse().ok()
}

/// The kernel version in a kernel link target such as `/nix/store/<hash>-linux-6.12.1/bzImage`.
pub fn kernel_version(target: &str) -> String {
    target
        .split('/')
        .find_map(|part| {
            let (_, name) = part.split_once('-')?;
            let version = name.strip_prefix("linux-")?;
            Some(version.split('-').next().unwrap_or(version).to_string())
        })
        .unwrap_or_default()
}

/// The value of `key` in an os-release document, unquoted.
pub fn os_release(document: &str, key: &str) -> Option<String> {
    document.lines().find_map(|line| {
        let value = line.strip_prefix(key)?.strip_prefix('=')?;
        Some(value.trim().trim_matches('"').to_string())
    })
}

/// The name a generation is known by: `PRETTY_NAME` from its os-release, else its `nixos-version`.
fn version_of(link: &Path) -> String {
    fs::read_to_string(link.join("etc/os-release"))
        .ok()
        .and_then(|document| os_release(&document, "PRETTY_NAME"))
        .or_else(|| fs::read_to_string(link.join("nixos-version")).ok().map(|text| text.trim().to_string()))
        .unwrap_or_default()
}

/// Every generation in `profiles`, newest first. `system` is the profile link
/// whose target marks the current one.
pub fn list(profiles: &Path, system: &Path) -> Vec<Generation> {
    let current = fs::read_link(system)
        .ok()
        .and_then(|target| target.file_name().and_then(|name| name.to_str()).and_then(link_number));
    let Ok(entries) = fs::read_dir(profiles) else {
        return Vec::new();
    };
    let mut generations: Vec<Generation> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let number = link_number(entry.file_name().to_str()?)?;
            let link = entry.path();
            let date = fs::symlink_metadata(&link)
                .and_then(|meta| meta.modified())
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|age| age.as_secs())
                .unwrap_or(0);
            let kernel = fs::read_link(link.join("kernel"))
                .map(|target| kernel_version(&target.to_string_lossy()))
                .unwrap_or_default();
            Some(Generation { number, date, version: version_of(&link), kernel, current: current == Some(number) })
        })
        .collect();
    generations.sort_by_key(|generation| std::cmp::Reverse(generation.number));
    generations
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn names() {
        assert_eq!(link_number("system-42-link"), Some(42));
        assert_eq!(link_number("system"), None);
        assert_eq!(link_number("system-x-link"), None);
        assert_eq!(link_number("per-user"), None);
        assert_eq!(kernel_version("/nix/store/abc123-linux-6.12.1/bzImage"), "6.12.1");
        assert_eq!(kernel_version("/nix/store/abc123-linux-6.18-rc3-modules"), "6.18");
        assert_eq!(kernel_version("/nix/store/abc-initrd"), "");
        let release = "NAME=Flux\nPRETTY_NAME=\"Flux 0.1.0\"\nVERSION_ID=\"25.11\"\n";
        assert_eq!(os_release(release, "PRETTY_NAME").as_deref(), Some("Flux 0.1.0"));
        assert_eq!(os_release(release, "NAME").as_deref(), Some("Flux"));
        assert_eq!(os_release(release, "ID"), None);
    }

    #[test]
    fn list_from_fixture() {
        let root = std::env::temp_dir().join(format!("flux-config-gens-{}", std::process::id()));
        let profiles = root.join("profiles");
        let _ = fs::create_dir_all(&profiles);
        for (number, version, kernel) in [(1, "Flux 0.1.0", "6.12.1"), (2, "Flux 0.2.0", "6.13.0")] {
            let store = root.join(format!("store/gen{number}"));
            let _ = fs::create_dir_all(store.join("etc"));
            let _ = fs::write(store.join("etc/os-release"), format!("PRETTY_NAME=\"{version}\"\n"));
            let _ = symlink(format!("/nix/store/h-linux-{kernel}/bzImage"), store.join("kernel"));
            let _ = symlink(&store, profiles.join(format!("system-{number}-link")));
        }
        let _ = symlink("system-1-link", profiles.join("system"));
        let generations = list(&profiles, &profiles.join("system"));
        let summary: Vec<(u32, &str, &str, bool)> = generations
            .iter()
            .map(|g| (g.number, g.version.as_str(), g.kernel.as_str(), g.current))
            .collect();
        assert_eq!(summary, [(2, "Flux 0.2.0", "6.13.0", false), (1, "Flux 0.1.0", "6.12.1", true)]);
        assert!(generations.iter().all(|g| g.date > 0));
        assert!(list(&root.join("missing"), &profiles.join("system")).is_empty());
        let _ = fs::remove_dir_all(&root);
    }
}
