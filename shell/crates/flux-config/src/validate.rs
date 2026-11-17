//! Strict checks for every value a person can type, shared by the helper, the
//! pages and (as regular expressions) `modules/settings.nix`.

use std::path::Path;

/// The locales Flux generates, the same list `flux-install` offers.
pub const LOCALES: &[&str] = &[
    "en_US.UTF-8",
    "en_GB.UTF-8",
    "fr_FR.UTF-8",
    "fr_BE.UTF-8",
    "fr_CH.UTF-8",
    "fr_CA.UTF-8",
    "es_ES.UTF-8",
    "de_DE.UTF-8",
    "de_CH.UTF-8",
    "it_IT.UTF-8",
    "pt_PT.UTF-8",
    "pt_BR.UTF-8",
    "nl_NL.UTF-8",
];

/// The xkb layouts Flux accepts, the same list `flux-install` offers.
pub const KEYMAPS: &[&str] = &[
    "us", "gb", "fr", "de", "es", "it", "pt", "br", "be", "ch", "ca", "nl", "se", "no", "dk", "fi", "pl", "cz",
    "latam",
];

/// Names a person may not take: system accounts and groups NixOS or Flux uses.
pub const RESERVED_USERS: &[&str] = &[
    "root", "bin", "daemon", "adm", "lp", "sync", "shutdown", "halt", "mail", "news", "uucp", "operator", "games",
    "ftp", "nobody", "nogroup", "nixbld", "messagebus", "polkituser", "sshd", "avahi", "greeter", "nscd", "dbus",
    "users", "wheel", "admin", "audio", "video", "input", "kvm", "render", "disk", "tty", "networkmanager",
    "geoclue", "rtkit", "pipewire", "systemd-network", "systemd-resolve", "systemd-timesync", "systemd-oom",
    "systemd-coredump", "man", "sudo", "dialout", "floppy", "cdrom", "tape", "kmem", "utmp", "lock", "regreet",
    "colord", "cups", "fwupd-refresh", "flatpak", "nm-openvpn", "nm-iodine", "usbmux", "tss", "uinput",
    "systemd-journal",
];

/// The extra groups a person may be given besides `wheel`.
pub const ALLOWED_GROUPS: &[&str] = &["networkmanager", "video", "audio", "input", "lp", "scanner", "dialout"];

/// Why a value was refused, in words a log or a terminal can show.
pub type Invalid = String;

/// An RFC 1123 host label: 1 to 63 letters, digits or hyphens, no hyphen at either end.
pub fn hostname(name: &str) -> Result<(), Invalid> {
    let ok = !name.is_empty()
        && name.len() <= 63
        && !name.starts_with('-')
        && !name.ends_with('-')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    ok.then_some(()).ok_or_else(|| {
        format!("invalid device name {name:?}: use 1 to 63 letters, digits or hyphens, not starting or ending with a hyphen")
    })
}

/// A login name: `^[a-z_][a-z0-9_-]{0,31}$` and not a reserved name.
pub fn username(name: &str) -> Result<(), Invalid> {
    let mut chars = name.chars();
    let shape = match chars.next() {
        Some(first) => {
            name.len() <= 32
                && (first.is_ascii_lowercase() || first == '_')
                && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        }
        None => false,
    };
    if !shape {
        return Err(format!(
            "invalid username {name:?}: start with a lowercase letter, then up to 31 lowercase letters, digits, _ or -"
        ));
    }
    if RESERVED_USERS.contains(&name) || name.starts_with("systemd-") || name.starts_with("nixbld") {
        return Err(format!("the username {name:?} is reserved by the system"));
    }
    Ok(())
}

/// A display name: 1 to 64 characters, no control characters and nothing a
/// passwd field cannot hold (`:` and newlines).
pub fn full_name(name: &str) -> Result<(), Invalid> {
    let ok = !name.trim().is_empty()
        && name.chars().count() <= 64
        && name.trim() == name
        && !name.chars().any(|c| c.is_control() || c == ':' || c == ',');
    ok.then_some(())
        .ok_or_else(|| format!("invalid full name {name:?}: 1 to 64 characters, no colon, comma or control character"))
}

/// A password `chpasswd` accepts: 1 to 512 bytes, no newline, no NUL.
pub fn password(secret: &str) -> Result<(), Invalid> {
    let ok = !secret.is_empty() && secret.len() <= 512 && !secret.contains(['\n', '\r', '\0']);
    ok.then_some(()).ok_or_else(|| "the password is empty, too long or holds a line break".to_string())
}

/// The shape of an IANA zone name such as `Europe/Paris` or `UTC`.
pub fn timezone_shape(zone: &str) -> bool {
    !zone.is_empty()
        && zone.len() <= 64
        && !zone.starts_with('/')
        && !zone.ends_with('/')
        && zone.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part.chars().all(|c| c.is_ascii_alphanumeric() || "_+-".contains(c))
        })
}

/// A zone that exists as a file under `zoneinfo`.
pub fn timezone(zone: &str, zoneinfo: &Path) -> Result<(), Invalid> {
    if timezone_shape(zone) && zoneinfo.join(zone).is_file() {
        Ok(())
    } else {
        Err(format!("unknown time zone {zone:?}"))
    }
}

/// One of [`LOCALES`].
pub fn locale(name: &str) -> Result<(), Invalid> {
    LOCALES
        .contains(&name)
        .then_some(())
        .ok_or_else(|| format!("unknown locale {name:?} (one of: {})", LOCALES.join(" ")))
}

