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

/// Why a command stopped.
pub enum Failure {
    Usage(String),
    Failed(String),
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
    print(&json!({ "exists": exists, "settings": settings }))
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
        Err(Failure::Failed("this change needs administrator rights: run it through pkexec flux-config".to_string()))
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
        _ => return usage("set takes hostname, timezone, locale, formats, keymap or auto-update"),
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

fn restore(mode: &Mode, existed: bool) {
    if !mode.writes_files() {
        return;
    }
    let path = mode.paths.settings();
    let _ = if existed { fs::copy(mode.paths.backup(), &path).map(|_| ()) } else { fs::remove_file(&path) };
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
    let result = rebuild(mode, &mut reporter, 0);
    if result.is_err() {
        restore(mode, existed);
    }
    let result = result.and_then(|()| after(&mut reporter));
    finish(mode, &mut reporter, result)
}

fn apply(mode: &Mode) -> Outcome {
    require_root(mode)?;
    let _lock = exec::lock(mode)?;
    let mut reporter = Reporter::new(mode, "apply");
    let result = rebuild(mode, &mut reporter, 0);
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
    let mut tracker = Tracker::new(2);
    let result = exec::streamed(&mut reporter, "nix", &args, &mut tracker).and_then(|()| rebuild(mode, &mut reporter, 20));
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
        .map(|_| ());
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
}
