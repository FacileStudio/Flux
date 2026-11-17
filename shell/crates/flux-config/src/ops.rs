//! Every `flux-config` command.

use std::fs;
use std::io::Read;

use serde_json::{json, Value};

use flux_config::flake::{self, Locked};
use flux_config::generations::{self, os_release, Generation};
use flux_config::machine;
use flux_config::paths::FLAKE_ATTR;
use flux_config::progress::{needs_restart, State, Step, Tracker};
use flux_config::seed;
use flux_config::settings::{self, Settings};
use flux_config::validate;

use crate::exec::{self, Mode, Reporter};
use crate::Args;

/// Why a command stopped. `main` gives each its own exit code.
pub enum Failure {
    Usage(String),
    Failed(String),
    Denied(String),
    Busy(String),
}

impl From<exec::LockError> for Failure {
    fn from(error: exec::LockError) -> Self {
        match error {
            exec::LockError::Busy => Failure::Busy("another change is already running; wait for it to finish".to_string()),
            exec::LockError::Other(error) => Failure::Failed(error),
        }
    }
}

impl From<String> for Failure {
    fn from(error: String) -> Self {
        Failure::Failed(error)
    }
}

type Outcome = Result<(), Failure>;

fn usage<T>(text: &str) -> Result<T, Failure> {
    Err(Failure::Usage(text.to_string()))
}

/// Run the command `args` names.
pub fn run(mode: &Mode, args: &Args) -> Outcome {
    let words: Vec<&str> = args.words.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["get"] => get(mode),
        ["status"] => status(mode),
        ["check"] => check(mode),
        ["generations"] => print(&json!(generations::list(&mode.paths.profiles(), &mode.paths.system_profile()))),
        ["set", pairs @ ..] if !pairs.is_empty() && pairs.len() % 2 == 0 => set(mode, pairs, args.apply),
        ["user", rest @ ..] => user(mode, rest, args),
        ["apply"] => apply(mode),
        ["update"] => update(mode, args.scheduled),
        ["rollback"] => rollback(mode, None),
        ["rollback", number] => rollback(mode, Some(number)),
        ["gc"] => gc(mode, args.keep.as_deref()),
        _ => usage("unknown command"),
    }
}

fn print(value: &Value) -> Outcome {
    println!("{value}");
    Ok(())
}

fn current(mode: &Mode) -> Result<(Settings, bool), Failure> {
    let zoneinfo = mode.paths.zoneinfo();
    match settings::load(&mode.paths.settings(), Some(&zoneinfo))? {
        Some(settings) => Ok((settings, true)),
        None => Ok((seed::from_system(&mode.paths), false)),
    }
}

fn get(mode: &Mode) -> Outcome {
    let (settings, exists) = current(mode)?;
    print(&json!({ "exists": exists, "settings": settings.resolved(), "unset": settings.unset_choices() }))
}

fn link(path: &std::path::Path) -> Option<String> {
    fs::read_link(path).ok().map(|target| target.to_string_lossy().into_owned())
}

fn restart_needed(mode: &Mode) -> bool {
    let parts = ["kernel", "kernel-modules", "initrd"];
    let booted: Vec<Option<String>> = parts.iter().map(|part| link(&mode.paths.booted().join(part))).collect();
    let current: Vec<Option<String>> = parts.iter().map(|part| link(&mode.paths.current().join(part))).collect();
    needs_restart(&booted, &current)
}

fn locked(mode: &Mode) -> Option<Locked> {
    flake::parse_lock(&fs::read_to_string(mode.paths.lock_file()).ok()?)
}

fn status(mode: &Mode) -> Outcome {
    let state = fs::read_to_string(mode.paths.state()).ok().and_then(|text| State::parse(&text));
    let running = state.as_ref().is_some_and(|state| state.running(exec::alive(state.pid)));
    let version = fs::read_to_string(mode.paths.os_release())
        .ok()
        .and_then(|document| os_release(&document, "PRETTY_NAME"))
        .unwrap_or_default();
    let flux = locked(mode).map(|locked| {
        json!({ "rev": locked.rev, "short": locked.short(), "date": locked.last_modified, "branch": locked.branch })
    });
    print(&json!({
        "state": state,
        "running": running,
        "restart": restart_needed(mode),
        "version": version,
        "flux": flux,
    }))
}

