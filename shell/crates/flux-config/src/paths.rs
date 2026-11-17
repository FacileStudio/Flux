//! Every file `flux-config` reads or writes, under one root so tests and dry
//! runs never touch the real `/etc`, `/run` or `/nix`.

use std::path::{Path, PathBuf};

/// The environment variable that moves the root, for tests and dry runs.
pub const ROOT_VAR: &str = "FLUX_CONFIG_ROOT";

/// The machine flake reference `nixos-rebuild` builds.
pub const FLAKE_ATTR: &str = "flux";

/// Where the files live, relative to a root that is `/` on a real system.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paths {
    pub root: PathBuf,
}

impl Paths {
    /// The root from [`ROOT_VAR`], or `/`.
    pub fn from_env() -> Paths {
        let root = std::env::var_os(ROOT_VAR).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"));
        Paths { root }
    }

    /// Whether the root is the real system.
    pub fn is_system(&self) -> bool {
        self.root == Path::new("/")
    }

    fn at(&self, path: &str) -> PathBuf {
        self.root.join(path.trim_start_matches('/'))
    }

    /// `/etc/flux`, the machine flake.
    pub fn flake_dir(&self) -> PathBuf {
        self.at("/etc/flux")
    }

    /// `/etc/flux/settings.json`.
    pub fn settings(&self) -> PathBuf {
        self.at("/etc/flux/settings.json")
    }

    /// `/etc/flux/settings.json.bak`, the copy kept before each write.
    pub fn backup(&self) -> PathBuf {
        self.at("/etc/flux/settings.json.bak")
    }

    /// `/etc/flux/flake.nix`.
    pub fn flake_nix(&self) -> PathBuf {
        self.at("/etc/flux/flake.nix")
    }

    /// `/etc/flux/flake.lock`.
    pub fn lock_file(&self) -> PathBuf {
        self.at("/etc/flux/flake.lock")
    }

    /// `/etc/flux/flake.lock.bak`, the lock as it was before the running update.
    pub fn lock_backup(&self) -> PathBuf {
        self.at("/etc/flux/flake.lock.bak")
    }

    /// `/var/lib/flux-config/locks`, one `flake.lock` per system generation still on disk.
    pub fn locks(&self) -> PathBuf {
        self.at("/var/lib/flux-config/locks")
    }

    /// `/run/flux-config`, the operation state that lives until the next boot.
    pub fn run_dir(&self) -> PathBuf {
        self.at("/run/flux-config")
    }

    /// `/run/flux-config/lock`, held while an operation runs.
    pub fn run_lock(&self) -> PathBuf {
        self.at("/run/flux-config/lock")
    }

    /// `/run/flux-config/state.json`, the last progress line of the running or last operation.
    pub fn state(&self) -> PathBuf {
        self.at("/run/flux-config/state.json")
    }

    /// `/run/flux-config/log`, the full output of the running or last operation.
    pub fn log(&self) -> PathBuf {
        self.at("/run/flux-config/log")
    }

    /// `/nix/var/nix/profiles`, where the system generations are linked.
    pub fn profiles(&self) -> PathBuf {
        self.at("/nix/var/nix/profiles")
    }

    /// `/nix/var/nix/profiles/system`.
    pub fn system_profile(&self) -> PathBuf {
        self.at("/nix/var/nix/profiles/system")
    }

    /// `/run/booted-system`.
    pub fn booted(&self) -> PathBuf {
        self.at("/run/booted-system")
    }

    /// `/run/current-system`.
    pub fn current(&self) -> PathBuf {
        self.at("/run/current-system")
    }

    /// `/etc/os-release`.
    pub fn os_release(&self) -> PathBuf {
        self.at("/etc/os-release")
    }

    /// `/etc/passwd`.
    pub fn passwd(&self) -> PathBuf {
        self.at("/etc/passwd")
    }

    /// The zoneinfo directory: `$TZDIR`, then `/etc/zoneinfo`, then `/usr/share/zoneinfo`, under the root.
    pub fn zoneinfo(&self) -> PathBuf {
        std::env::var_os("TZDIR")
            .map(PathBuf::from)
            .into_iter()
            .chain(["/etc/zoneinfo", "/usr/share/zoneinfo"].map(|dir| self.at(dir)))
            .find(|dir| dir.is_dir())
            .unwrap_or_else(|| self.at("/etc/zoneinfo"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_prefixes_every_path() {
        let paths = Paths { root: PathBuf::from("/tmp/t") };
        assert_eq!(paths.settings(), PathBuf::from("/tmp/t/etc/flux/settings.json"));
        assert_eq!(paths.state(), PathBuf::from("/tmp/t/run/flux-config/state.json"));
        assert!(!paths.is_system());
        assert!(Paths { root: PathBuf::from("/") }.is_system());
    }
}