/// One of [`KEYMAPS`].
pub fn keymap(name: &str) -> Result<(), Invalid> {
    KEYMAPS
        .contains(&name)
        .then_some(())
        .ok_or_else(|| format!("unknown keyboard layout {name:?} (one of: {})", KEYMAPS.join(" ")))
}

/// One of [`ALLOWED_GROUPS`].
pub fn group(name: &str) -> Result<(), Invalid> {
    ALLOWED_GROUPS
        .contains(&name)
        .then_some(())
        .ok_or_else(|| format!("the group {name:?} cannot be given (one of: {})", ALLOWED_GROUPS.join(" ")))
}

fn ascii(c: char) -> &'static str {
    match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => "a",
        'ç' => "c",
        'è' | 'é' | 'ê' | 'ë' => "e",
        'ì' | 'í' | 'î' | 'ï' => "i",
        'ñ' => "n",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => "o",
        'ù' | 'ú' | 'û' | 'ü' => "u",
        'ý' | 'ÿ' => "y",
        'æ' => "ae",
        'œ' => "oe",
        'ß' => "ss",
        _ => "",
    }
}

fn fold(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii() { c.to_string() } else { ascii(c).to_string() })
        .collect()
}

/// The username a full name suggests: the first name, folded to ASCII, as the installer does.
pub fn derive_username(full_name: &str) -> String {
    let first = fold(full_name.split_whitespace().next().unwrap_or(""));
    let mut name: String = first
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_' || *c == '-')
        .collect();
    if name.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
        name.insert(0, 'u');
    }
    name.truncate(32);
    name
}

/// The device name a person's text suggests: `Ada's Laptop` becomes `adas-laptop`.
pub fn derive_hostname(text: &str) -> String {
    let folded: String = fold(text)
        .chars()
        .filter(|c| *c != '\'')
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let mut name = String::new();
    for c in folded.chars() {
        if c != '-' || !name.ends_with('-') {
            name.push(c);
        }
    }
    name.truncate(63);
    name.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_match_the_nix_rules() {
        let rules: serde_json::Value = serde_json::from_str(include_str!("../rules.json")).unwrap_or_default();
        let list = |key: &str| -> Vec<String> {
            rules[key].as_array().map(|items| items.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default()
        };
        assert_eq!(list("locales"), LOCALES);
        assert_eq!(list("keymaps"), KEYMAPS);
        assert_eq!(list("reservedUsers"), RESERVED_USERS);
        assert_eq!(list("groups"), ALLOWED_GROUPS);
    }

    #[test]
    fn hostnames() {
        for good in ["flux", "ada-flux", "A1", "x", &"a".repeat(63)] {
            assert!(hostname(good).is_ok(), "{good}");
        }
        for bad in ["", "-pc", "pc-", "my.pc", "my pc", "é", &"a".repeat(64)] {
            assert!(hostname(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn usernames() {
        for good in ["ada", "_svc", "a-b_9", &"a".repeat(32)] {
            assert!(username(good).is_ok(), "{good}");
        }
        for bad in ["", "Ada", "9ada", "-ada", "ada!", "root", "wheel", "systemd-foo", "nixbld3", &"a".repeat(33)] {
            assert!(username(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn full_names() {
        assert!(full_name("Ada Lovelace").is_ok());
        assert!(full_name("Zoë O'Brien").is_ok());
        for bad in ["", "  ", " Ada", "a:b", "a,b", "a\nb", &"a".repeat(65)] {
            assert!(full_name(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn passwords() {
        assert!(password("correct horse").is_ok());
        assert!(password("").is_err());
        assert!(password("a\nb").is_err());
    }

    #[test]
    fn zone_shapes() {
        assert!(timezone_shape("Europe/Paris"));
        assert!(timezone_shape("America/Argentina/Buenos_Aires"));
        assert!(timezone_shape("Etc/GMT+3"));
        for bad in ["", "/etc/passwd", "../x", "Europe/../../x", "Europe/", "a b"] {
            assert!(!timezone_shape(bad), "{bad}");
        }
    }

    #[test]
    fn zones_must_exist() {
        let dir = std::env::temp_dir().join(format!("flux-config-zones-{}", std::process::id()));
        let _ = std::fs::create_dir_all(dir.join("Europe"));
        let _ = std::fs::write(dir.join("Europe/Paris"), "TZif");
        assert!(timezone("Europe/Paris", &dir).is_ok());
        assert!(timezone("Europe/Nowhere", &dir).is_err());
        assert!(timezone("Europe", &dir).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn lists() {
        assert!(locale("fr_FR.UTF-8").is_ok());
        assert!(locale("xx_XX").is_err());
        assert!(keymap("latam").is_ok());
        assert!(keymap("qwerty").is_err());
        assert!(group("video").is_ok());
        assert!(group("wheel").is_err());
    }

    #[test]
    fn derivations() {
        assert_eq!(derive_username("Élodie Durand"), "elodie");
        assert_eq!(derive_username("42 Ada"), "u42");
        assert_eq!(derive_hostname("Ada's Laptop"), "adas-laptop");
        assert_eq!(derive_hostname("  --Bureau   d'Élodie--"), "bureau-delodie");
        assert!(hostname(&derive_hostname(&"é".repeat(80))).is_ok());
    }
}