fn remote_rev(mode: &Mode, locked: &Locked) -> Result<String, String> {
    if !mode.runs_commands() {
        eprintln!("dry-run: git ls-remote {} {}", locked.git_url().unwrap_or_default(), locked.remote_ref());
        return Ok(locked.rev.clone());
    }
    if let Some(url) = locked.git_url() {
        if let Ok((out, _, true)) = exec::capture("git", &["ls-remote".into(), url, locked.remote_ref()], None) {
            if let Some(rev) = flake::parse_ls_remote(&out) {
                return Ok(rev);
            }
        }
    }
    let reference = locked.flake_ref().ok_or("the flux input is not a git or GitHub flake")?;
    let args = ["flake", "metadata", "--json", "--refresh", reference.as_str()].map(String::from);
    let (out, errors, ok) = exec::capture("nix", &args, None)?;
    if !ok {
        return Err(format!("cannot reach the update server: {}", errors.trim()));
    }
    flake::parse_metadata(&out).ok_or_else(|| "the update server gave no revision".to_string())
}

fn check(mode: &Mode) -> Outcome {
    let locked = locked(mode).ok_or_else(|| format!("{} has no flux input", mode.paths.lock_file().display()))?;
    let latest = remote_rev(mode, &locked)?;
    print(&json!({
        "current": locked.rev,
        "latest": latest,
        "available": latest != locked.rev,
        "branch": locked.branch,
        "checked": exec::now(),
    }))
}

fn require_root(mode: &Mode) -> Outcome {
    if !mode.paths.is_system() || mode.dry_run || exec::euid() == Some(0) {
        Ok(())
    } else {
        Err(Failure::Denied("this change needs administrator rights: run it through pkexec flux-config".to_string()))
    }
}

fn caller(mode: &Mode) -> Option<String> {
    let uid: u32 = std::env::var("PKEXEC_UID").ok()?.parse().ok()?;
    let accounts = seed::parse_passwd(&fs::read_to_string(mode.paths.passwd()).ok()?);
    accounts.into_iter().find(|account| account.uid == uid).map(|account| account.name)
}

fn taken_names(mode: &Mode) -> Vec<String> {
    let accounts = seed::parse_passwd(&fs::read_to_string(mode.paths.passwd()).unwrap_or_default());
    accounts.into_iter().map(|account| account.name).collect()
}

fn read_password() -> Result<String, Failure> {
    let mut input = String::new();
    std::io::stdin().take(4096).read_to_string(&mut input).map_err(|error| format!("cannot read the password: {error}"))?;
    let password = input.strip_suffix('\n').unwrap_or(&input).to_string();
    validate::password(&password)?;
    Ok(password)
}

fn on_off(value: &str) -> Result<bool, Failure> {
    match value {
        "on" | "true" | "yes" => Ok(true),
        "off" | "false" | "no" => Ok(false),
        _ => usage("expected on or off"),
    }
}

fn modify(mode: &Mode, settings: &mut Settings, key: &str, value: &str) -> Result<bool, Failure> {
    match key {
        "hostname" => {
            validate::hostname(value)?;
            settings.hostname = value.to_string();
        }
        "timezone" if value == "auto" => settings.timezone = None,
        "timezone" => {
            validate::timezone(value, &mode.paths.zoneinfo())?;
            settings.timezone = Some(value.to_string());
        }
        "locale" => {
            validate::locale(value)?;
            settings.locale = value.to_string();
        }
        "formats" if value == "same" => settings.formats = None,
        "formats" => {
            validate::locale(value)?;
            settings.formats = Some(value.to_string());
        }
        "keymap" => {
            validate::keymap(value)?;
            settings.keymap = value.to_string();
        }
        "auto-update" => {
            settings.updates.automatic = on_off(value)?;
            return Ok(false);
        }
        "software.facile" => settings.software.facile = Some(on_off(value)?),
        "software.apps" => settings.software.apps = Some(on_off(value)?),
        "software.gaming" => settings.software.gaming = Some(on_off(value)?),
        "hardware.nvidia" => settings.hardware.nvidia = Some(on_off(value)?),
        _ => {
            return usage(
                "set takes hostname, timezone, locale, formats, keymap, auto-update, software.facile, software.apps, software.gaming or hardware.nvidia",
            )
        }
    }
    Ok(true)
}

