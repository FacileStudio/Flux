//! `/etc/flux/settings.json`: the machine state NixOS owns declaratively and
//! people change from Settings. `modules/settings.nix` reads the same file;
//! `docs/settings-json.md` is the schema both sides follow.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::validate;

/// The schema version this build reads and writes.
pub const VERSION: u32 = 1;

/// One person's account. Passwords never live here: they stay in `/etc/shadow`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct User {
    pub name: String,
    pub full_name: String,
    #[serde(default)]
    pub admin: bool,
    #[serde(default = "default_groups")]
    pub groups: Vec<String>,
}

/// How the machine updates itself.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Updates {
    #[serde(default)]
    pub automatic: bool,
}

/// The optional software groups Flux installs. `None` means the file does not
/// say: the key stays out of the file and NixOS keeps what `configuration.nix` says.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Software {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facile: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apps: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gaming: Option<bool>,
}

impl Software {
    /// Every choice filled in, with the Flux default where the file is silent.
    pub fn resolved(self) -> Software {
        Software {
            facile: Some(self.facile.unwrap_or(true)),
            apps: Some(self.apps.unwrap_or(true)),
            gaming: Some(self.gaming.unwrap_or(false)),
        }
    }

    fn is_unset(&self) -> bool {
        *self == Software::default()
    }
}

/// Drivers the machine needs beyond the free defaults. `None` works as in [`Software`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Hardware {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nvidia: Option<bool>,
}

impl Hardware {
    /// The choice filled in, with the Flux default where the file is silent.
    pub fn resolved(self) -> Hardware {
        Hardware { nvidia: Some(self.nvidia.unwrap_or(false)) }
    }

    fn is_unset(&self) -> bool {
        *self == Hardware::default()
    }
}

/// The whole file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Settings {
    pub version: u32,
    pub hostname: String,
    #[serde(default)]
    pub timezone: Option<String>,
    pub locale: String,
    #[serde(default)]
    pub formats: Option<String>,
    pub keymap: String,
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(default)]
    pub updates: Updates,
    #[serde(default, skip_serializing_if = "Software::is_unset")]
    pub software: Software,
    #[serde(default, skip_serializing_if = "Hardware::is_unset")]
    pub hardware: Hardware,
}

/// The groups an account listed without `groups` has, as `modules/settings.nix` reads it.
pub fn default_groups() -> Vec<String> {
    ["networkmanager", "video", "audio"].map(String::from).to_vec()
}

/// The groups an account made in Settings gets besides `wheel`: the installer's list.
pub fn new_user_groups() -> Vec<String> {
    ["networkmanager", "video", "audio", "lp"].map(String::from).to_vec()
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            version: VERSION,
            hostname: "flux".to_string(),
            timezone: Some("UTC".to_string()),
            locale: "en_US.UTF-8".to_string(),
            formats: None,
            keymap: "us".to_string(),
            users: Vec::new(),
            updates: Updates::default(),
            software: Software::default(),
            hardware: Hardware::default(),
        }
    }
}

impl Settings {
    /// Parse and check a document. `zoneinfo`, when given, must hold the time zone.
    pub fn parse(json: &str, zoneinfo: Option<&Path>) -> Result<Settings, String> {
        let settings: Settings = serde_json::from_str(json).map_err(|error| format!("settings.json: {error}"))?;
        settings.check(zoneinfo).map_err(|errors| errors.join("; "))?;
        Ok(settings)
    }

