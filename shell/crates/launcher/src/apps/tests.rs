//! The `.desktop` reading checks: the fields the launcher keeps, the entries it
//! drops, the directories it reads and the query matching.

use std::ffi::OsString;

use super::*;

fn env() -> Env {
    Env {
        data_home: None,
        home: Some("/home/ada".into()),
        data_dirs: None,
        desktops: vec!["niri".into()],
        path: Some("/bin:/usr/bin".into()),
    }
}

fn app(text: &str) -> Option<App> {
    entry_from(text).into_app(&env())
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("flux-launcher-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

#[test]
fn parses_fields_and_skips_the_action_groups() {
    let app = app(
        "[Desktop Entry]\nType=Application\nName=Fire\nGenericName=Browser\nIcon=firefox\n\
         Exec=firefox %u\n\n[Desktop Action new]\nExec=firefox --new-window\nName=New\n",
    )
    .expect("visible entry");
    assert_eq!(app.name, "Fire");
    assert_eq!(app.generic.as_deref(), Some("Browser"));
    assert_eq!(app.exec, "firefox %u");
    assert_eq!(app.icon.as_deref(), Some("firefox"));
    assert!(!app.terminal);

    let localized = app_named("[Desktop Entry]\nName=Term\nName[fr]=Terminale\nExec=kitty\n");
    assert_eq!(localized.as_deref(), Some("Term"));
}

fn app_named(text: &str) -> Option<String> {
    app(text).map(|app| app.name)
}

#[test]
fn terminal_entries_are_marked() {
    let app = app("[Desktop Entry]\nType=Application\nName=htop\nExec=htop\nTerminal=true\n")
        .expect("terminal entry");
    assert!(app.terminal);
}

#[test]
fn hidden_and_foreign_entries_are_dropped() {
    assert!(app("[Desktop Entry]\nName=Secret\nNoDisplay=true\nExec=secret\n").is_none());
    assert!(app("[Desktop Entry]\nName=Gone\nHidden=true\nExec=gone\n").is_none());
    assert!(app("[Desktop Entry]\nType=Link\nName=Site\nURL=https://x\nExec=x\n").is_none());
    assert!(app("[Desktop Entry]\nName=NoExec\n").is_none());
    assert!(app("[Desktop Entry]\nName=Blank\nExec=  \n").is_none());
    assert!(app("[Desktop Entry]\nName=Kde\nExec=k\nOnlyShowIn=KDE;\n").is_none());
    assert!(app("[Desktop Entry]\nName=NotNiri\nExec=k\nNotShowIn=niri;\n").is_none());
    assert!(app("[Desktop Entry]\nName=Ours\nExec=k\nOnlyShowIn=GNOME;niri;\n").is_some());
    assert!(app("[Desktop Entry]\nName=Missing\nExec=x\nTryExec=surely-not-installed-xyz\n").is_none());
    assert!(app("[Desktop Entry]\nName=Shell\nExec=sh\nTryExec=sh\n").is_some());
}

#[test]
fn data_dirs_follow_xdg_data_dirs() {
    let mut env = env();
    env.data_dirs = Some(OsString::from(
        "/run/current-system/sw/share:/etc/profiles/per-user/ada/share::relative:/run/current-system/sw/share",
    ));
    assert_eq!(
        data_dirs(&env),
        vec![
            PathBuf::from("/home/ada/.local/share/applications"),
            PathBuf::from("/run/current-system/sw/share/applications"),
            PathBuf::from("/etc/profiles/per-user/ada/share/applications"),
        ]
    );

    env.data_home = Some("/data".into());
    env.data_dirs = Some(OsString::new());
    assert_eq!(
        data_dirs(&env),
        vec![
            PathBuf::from("/data/applications"),
            PathBuf::from("/usr/local/share/applications"),
            PathBuf::from("/usr/share/applications"),
        ]
    );
}

#[test]
fn the_first_directory_claims_a_name() {
    let root = scratch("claims");
    let user = root.join("user/applications");
    let system = root.join("system/applications");
    std::fs::create_dir_all(&user).expect("user dir");
    std::fs::create_dir_all(&system).expect("system dir");
    std::fs::write(user.join("a.desktop"), "[Desktop Entry]\nName=Mine\nExec=a\n").expect("user entry");
    std::fs::write(system.join("a.desktop"), "[Desktop Entry]\nName=Theirs\nExec=a\n").expect("system entry");
    std::fs::write(system.join("b.desktop"), "[Desktop Entry]\nName=Beta\nExec=b\n").expect("system entry");

    let mut env = env();
    env.data_home = Some(root.join("user").into_os_string());
    env.data_dirs = Some(root.join("system").into_os_string());
    let names: Vec<String> = discover(&env).into_iter().map(|app| app.name).collect();
    assert_eq!(names, vec!["Beta", "Mine"]);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn matches_fuzzy_substrings_and_caps_results() {
    let apps = vec![
        App {
            name: "Firefox".into(),
            generic: Some("Web Browser".into()),
            exec: "firefox".into(),
            icon: None,
            terminal: false,
        },
        App {
            name: "Files".into(),
            generic: None,
            exec: "nautilus".into(),
            icon: None,
            terminal: false,
        },
    ];
    assert_eq!(matches(&apps, "FIRE"), vec![0]);
    assert_eq!(matches(&apps, "browser"), vec![0]);
    assert_eq!(matches(&apps, ""), vec![0, 1]);
    assert_eq!(matches(&apps, "zzz"), Vec::<usize>::new());
}