fn set(mode: &Mode, pairs: &[&str], apply: bool) -> Outcome {
    require_root(mode)?;
    let _lock = exec::lock(mode)?;
    let (mut settings, _) = current(mode)?;
    let mut rebuild = false;
    for pair in pairs.chunks(2) {
        rebuild |= modify(mode, &mut settings, pair[0], pair[1])?;
    }
    commit(mode, &format!("set-{}", pairs[0]), &settings, apply && rebuild, |_| Ok(()))
}

fn edit_user(mode: &Mode, settings: &mut Settings, words: &[&str], args: &Args) -> Result<Option<String>, Failure> {
    match words {
        ["add", name] => {
            let full_name = args.full_name.as_deref().ok_or(Failure::Usage("user add needs --full-name".into()))?;
            settings.add_user(name, full_name, args.admin, &taken_names(mode))?;
            Ok(Some(read_password()?))
        }
        ["remove", name] => {
            settings.remove_user(name, caller(mode).as_deref())?;
            Ok(None)
        }
        ["rename", name, full_name] => {
            settings.rename_user(name, full_name)?;
            Ok(None)
        }
        ["set-admin", name, value] => {
            let admin = on_off(value)?;
            if !admin && caller(mode).as_deref() == Some(*name) {
                return Err(Failure::Failed("you cannot take administrator rights from yourself".to_string()));
            }
            settings.set_admin(name, admin)?;
            Ok(None)
        }
        _ => usage("user takes add, remove, rename, set-admin or set-password"),
    }
}

fn set_password(mode: &Mode, name: &str) -> Outcome {
    require_root(mode)?;
    validate::username(name)?;
    let password = read_password()?;
    if !taken_names(mode).iter().any(|taken| taken == name) && mode.paths.is_system() {
        return Err(Failure::Failed(format!("no account called {name:?} exists yet")));
    }
    let mut reporter = Reporter::new(mode, "set-password");
    let result = chpasswd(&mut reporter, name, &password);
    finish(mode, &mut reporter, result)
}

fn chpasswd(reporter: &mut Reporter, name: &str, password: &str) -> Result<(), String> {
    reporter.step(Step::Password, 98, "Setting the password");
    let input = format!("{name}:{password}\n");
    exec::quiet(reporter, "chpasswd", &[], Some(input.as_bytes())).map(|_| ())
}

fn user(mode: &Mode, words: &[&str], args: &Args) -> Outcome {
    if let ["set-password", name] = words {
        return set_password(mode, name);
    }
    require_root(mode)?;
    let _lock = exec::lock(mode)?;
    let (mut settings, _) = current(mode)?;
    let password = edit_user(mode, &mut settings, words, args)?;
    let name = words.get(1).copied().unwrap_or_default().to_string();
    let operation = format!("user-{}", words.first().copied().unwrap_or_default());
    let apply = args.apply || password.is_some();
    commit(mode, &operation, &settings, apply, move |reporter| match &password {
        Some(password) => chpasswd(reporter, &name, password),
        None => Ok(()),
    })
}

fn wire_flake(mode: &Mode) -> Result<(), String> {
    let path = mode.paths.flake_nix();
    let Ok(flake) = fs::read_to_string(&path) else {
        return Ok(());
    };
    let Some(wired) = machine::wire(&flake)? else {
        return Ok(());
    };
    fs::copy(&path, path.with_extension("nix.bak")).map_err(|error| format!("cannot back up {}: {error}", path.display()))?;
    let temp = path.with_extension("nix.tmp");
    fs::write(&temp, wired)
        .and_then(|()| fs::rename(&temp, &path))
        .map_err(|error| format!("cannot update {}: {error}", path.display()))
}

fn write(mode: &Mode, settings: &Settings) -> Result<(), String> {
    if mode.writes_files() {
        wire_flake(mode)?;
        settings::save(&mode.paths.settings(), &mode.paths.backup(), settings)
    } else {
        settings.check(None).map_err(|errors| errors.join("; "))?;
        eprintln!("dry-run: would write {}:\n{}", mode.paths.settings().display(), settings.to_json());
        Ok(())
    }
}

fn restore(mode: &Mode, reporter: &mut Reporter, existed: bool) {
    if !mode.writes_files() {
        return;
    }
    let path = mode.paths.settings();
    let result = if existed {
        settings::copy_atomic(&mode.paths.backup(), &path)
    } else {
        fs::remove_file(&path).map_err(|error| format!("cannot remove {}: {error}", path.display()))
    };
    if let Err(error) = result {
        reporter.log(&format!("could not put the previous settings back: {error}"));
    }
}