    /// Every rule `modules/settings.nix` enforces, so a file the helper writes always evaluates.
    pub fn check(&self, zoneinfo: Option<&Path>) -> Result<(), Vec<String>> {
        let mut results = self.field_checks(zoneinfo);
        results.extend(self.user_checks());
        let errors: Vec<String> = results.into_iter().filter_map(Result::err).collect();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn field_checks(&self, zoneinfo: Option<&Path>) -> Vec<Result<(), String>> {
        let version = if self.version == VERSION {
            Ok(())
        } else {
            Err(format!("settings.json version {} is not {VERSION}", self.version))
        };
        let zone = match (&self.timezone, zoneinfo) {
            (None, _) => Ok(()),
            (Some(zone), Some(dir)) => validate::timezone(zone, dir),
            (Some(zone), None) if validate::timezone_shape(zone) => Ok(()),
            (Some(zone), None) => Err(format!("unknown time zone {zone:?}")),
        };
        let formats = self.formats.as_deref().map_or(Ok(()), validate::locale);
        vec![version, validate::hostname(&self.hostname), zone, validate::locale(&self.locale), formats, validate::keymap(&self.keymap)]
    }

    fn user_checks(&self) -> Vec<Result<(), String>> {
        let mut results = Vec::new();
        for (index, user) in self.users.iter().enumerate() {
            results.push(validate::username(&user.name));
            results.push(validate::full_name(&user.full_name));
            results.extend(user.groups.iter().map(|group| validate::group(group)));
            if self.users[..index].iter().any(|other| other.name == user.name) {
                results.push(Err(format!("the user {:?} is listed twice", user.name)));
            }
        }
        if !self.users.is_empty() && self.admins() == 0 {
            results.push(Err("at least one user must be an administrator".to_string()));
        }
        results
    }

    /// The account called `name`.
    pub fn user(&self, name: &str) -> Option<&User> {
        self.users.iter().find(|user| user.name == name)
    }

    fn user_mut(&mut self, name: &str) -> Result<&mut User, String> {
        self.users.iter_mut().find(|user| user.name == name).ok_or_else(|| format!("no user called {name:?}"))
    }

    fn admins(&self) -> usize {
        self.users.iter().filter(|user| user.admin).count()
    }

    /// Add an account, refusing a name already taken here or by `existing` system accounts.
    pub fn add_user(&mut self, name: &str, full_name: &str, admin: bool, existing: &[String]) -> Result<(), String> {
        validate::username(name)?;
        validate::full_name(full_name)?;
        if self.user(name).is_some() || existing.iter().any(|taken| taken == name) {
            return Err(format!("the username {name:?} is already taken"));
        }
        self.users.push(User {
            name: name.to_string(),
            full_name: full_name.to_string(),
            admin,
            groups: new_user_groups(),
        });
        Ok(())
    }

    /// Remove an account, never the last administrator and never `caller`.
    pub fn remove_user(&mut self, name: &str, caller: Option<&str>) -> Result<(), String> {
        if caller == Some(name) {
            return Err("you cannot remove the account you are signed in with".to_string());
        }
        let user = self.user(name).ok_or_else(|| format!("no user called {name:?}"))?;
        if user.admin && self.admins() == 1 {
            return Err("this is the last administrator; make someone else an administrator first".to_string());
        }
        self.users.retain(|user| user.name != name);
        Ok(())
    }

    /// Change the name people see; the login name stays, so the home folder stays.
    pub fn rename_user(&mut self, name: &str, full_name: &str) -> Result<(), String> {
        validate::full_name(full_name)?;
        self.user_mut(name)?.full_name = full_name.to_string();
        Ok(())
    }

    /// Grant or take back administrator rights, never from the last administrator.
    pub fn set_admin(&mut self, name: &str, admin: bool) -> Result<(), String> {
        let last = self.admins() == 1;
        let user = self.user_mut(name)?;
        if user.admin && !admin && last {
            return Err("this is the last administrator; make someone else an administrator first".to_string());
        }
        user.admin = admin;
        Ok(())
    }

    /// The `software` and `hardware` keys the file does not set, as `flux-config set` names them.
    pub fn unset_choices(&self) -> Vec<&'static str> {
        [
            ("software.facile", self.software.facile),
            ("software.apps", self.software.apps),
            ("software.gaming", self.software.gaming),
            ("hardware.nvidia", self.hardware.nvidia),
        ]
        .into_iter()
        .filter_map(|(key, value)| value.is_none().then_some(key))
        .collect()
    }

    /// The settings with every `software` and `hardware` choice filled in, for showing. Never save this.
    pub fn resolved(&self) -> Settings {
        Settings { software: self.software.resolved(), hardware: self.hardware.resolved(), ..self.clone() }
    }

    /// The document as written to disk: pretty, with a trailing newline.
    pub fn to_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(self).unwrap_or_default();
        json.push('\n');
        json
    }
}

