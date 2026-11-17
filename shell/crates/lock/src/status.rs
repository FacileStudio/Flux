//! What the lock screen shows besides the clock: who is locked out, the
//! battery, the keyboard layout. Every reader returns `None` when it cannot
//! tell, and the screen then leaves that line out.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The default wallpaper the desktop module installs.
pub const DEFAULT_WALLPAPER: &str = "/run/current-system/sw/share/backgrounds/flux/default.jpg";

/// The login name and the display name of the user with `uid`, from a
/// `passwd(5)` document. The display name is the first GECOS field, or the
/// login name when that is empty.
pub fn user_from_passwd(passwd: &str, uid: u32) -> Option<(String, String)> {
    passwd.lines().find_map(|line| {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 5 || fields[2].parse::<u32>().ok()? != uid || fields[0].is_empty() {
            return None;
        }
        let real = fields[4].split(',').next().unwrap_or("").trim();
        let display = if real.is_empty() { fields[0] } else { real };
        Some((fields[0].to_string(), display.to_string()))
    })
}

/// The user this process runs as: `/etc/passwd` first, `$USER` second.
pub fn current_user() -> Option<(String, String)> {
    let uid = unsafe { libc::getuid() };
    std::fs::read_to_string("/etc/passwd")
        .ok()
        .and_then(|passwd| user_from_passwd(&passwd, uid))
        .or_else(|| {
            let name = std::env::var("USER").ok().filter(|name| !name.is_empty())?;
            Some((name.clone(), name))
        })
}

/// The battery charge in percent and whether it is charging, read from one
/// `power_supply` directory's `capacity` and `status` files.
pub fn battery_from(capacity: &str, status: &str) -> Option<(u8, bool)> {
    let level = capacity.trim().parse::<u8>().ok()?.min(100);
    let charging = matches!(status.trim(), "Charging" | "Full");
    Some((level, charging))
}

/// The first system battery under `/sys/class/power_supply`, if there is one.
pub fn battery() -> Option<(u8, bool)> {
    let entries = std::fs::read_dir("/sys/class/power_supply").ok()?;
    entries.flatten().find_map(|entry| {
        let path = entry.path();
        let kind = std::fs::read_to_string(path.join("type")).ok()?;
        let scope = std::fs::read_to_string(path.join("scope")).unwrap_or_default();
        if kind.trim() != "Battery" || scope.trim() == "Device" {
            return None;
        }
        battery_from(
            &std::fs::read_to_string(path.join("capacity")).ok()?,
            &std::fs::read_to_string(path.join("status")).unwrap_or_default(),
        )
    })
}

/// The active keyboard layout from `niri msg --json keyboard-layouts`.
pub fn layout_from_niri(json: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    let index = usize::try_from(value.get("current_idx")?.as_u64()?).ok()?;
    let name = value.get("names")?.as_array()?.get(index)?.as_str()?;
    (!name.is_empty()).then(|| name.to_string())
}

/// The active keyboard layout, asked from niri.
pub fn layout() -> Option<String> {
    let output = Command::new("niri")
        .args(["msg", "--json", "keyboard-layouts"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    layout_from_niri(&String::from_utf8_lossy(&output.stdout))
}

/// The wallpaper the user picked: `~/.config/flux/wallpaper` is either a link
/// to the image or a file whose first line is its path.
pub fn wallpaper_choice(choice: &Path) -> Option<PathBuf> {
    let link = std::fs::read_link(choice).ok().map(|target| {
        if target.is_absolute() {
            target
        } else {
            choice.parent().map(|dir| dir.join(&target)).unwrap_or(target)
        }
    });
    let path = link.or_else(|| {
        let text = std::fs::read_to_string(choice).ok()?;
        let line = text.lines().next()?.trim();
        (!line.is_empty()).then(|| PathBuf::from(line))
    })?;
    path.is_file().then_some(path)
}

/// The wallpaper to draw behind the lock: the user's choice, else the default.
pub fn wallpaper() -> PathBuf {
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".config/flux/wallpaper"))
        .and_then(|choice| wallpaper_choice(&choice))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_WALLPAPER))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passwd_gives_the_gecos_name_or_the_login() {
        let passwd = "root:x:0:0:System administrator:/root:/bin/sh\n\
                      ada:x:1000:100:Ada Lovelace,,,:/home/ada:/bin/sh\n\
                      bob:x:1001:100::/home/bob:/bin/sh\n";
        assert_eq!(
            user_from_passwd(passwd, 1000),
            Some(("ada".into(), "Ada Lovelace".into()))
        );
        assert_eq!(user_from_passwd(passwd, 1001), Some(("bob".into(), "bob".into())));
        assert_eq!(user_from_passwd(passwd, 42), None);
        assert_eq!(user_from_passwd("garbage", 0), None);
    }

    #[test]
    fn battery_files_parse() {
        assert_eq!(battery_from("87\n", "Discharging\n"), Some((87, false)));
        assert_eq!(battery_from("100", "Full"), Some((100, true)));
        assert_eq!(battery_from("", "Charging"), None);
    }

    #[test]
    fn niri_layouts_parse() {
        let json = r#"{"names":["English (US)","French"],"current_idx":1}"#;
        assert_eq!(layout_from_niri(json), Some("French".into()));
        assert_eq!(layout_from_niri(r#"{"names":[],"current_idx":0}"#), None);
        assert_eq!(layout_from_niri("not json"), None);
    }

    #[test]
    fn wallpaper_choice_reads_a_link_or_a_path_file() {
        let dir = std::env::temp_dir().join(format!("flux-lock-wall-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        let image = dir.join("hills.jpg");
        std::fs::write(&image, b"jpg").expect("image");
        let file = dir.join("choice");
        std::fs::write(&file, format!("{}\n", image.display())).expect("choice file");
        assert_eq!(wallpaper_choice(&file), Some(image.clone()));
        let link = dir.join("link");
        std::os::unix::fs::symlink("hills.jpg", &link).expect("link");
        assert_eq!(wallpaper_choice(&link), Some(image));
        std::fs::write(&file, "/nowhere/at/all.jpg").expect("missing");
        assert_eq!(wallpaper_choice(&file), None);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
