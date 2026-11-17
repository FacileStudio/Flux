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
        locale: None,
    }
}

fn app(text: &str) -> Option<App> {
    entry_from(text, &[]).into_app(&env())
}

fn app_in(locale: &str, text: &str) -> Option<App> {
    entry_from(text, &locale_tags(locale)).into_app(&env())
}

fn named(name: &str, generic: Option<&str>, keywords: &[&str]) -> App {
    App {
        name: name.into(),
        generic: generic.map(str::to_string),
        keywords: keywords.iter().map(|keyword| keyword.to_string()).collect(),
        exec: "x".into(),
        icon: None,
        terminal: false,
    }
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

    let untranslated = app_named("[Desktop Entry]\nName=Term\nName[fr]=Terminale\nExec=kitty\n");
    assert_eq!(untranslated.as_deref(), Some("Term"), "no locale, so the bare key");
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
fn a_locale_reads_as_its_tags_best_first() {
    assert_eq!(locale_tags("fr_FR.UTF-8"), vec!["fr_FR", "fr"]);
    assert_eq!(locale_tags("sr_RS@latin"), vec!["sr_RS@latin", "sr_RS", "sr@latin", "sr"]);
    assert_eq!(locale_tags("de"), vec!["de"]);
    assert!(locale_tags("C.UTF-8").is_empty());
    assert!(locale_tags("POSIX").is_empty());
    assert!(locale_tags("").is_empty());
}

#[test]
fn the_session_language_picks_the_translated_strings() {
    let entry = "[Desktop Entry]\nName[fr]=Fichiers\nName=Files\nName[fr_CA]=Dossiers\nName[de]=Dateien\n\
                 GenericName=File Manager\nGenericName[fr]=Gestionnaire de fichiers\n\
                 Keywords=folder;manager;\nKeywords[fr]=dossier;gestionnaire;\nExec=nautilus\nExec[fr]=wrong\n";
    let french = app_in("fr_FR.UTF-8", entry).expect("entry");
    assert_eq!(french.name, "Fichiers");
    assert_eq!(french.generic.as_deref(), Some("Gestionnaire de fichiers"));
    assert_eq!(french.keywords, vec!["dossier", "gestionnaire"]);
    assert_eq!(french.exec, "nautilus");

    assert_eq!(app_in("fr_CA.UTF-8", entry).expect("entry").name, "Dossiers");
    let english = app_in("en_US.UTF-8", entry).expect("entry");
    assert_eq!(english.name, "Files");
    assert_eq!(english.keywords, vec!["folder", "manager"]);
    assert_eq!(app_in("C", entry).expect("entry").name, "Files");
}

#[test]
fn an_empty_query_lists_every_app() {
    let apps: Vec<App> = (0..120).map(|index| named(&format!("App {index:03}"), None, &[])).collect();
    assert_eq!(matches(&apps, ""), (0..120).collect::<Vec<_>>());
    assert_eq!(matches(&apps, "  "), (0..120).collect::<Vec<_>>());
    assert_eq!(matches(&apps, "app").len(), MAX_MATCHES);
}

#[test]
fn a_query_is_ranked_before_it_is_cut() {
    let apps = vec![
        named("Calls", None, &[]),
        named("Disks", None, &[]),
        named("Files", Some("File Manager"), &["folder"]),
        named("Paramètres", None, &["wifi", "réseau"]),
        named("Sound Recorder", None, &[]),
        named("Text Editor", None, &["notes"]),
        named("Web", Some("Browser"), &["internet", "sites"]),
    ];
    assert_eq!(matches(&apps, "s"), vec![4, 0, 1, 2, 3, 5, 6]);
    assert_eq!(matches(&apps, "ed"), vec![5]);
    assert_eq!(matches(&apps, "INTERNET"), vec![6]);
    assert_eq!(matches(&apps, "parametres"), vec![3]);
    assert_eq!(matches(&apps, "reseau"), vec![3]);
    assert_eq!(matches(&apps, "manager"), vec![2]);
    assert_eq!(matches(&apps, "zzz"), Vec::<usize>::new());
}
