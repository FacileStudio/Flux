//! The first `settings.json` of a machine installed before the file existed:
//! read back from what the running system already says.

use std::fs;

use crate::paths::Paths;
use crate::settings::{default_groups, Settings, User};
use crate::validate;

/// An account in `/etc/passwd`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    pub name: String,
    pub uid: u32,
    pub gecos: String,
}

/// The accounts of a passwd document.
pub fn parse_passwd(document: &str) -> Vec<Account> {
    document
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split(':').collect();
            Some(Account {
                name: fields.first()?.to_string(),
                uid: fields.get(2)?.parse().ok()?,
                gecos: fields.get(4).map(|gecos| gecos.split(',').next().unwrap_or("").to_string())?,
            })
        })
        .collect()
}

/// The people (uid 1000 to 29999) of a passwd document.
pub fn people(accounts: &[Account]) -> Vec<&Account> {
    accounts.iter().filter(|account| (1000..30000).contains(&account.uid)).collect()
}

/// The members of `group` in a group document.
pub fn group_members(document: &str, group: &str) -> Vec<String> {
    document
        .lines()
        .find_map(|line| {
            let fields: Vec<&str> = line.split(':').collect();
            (fields.first() == Some(&group)).then(|| fields.get(3).copied().unwrap_or(""))
        })
        .map(|members| members.split(',').filter(|m| !m.is_empty()).map(String::from).collect())
        .unwrap_or_default()
}

/// The xkb layout an `00-keyboard.conf` names.
pub fn xkb_layout(document: &str) -> Option<String> {
    document.lines().find_map(|line| {
        let rest = line.trim().strip_prefix("Option")?.trim();
        let rest = rest.strip_prefix("\"XkbLayout\"")?.trim();
        Some(rest.trim_matches('"').split(',').next()?.to_string())
    })
}

/// `LANG` in a `locale.conf`.
pub fn lang(document: &str) -> Option<String> {
    document.lines().find_map(|line| line.trim().strip_prefix("LANG=").map(|v| v.trim_matches('"').to_string()))
}

/// The zone a `/etc/localtime` link target points at.
pub fn zone_from_link(target: &str) -> Option<String> {
    target.split_once("zoneinfo/").map(|(_, zone)| zone.to_string()).filter(|zone| validate::timezone_shape(zone))
}

fn read(paths: &Paths, path: &str) -> String {
    fs::read_to_string(paths.root.join(path)).unwrap_or_default()
}

/// The settings the running system already has.
pub fn from_system(paths: &Paths) -> Settings {
    let mut settings = Settings::default();
    let hostname = read(paths, "etc/hostname").trim().to_string();
    if validate::hostname(&hostname).is_ok() {
        settings.hostname = hostname;
    }
    settings.timezone = fs::read_link(paths.root.join("etc/localtime"))
        .ok()
        .and_then(|target| zone_from_link(&target.to_string_lossy()))
        .or(settings.timezone);
    if let Some(locale) = lang(&read(paths, "etc/locale.conf")).filter(|l| validate::locale(l).is_ok()) {
        settings.locale = locale;
    }
    if let Some(layout) = xkb_layout(&read(paths, "etc/X11/xorg.conf.d/00-keyboard.conf")) {
        if validate::keymap(&layout).is_ok() {
            settings.keymap = layout;
        }
    }
    let wheel = group_members(&read(paths, "etc/group"), "wheel");
    let accounts = parse_passwd(&read(paths, "etc/passwd"));
    settings.users = people(&accounts)
        .into_iter()
        .filter(|account| validate::username(&account.name).is_ok())
        .map(|account| {
            let full_name = Some(account.gecos.trim().to_string())
                .filter(|name| validate::full_name(name).is_ok())
                .unwrap_or_else(|| account.name.clone());
            User { name: account.name.clone(), full_name, admin: wheel.contains(&account.name), groups: default_groups() }
        })
        .collect();
    if !settings.users.is_empty() && !settings.users.iter().any(|user| user.admin) {
        settings.users[0].admin = true;
    }
    settings
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASSWD: &str = "root:x:0:0:System administrator:/root:/bin/sh\n\
        ada:x:1000:100:Ada Lovelace,,,:/home/ada:/bin/zsh\n\
        bob:x:1001:100::/home/bob:/bin/zsh\n\
        nobody:x:65534:65534:Unprivileged account:/var/empty:/bin/false\n";

    #[test]
    fn passwd_and_group() {
        let accounts = parse_passwd(PASSWD);
        let names: Vec<&str> = people(&accounts).iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["ada", "bob"]);
        assert_eq!(accounts[1].gecos, "Ada Lovelace");
        assert_eq!(group_members("users:x:100:\nwheel:x:1:ada,carol\n", "wheel"), ["ada", "carol"]);
        assert!(group_members("wheel:x:1:\n", "wheel").is_empty());
    }

    #[test]
    fn small_files() {
        let conf = "Section \"InputClass\"\n  Option \"XkbLayout\" \"fr,us\"\nEndSection\n";
        assert_eq!(xkb_layout(conf).as_deref(), Some("fr"));
        assert_eq!(lang("LANG=fr_FR.UTF-8\nLC_TIME=x\n").as_deref(), Some("fr_FR.UTF-8"));
        assert_eq!(zone_from_link("/etc/zoneinfo/Europe/Paris").as_deref(), Some("Europe/Paris"));
        assert_eq!(zone_from_link("/nix/store/x-tzdata/share/zoneinfo/UTC").as_deref(), Some("UTC"));
        assert_eq!(zone_from_link("/etc/static/localtime"), None);
    }

    #[test]
    fn seeds_from_a_root() {
        let root = std::env::temp_dir().join(format!("flux-config-seed-{}", std::process::id()));
        let _ = fs::create_dir_all(root.join("etc"));
        let _ = fs::write(root.join("etc/hostname"), "lab\n");
        let _ = fs::write(root.join("etc/passwd"), PASSWD);
        let _ = fs::write(root.join("etc/group"), "wheel:x:1:bob\n");
        let _ = fs::write(root.join("etc/locale.conf"), "LANG=fr_FR.UTF-8\n");
        let settings = from_system(&Paths { root: root.clone() });
        assert_eq!(settings.hostname, "lab");
        assert_eq!(settings.locale, "fr_FR.UTF-8");
        assert_eq!(settings.timezone.as_deref(), Some("UTC"));
        let users: Vec<(&str, &str, bool)> =
            settings.users.iter().map(|u| (u.name.as_str(), u.full_name.as_str(), u.admin)).collect();
        assert_eq!(users, [("ada", "Ada Lovelace", false), ("bob", "bob", true)]);
        assert!(settings.check(None).is_ok());
        let _ = fs::remove_dir_all(&root);
    }
}