fn generation_name(mode: &Mode, number: Option<u32>) -> Option<String> {
    let number = number.or_else(|| generations::current_number(&mode.paths.system_profile()))?;
    generations::store_name(&mode.paths.profiles(), number)
}

fn remember_lock(mode: &Mode, reporter: &mut Reporter) {
    let lock_file = mode.paths.lock_file();
    let Some(generation) = generation_name(mode, None).filter(|_| mode.writes_files() && lock_file.is_file()) else {
        return;
    };
    let locks = mode.paths.locks();
    if let Err(error) = flake::remember(&locks, &generation, &lock_file) {
        reporter.log(&format!("could not keep the lock of this version: {error}"));
    }
    forget_dead_locks(mode);
}

fn forget_dead_locks(mode: &Mode) {
    let profiles = mode.paths.profiles();
    let alive: Vec<String> = generations::list(&profiles, &mode.paths.system_profile())
        .iter()
        .filter_map(|generation| generations::store_name(&profiles, generation.number))
        .collect();
    if mode.writes_files() && !alive.is_empty() {
        flake::forget_except(&mode.paths.locks(), &alive);
    }
}

fn back_up_lock(mode: &Mode) -> Result<(), String> {
    let lock_file = mode.paths.lock_file();
    if !mode.writes_files() || !lock_file.is_file() {
        return Ok(());
    }
    fs::copy(&lock_file, mode.paths.lock_backup()).map_err(|error| format!("cannot back up {}: {error}", lock_file.display()))?;
    let locks = mode.paths.locks();
    match generation_name(mode, None) {
        Some(generation) if !flake::remembers(&locks, &generation) => flake::remember(&locks, &generation, &lock_file),
        _ => Ok(()),
    }
}

fn restore_lock(mode: &Mode, reporter: &mut Reporter) {
    let backup = mode.paths.lock_backup();
    if !mode.writes_files() || !backup.is_file() {
        return;
    }
    if let Err(error) = settings::copy_atomic(&backup, &mode.paths.lock_file()) {
        reporter.log(&format!("could not put the previous flake.lock back: {error}"));
    }
}

fn recall_lock(mode: &Mode, reporter: &mut Reporter, number: u32) {
    let Some(generation) = generation_name(mode, Some(number)).filter(|_| mode.writes_files()) else {
        return;
    };
    match flake::recall(&mode.paths.locks(), &generation, &mode.paths.lock_file()) {
        Ok(true) => reporter.log(&format!("flake.lock is the one version {number} was built from")),
        Ok(false) => reporter.log(&format!("no flake.lock was kept for version {number}; it stays as it is")),
        Err(error) => reporter.log(&format!("could not put the flake.lock of version {number} back: {error}")),
    }
}

/// What a rebuild came to: an error after the system moved to the new version
/// is a warning, because the change is live and is what the machine boots.
fn settle(result: Result<(), String>, before: Option<&str>, after: Option<&str>) -> Result<Option<String>, String> {
    match result {
        Ok(()) => Ok(None),
        Err(error) if after.is_some() && before != after => Ok(Some(error)),
        Err(error) => Err(error),
    }
}

fn applied(mode: &Mode, reporter: &mut Reporter, work: impl FnOnce(&mut Reporter) -> Result<(), String>) -> Result<(), String> {
    let before = link(&mode.paths.current());
    let result = work(reporter);
    let after = link(&mode.paths.current());
    if let Some(warning) = settle(result, before.as_deref(), after.as_deref())? {
        reporter.log(&format!("the new version is in use, but: {warning}"));
        eprintln!("flux-config: warning: the new version is in use, but: {warning}");
    }
    remember_lock(mode, reporter);
    Ok(())
}

fn commit(
    mode: &Mode,
    operation: &str,
    settings: &Settings,
    apply: bool,
    after: impl FnOnce(&mut Reporter) -> Result<(), String>,
) -> Outcome {
    let existed = mode.paths.settings().exists();
    write(mode, settings)?;
    if !apply {
        return print(&json!({ "step": "done", "percent": 100, "message": "Saved" }));
    }
    let mut reporter = Reporter::new(mode, operation);
    let result = applied(mode, &mut reporter, |reporter| rebuild(mode, reporter, 0));
    if result.is_err() {
        restore(mode, &mut reporter, existed);
    }
    let result = result.and_then(|()| after(&mut reporter));
    finish(mode, &mut reporter, result)
}

