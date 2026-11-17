use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("flux-config-cli-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let _ = fs::create_dir_all(root.join("etc/zoneinfo/Europe"));
    let _ = fs::write(root.join("etc/zoneinfo/Europe/Paris"), "TZif");
    let _ = fs::write(root.join("etc/zoneinfo/UTC"), "TZif");
    let _ = fs::write(root.join("etc/hostname"), "flux\n");
    let _ = fs::write(root.join("etc/passwd"), "root:x:0:0::/root:/bin/sh\nada:x:1000:100:Ada Lovelace:/home/ada:/bin/zsh\n");
    let _ = fs::write(root.join("etc/group"), "wheel:x:1:ada\n");
    let _ = fs::create_dir_all(root.join("etc/flux"));
    let _ = fs::write(root.join("etc/flux/flake.nix"), "{\n  modules = [\n    ./configuration.nix\n  ];\n}\n");
    root
}

fn run(root: &Path, args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_flux-config"))
        .args(args)
        .env("FLUX_CONFIG_ROOT", root)
        .env("PKEXEC_UID", "1000")
        .env_remove("TZDIR")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("cannot start flux-config: {error}"));
    if let Some(mut input) = child.stdin.take() {
        let _ = input.write_all(stdin.as_bytes());
    }
    child.wait_with_output().unwrap_or_else(|error| panic!("flux-config did not finish: {error}"))
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn settings(root: &Path) -> serde_json::Value {
    let document = fs::read_to_string(root.join("etc/flux/settings.json")).unwrap_or_default();
    serde_json::from_str(&document).unwrap_or(serde_json::Value::Null)
}

#[test]
fn edits_settings_and_reports_progress() {
    let root = root("edit");
    let out = run(&root, &["set", "hostname", "ada-laptop"], "");
    assert!(out.status.success(), "{}", text(&out.stderr));
    let lines: Vec<String> = text(&out.stdout).lines().map(String::from).collect();
    assert!(lines.first().is_some_and(|line| line.contains("\"step\":\"start\"")), "{lines:?}");
    assert!(lines.last().is_some_and(|line| line.contains("\"step\":\"done\"")), "{lines:?}");
    assert!(text(&out.stderr).contains("dry-run: nixos-rebuild switch --flake"));
    let saved = settings(&root);
    assert_eq!(saved["hostname"], "ada-laptop");
    assert_eq!(saved["users"][0]["name"], "ada");
    assert_eq!(saved["users"][0]["admin"], true);
    let state = fs::read_to_string(root.join("run/flux-config/state.json")).unwrap_or_default();
    assert!(state.contains("\"operation\":\"set-hostname\"") && state.contains("\"done\""), "{state}");
    let flake = fs::read_to_string(root.join("etc/flux/flake.nix")).unwrap_or_default();
    assert!(flake.contains("    ./configuration.nix\n    { flux.settingsFile = ./settings.json; }\n"), "{flake}");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn several_values_and_accounts() {
    let root = root("values");
    let out = run(&root, &["set", "timezone", "Europe/Paris", "locale", "fr_FR.UTF-8", "formats", "fr_BE.UTF-8"], "");
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(text(&out.stderr).matches("dry-run: nixos-rebuild").count(), 1);
    assert_eq!(settings(&root)["timezone"], "Europe/Paris");
    assert_eq!(settings(&root)["formats"], "fr_BE.UTF-8");
    let out = run(&root, &["set", "timezone", "auto", "--no-apply"], "");
    assert!(out.status.success());
    assert!(settings(&root)["timezone"].is_null());

    let out = run(&root, &["user", "add", "bob", "--full-name", "Bob Marley"], "hunter2\n");
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(text(&out.stderr).contains("dry-run: chpasswd < (stdin)"));
    assert!(!text(&out.stderr).contains("hunter2") && !text(&out.stdout).contains("hunter2"));
    assert_eq!(settings(&root)["users"][1]["fullName"], "Bob Marley");
    assert!(root.join("etc/flux/settings.json.bak").is_file());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn refuses_bad_input_without_writing() {
    let root = root("refuse");
    for args in [
        vec!["set", "hostname", "bad-"],
        vec!["set", "hostname", "a.b"],
        vec!["set", "timezone", "../../etc/passwd"],
        vec!["set", "timezone", "Mars/Olympus"],
        vec!["set", "locale", "xx_XX.UTF-8"],
        vec!["user", "remove", "ada"],
        vec!["user", "set-admin", "ada", "off"],
        vec!["user", "add", "root", "--full-name", "Root"],
        vec!["user", "add", "ada", "--full-name", "Again"],
    ] {
        let out = run(&root, &args, "pw\n");
        assert!(!out.status.success(), "{args:?} was accepted");
        assert!(!root.join("etc/flux/settings.json").exists(), "{args:?} wrote the file");
    }
    let out = run(&root, &["user", "add", "eve", "--full-name", "Eve"], "");
    assert!(!out.status.success(), "an empty password was accepted");
    let out = run(&root, &["frobnicate"], "");
    assert_eq!(out.status.code(), Some(2));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn one_change_at_a_time() {
    let root = root("lock");
    let _ = fs::create_dir_all(root.join("run/flux-config"));
    let held = fs::OpenOptions::new().write(true).create(true).truncate(false).open(root.join("run/flux-config/lock"));
    let held = held.unwrap_or_else(|error| panic!("cannot open the lock: {error}"));
    let _ = held.lock();
    let out = run(&root, &["apply"], "");
    assert!(!out.status.success());
    assert!(text(&out.stderr).contains("already running"), "{}", text(&out.stderr));
    drop(held);
    let out = run(&root, &["apply"], "");
    assert!(out.status.success(), "{}", text(&out.stderr));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn read_only_commands() {
    let root = root("read");
    let out = run(&root, &["get"], "");
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or(serde_json::Value::Null);
    assert_eq!(value["exists"], false);
    assert_eq!(value["settings"]["users"][0]["fullName"], "Ada Lovelace");
    let out = run(&root, &["generations"], "");
    assert_eq!(text(&out.stdout).trim(), "[]");
    let out = run(&root, &["status"], "");
    assert!(text(&out.stdout).contains("\"running\":false"));
    let out = run(&root, &["rollback"], "");
    assert!(!out.status.success());
    let _ = fs::remove_dir_all(&root);
}