/// Read `path`: `Ok(None)` when it does not exist yet.
pub fn load(path: &Path, zoneinfo: Option<&Path>) -> Result<Option<Settings>, String> {
    match fs::read_to_string(path) {
        Ok(json) => Settings::parse(&json, zoneinfo).map(Some),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("cannot read {}: {error}", path.display())),
    }
}

/// Replace `path` with `bytes` without a moment where it is empty or half
/// written: a temporary file beside it, flushed, then renamed over it.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let dir = path.parent().ok_or_else(|| format!("{} has no directory", path.display()))?;
    let name = path.file_name().ok_or_else(|| format!("{} has no file name", path.display()))?;
    fs::create_dir_all(dir).map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
    let temp = dir.join(format!(".{}.tmp", name.to_string_lossy()));
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o644)
        .open(&temp)
        .map_err(|error| format!("cannot write {}: {error}", temp.display()))?;
    file.write_all(bytes).and_then(|()| file.sync_all()).map_err(|error| format!("cannot write {}: {error}", temp.display()))?;
    fs::rename(&temp, path).map_err(|error| format!("cannot replace {}: {error}", path.display()))?;
    if let Ok(dir) = File::open(dir) {
        let _ = dir.sync_all();
    }
    Ok(())
}

/// Put the content of `from` at `path`, atomically.
pub fn copy_atomic(from: &Path, path: &Path) -> Result<(), String> {
    let bytes = fs::read(from).map_err(|error| format!("cannot read {}: {error}", from.display()))?;
    write_atomic(path, &bytes)
}