fn apply(mode: &Mode) -> Outcome {
    require_root(mode)?;
    let _lock = exec::lock(mode)?;
    let mut reporter = Reporter::new(mode, "apply");
    let result = applied(mode, &mut reporter, |reporter| rebuild(mode, reporter, 0));
    finish(mode, &mut reporter, result)
}

fn rebuild(mode: &Mode, reporter: &mut Reporter, base: u8) -> Result<(), String> {
    reporter.step(Step::Evaluate, base.max(5), "Preparing the new version");
    let flake = format!("{}#{FLAKE_ATTR}", mode.paths.flake_dir().display());
    let args = ["switch", "--flake", flake.as_str()].map(String::from);
    exec::streamed(reporter, "nixos-rebuild", &args, &mut Tracker::new(base.max(5)))
}

fn finish(mode: &Mode, reporter: &mut Reporter, result: Result<(), String>) -> Outcome {
    match result {
        Ok(()) => {
            reporter.restart = restart_needed(mode);
            let message = if reporter.restart { "Restart to finish" } else { "Done" };
            reporter.step(Step::Done, 100, message);
            Ok(())
        }
        Err(error) => {
            reporter.log(&error);
            reporter.step(Step::Error, 0, &error);
            Err(Failure::Failed(error))
        }
    }
}

fn update(mode: &Mode, scheduled: bool) -> Outcome {
    require_root(mode)?;
    if scheduled && !current(mode)?.0.updates.automatic {
        println!("automatic updates are off");
        return Ok(());
    }
    let _lock = exec::lock(mode)?;
    let mut reporter = Reporter::new(mode, "update");
    reporter.step(Step::Fetch, 2, "Looking for the new version");
    let dir = mode.paths.flake_dir().display().to_string();
    let args = ["flake", "update", flake::INPUT, "--flake", dir.as_str()].map(String::from);
    let result = back_up_lock(mode).and_then(|()| {
        let result = applied(mode, &mut reporter, |reporter| {
            exec::streamed(reporter, "nix", &args, &mut Tracker::new(2)).and_then(|()| rebuild(mode, reporter, 20))
        });
        if result.is_err() {
            restore_lock(mode, &mut reporter);
        }
        result
    });
    finish(mode, &mut reporter, result)
}

fn target(list: &[Generation], number: Option<&str>) -> Result<u32, Failure> {
    let current = list.iter().find(|generation| generation.current).map(|generation| generation.number);
    let wanted = match number {
        Some(text) => text.parse::<u32>().map_err(|_| Failure::Usage(format!("{text:?} is not a generation number")))?,
        None => list
            .iter()
            .map(|generation| generation.number)
            .find(|number| current.is_some_and(|current| *number < current))
            .ok_or_else(|| Failure::Failed("there is no older version to go back to".to_string()))?,
    };
    if !list.iter().any(|generation| generation.number == wanted) {
        return Err(Failure::Failed(format!("generation {wanted} does not exist")));
    }
    if current == Some(wanted) {
        return Err(Failure::Failed(format!("generation {wanted} is already in use")));
    }
    Ok(wanted)
}

fn rollback(mode: &Mode, number: Option<&str>) -> Outcome {
    require_root(mode)?;
    let list = generations::list(&mode.paths.profiles(), &mode.paths.system_profile());
    let wanted = target(&list, number)?;
    let _lock = exec::lock(mode)?;
    let mut reporter = Reporter::new(mode, "rollback");
    reporter.step(Step::Activate, 20, &format!("Going back to version {wanted}"));
    let profile = mode.paths.system_profile().display().to_string();
    let switch = format!("{profile}/bin/switch-to-configuration");
    let result = exec::quiet(&mut reporter, "nix-env", &["-p".into(), profile, "--switch-generation".into(), wanted.to_string()], None)
        .and_then(|_| {
            recall_lock(mode, &mut reporter, wanted);
            reporter.step(Step::Bootloader, 60, "Switching and updating the boot menu");
            exec::streamed(&mut reporter, &switch, &["switch".to_string()], &mut Tracker::new(60))
        });
    finish(mode, &mut reporter, result)
}

fn gc(mode: &Mode, keep: Option<&str>) -> Outcome {
    require_root(mode)?;
    let keep: u32 = match keep {
        Some(text) => text.parse().ok().filter(|count| (1..=50).contains(count)).ok_or(Failure::Usage("--keep takes 1 to 50".into()))?,
        None => 3,
    };
    let _lock = exec::lock(mode)?;
    let mut reporter = Reporter::new(mode, "gc");
    let profile = mode.paths.system_profile().display().to_string();
    let switch = format!("{}/bin/switch-to-configuration", mode.paths.current().display());
    reporter.step(Step::Clean, 10, "Removing old versions");
    let result = exec::quiet(&mut reporter, "nix-env", &["-p".into(), profile, "--delete-generations".into(), format!("+{keep}")], None)
        .and_then(|_| {
            reporter.step(Step::Clean, 40, "Freeing disk space");
            exec::quiet(&mut reporter, "nix-store", &["--gc".to_string()], None)
        })
        .and_then(|_| {
            reporter.step(Step::Bootloader, 90, "Updating the boot menu");
            exec::quiet(&mut reporter, &switch, &["boot".to_string()], None)
        })
        .map(|_| forget_dead_locks(mode));
    finish(mode, &mut reporter, result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generation(number: u32, current: bool) -> Generation {
        Generation { number, date: 0, version: String::new(), kernel: String::new(), current }
    }

    #[test]
    fn rollback_targets() {
        let list = [generation(3, false), generation(2, true), generation(1, false)];
        assert!(matches!(target(&list, None), Ok(1)));
        assert!(matches!(target(&list, Some("3")), Ok(3)));
        assert!(matches!(target(&list, Some("2")), Err(Failure::Failed(_))));
        assert!(matches!(target(&list, Some("9")), Err(Failure::Failed(_))));
        assert!(matches!(target(&list, Some("x")), Err(Failure::Usage(_))));
        assert!(matches!(target(&[generation(1, true)], None), Err(Failure::Failed(_))));
    }

    #[test]
    fn an_error_after_the_switch_is_a_warning() {
        let failed = || Err("nixos-rebuild failed (exit status: 4)".to_string());
        assert_eq!(settle(Ok(()), Some("/nix/store/a"), Some("/nix/store/b")), Ok(None));
        assert_eq!(settle(failed(), Some("/nix/store/a"), Some("/nix/store/a")), failed().map(|()| None));
        assert_eq!(settle(failed(), None, None), failed().map(|()| None));
        assert_eq!(
            settle(failed(), Some("/nix/store/a"), Some("/nix/store/b")),
            Ok(Some("nixos-rebuild failed (exit status: 4)".to_string()))
        );
    }

    fn mode(name: &str) -> Mode {
        let root = std::env::temp_dir().join(format!("flux-config-ops-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let _ = fs::create_dir_all(root.join("etc/flux"));
        Mode { paths: flux_config::paths::Paths { root }, dry_run: false }
    }

    #[test]
    fn a_failed_update_gets_its_lock_back() {
        let mode = mode("lock");
        let lock_file = mode.paths.lock_file();
        let _ = fs::write(&lock_file, "before");
        assert_eq!(back_up_lock(&mode), Ok(()));
        let _ = fs::write(&lock_file, "after");
        let mut reporter = Reporter::new(&mode, "update");
        restore_lock(&mode, &mut reporter);
        assert_eq!(fs::read_to_string(&lock_file).unwrap_or_default(), "before");
        let _ = fs::remove_dir_all(&mode.paths.root);
    }

    #[test]
    fn a_failed_change_gets_its_settings_back() {
        let mode = mode("settings");
        let first = Settings::default();
        let second = Settings { hostname: "lab".to_string(), ..Settings::default() };
        let mut reporter = Reporter::new(&mode, "set-hostname");
        assert_eq!(write(&mode, &second), Ok(()));
        restore(&mode, &mut reporter, false);
        assert!(!mode.paths.settings().exists());
        assert_eq!(write(&mode, &first).and_then(|()| write(&mode, &second)), Ok(()));
        restore(&mode, &mut reporter, true);
        assert_eq!(settings::load(&mode.paths.settings(), None), Ok(Some(first)));
        let _ = fs::remove_dir_all(&mode.paths.root);
    }
}