/// Write `settings` to `path` atomically, after copying the old file to `backup`.
pub fn save(path: &Path, backup: &Path, settings: &Settings) -> Result<(), String> {
    settings.check(None).map_err(|errors| errors.join("; "))?;
    if path.exists() {
        fs::copy(path, backup).map_err(|error| format!("cannot back up {}: {error}", path.display()))?;
    }
    write_atomic(path, settings.to_json().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Settings {
        let mut settings = Settings::default();
        let _ = settings.add_user("ada", "Ada Lovelace", true, &[]);
        settings
    }

    #[test]
    fn round_trip() {
        let mut settings = sample();
        settings.timezone = None;
        settings.formats = Some("fr_FR.UTF-8".to_string());
        let parsed = Settings::parse(&settings.to_json(), None);
        assert_eq!(parsed, Ok(settings));
    }

    #[test]
    fn camel_case_and_defaults() {
        let json = r#"{"version":1,"hostname":"pc","locale":"fr_FR.UTF-8","keymap":"fr",
            "users":[{"name":"ada","fullName":"Ada","admin":true}]}"#;
        let settings = Settings::parse(json, None).unwrap_or_default();
        assert_eq!(settings.hostname, "pc");
        assert_eq!(settings.timezone, None);
        assert_eq!(settings.users[0].groups, default_groups());
        assert!(settings.to_json().contains("\"fullName\": \"Ada\""));
    }

    #[test]
    fn absent_choices_stay_absent() {
        let json = r#"{"version":1,"hostname":"pc","locale":"en_US.UTF-8","keymap":"us"}"#;
        let settings = Settings::parse(json, None).unwrap_or_default();
        assert_eq!(settings.software, Software::default());
        assert_eq!(settings.hardware.nvidia, None);
        let written = settings.to_json();
        assert!(!written.contains("software") && !written.contains("hardware"), "{written}");
        assert_eq!(settings.unset_choices(), ["software.facile", "software.apps", "software.gaming", "hardware.nvidia"]);
        let resolved = settings.resolved();
        assert_eq!(resolved.software, Software { facile: Some(true), apps: Some(true), gaming: Some(false) });
        assert_eq!(resolved.hardware.nvidia, Some(false));
    }

    #[test]
    fn set_choices_are_kept() {
        let partial = r#"{"version":1,"hostname":"pc","locale":"en_US.UTF-8","keymap":"us",
            "software":{"apps":false,"gaming":true},"hardware":{"nvidia":true}}"#;
        let settings = Settings::parse(partial, None).unwrap_or_default();
        assert_eq!(settings.software, Software { facile: None, apps: Some(false), gaming: Some(true) });
        assert_eq!(settings.hardware.nvidia, Some(true));
        assert_eq!(settings.unset_choices(), ["software.facile"]);
        let written: serde_json::Value = serde_json::from_str(&settings.to_json()).unwrap_or_default();
        assert_eq!(written["software"], serde_json::json!({ "apps": false, "gaming": true }));
        assert_eq!(written["hardware"], serde_json::json!({ "nvidia": true }));
        assert_eq!(settings.resolved().software.facile, Some(true));
        assert!(Settings::parse(r#"{"version":1,"hostname":"pc","locale":"en_US.UTF-8","keymap":"us","software":{"games":true}}"#, None).is_err());
        assert!(Settings::parse(r#"{"version":1,"hostname":"pc","locale":"en_US.UTF-8","keymap":"us","hardware":{"nvidia":"yes"}}"#, None).is_err());
    }

    #[test]
    fn strict_parse() {
        assert!(Settings::parse(r#"{"version":1,"hostname":"pc","locale":"en_US.UTF-8","keymap":"us","extra":1}"#, None).is_err());
        assert!(Settings::parse(r#"{"version":2,"hostname":"pc","locale":"en_US.UTF-8","keymap":"us"}"#, None).is_err());
        assert!(Settings::parse(r#"{"version":1,"hostname":"pc-","locale":"en_US.UTF-8","keymap":"us"}"#, None).is_err());
        let no_admin = r#"{"version":1,"hostname":"pc","locale":"en_US.UTF-8","keymap":"us",
            "users":[{"name":"ada","fullName":"Ada"}]}"#;
        assert!(Settings::parse(no_admin, None).is_err());
        let twice = r#"{"version":1,"hostname":"pc","locale":"en_US.UTF-8","keymap":"us",
            "users":[{"name":"ada","fullName":"Ada","admin":true},{"name":"ada","fullName":"B"}]}"#;
        assert!(Settings::parse(twice, None).is_err());
    }

    #[test]
    fn user_rules() {
        let mut settings = sample();
        assert!(settings.add_user("ada", "Again", false, &[]).is_err());
        assert!(settings.add_user("bob", "Bob", false, &["bob".to_string()]).is_err());
        assert!(settings.add_user("root", "Root", false, &[]).is_err());
        assert!(settings.remove_user("ada", None).is_err());
        assert!(settings.set_admin("ada", false).is_err());
        assert!(settings.add_user("bob", "Bob", false, &[]).is_ok());
        assert!(settings.remove_user("bob", Some("bob")).is_err());
        assert!(settings.set_admin("bob", true).is_ok());
        assert!(settings.set_admin("ada", false).is_ok());
        assert!(settings.remove_user("ada", Some("bob")).is_ok());
        assert!(settings.rename_user("bob", "Robert").is_ok());
        assert_eq!(settings.user("bob").map(|user| user.full_name.as_str()), Some("Robert"));
        assert!(settings.rename_user("bob", "a:b").is_err());
        assert!(settings.rename_user("nobody-here", "X").is_err());
    }

    #[test]
    fn atomic_save_keeps_a_backup() {
        let dir = std::env::temp_dir().join(format!("flux-config-save-{}", std::process::id()));
        let path = dir.join("settings.json");
        let backup = dir.join("settings.json.bak");
        let first = sample();
        assert_eq!(save(&path, &backup, &first), Ok(()));
        let mut second = first.clone();
        second.hostname = "lab".to_string();
        assert_eq!(save(&path, &backup, &second), Ok(()));
        assert_eq!(load(&path, None), Ok(Some(second)));
        assert_eq!(load(&backup, None), Ok(Some(first.clone())));
        assert!(!dir.join(".settings.json.tmp").exists());
        let bad = Settings { hostname: "-".to_string(), ..Settings::default() };
        assert!(save(&path, &backup, &bad).is_err());
        assert_eq!(load(&dir.join("missing.json"), None), Ok(None));
        assert_eq!(copy_atomic(&backup, &path), Ok(()));
        assert_eq!(load(&path, None), Ok(Some(first)));
        assert!(copy_atomic(&dir.join("missing.json"), &path).is_err());
        assert!(path.is_file() && !dir.join(".settings.json.tmp").exists());
        let _ = fs::remove_dir_all(&dir);
    }
}
