//! The swayidle command line `flux-idle` builds from the power rules.
//!
//! swayidle runs every command through `sh -c`, so each one is assembled from
//! quoted store paths. `-w` makes it wait for a command before it goes on,
//! which is what holds sleep until `flux-lock --daemon` reports the session
//! locked.

use crate::config::{Lid, PowerConfig};

/// The programs the command line calls, by absolute path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Programs {
    pub swayidle: String,
    pub lock: String,
    pub niri: String,
    pub inhibit: String,
    pub idle: String,
}

/// `value` quoted for `sh`.
pub fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

/// The program and arguments to exec. `lock` false leaves every lock rule out,
/// for machines without a password such as a kiosk.
pub fn command(config: &PowerConfig, programs: &Programs, lock: bool) -> Vec<String> {
    let locker = format!("{} --daemon", quote(&programs.lock));
    let mut args = vec![programs.swayidle.clone(), "-w".to_string()];
    let mut timeout = |seconds: Option<u32>, run: String| {
        if let Some(seconds) = seconds {
            args.extend(["timeout".to_string(), seconds.to_string(), run]);
        }
    };
    if lock {
        timeout(config.lock, locker.clone());
    }
    timeout(
        config.screen_off,
        format!("{} msg action power-off-monitors", quote(&programs.niri)),
    );
    timeout(config.suspend_battery, format!("{} suspend-if battery", quote(&programs.idle)));
    timeout(config.suspend_ac, format!("{} suspend-if ac", quote(&programs.idle)));
    if lock {
        args.extend(["before-sleep".to_string(), locker.clone(), "lock".to_string(), locker]);
    }
    if config.lid == Lid::Nothing {
        let mut inhibited = vec![
            programs.inhibit.clone(),
            "--what=handle-lid-switch".to_string(),
            "--who=Flux".to_string(),
            "--why=Lid close is set to do nothing".to_string(),
            "--mode=block".to_string(),
        ];
        inhibited.append(&mut args);
        return inhibited;
    }
    args
}

/// Whether the machine runs on mains power, from each `power_supply` entry's
/// `type` and `online`. A machine with no mains entry at all (a desktop) counts
/// as plugged in.
pub fn on_mains(supplies: &[(String, String)]) -> bool {
    let mut mains = supplies.iter().filter(|(kind, _)| kind.trim() == "Mains").peekable();
    mains.peek().is_none() || mains.any(|(_, online)| online.trim() == "1")
}

/// Read every `/sys/class/power_supply` entry's `type` and `online`.
pub fn supplies() -> Vec<(String, String)> {
    let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let kind = std::fs::read_to_string(entry.path().join("type")).ok()?;
            let online = std::fs::read_to_string(entry.path().join("online")).unwrap_or_default();
            Some((kind, online))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn programs() -> Programs {
        Programs {
            swayidle: "/s/swayidle".into(),
            lock: "/s/flux-lock".into(),
            niri: "/s/niri".into(),
            inhibit: "/s/systemd-inhibit".into(),
            idle: "/s/flux-idle".into(),
        }
    }

    #[test]
    fn the_default_rules_lock_dim_and_sleep_on_battery() {
        let args = command(&PowerConfig::default(), &programs(), true);
        assert_eq!(
            args,
            vec![
                "/s/swayidle", "-w",
                "timeout", "300", "'/s/flux-lock' --daemon",
                "timeout", "360", "'/s/niri' msg action power-off-monitors",
                "timeout", "1200", "'/s/flux-idle' suspend-if battery",
                "before-sleep", "'/s/flux-lock' --daemon",
                "lock", "'/s/flux-lock' --daemon",
            ]
        );
    }

    #[test]
    fn never_drops_the_rule_and_no_lock_drops_every_lock() {
        let config = PowerConfig { screen_off: None, suspend_battery: None, suspend_ac: Some(3600), ..PowerConfig::default() };
        let args = command(&config, &programs(), false);
        assert_eq!(args, vec!["/s/swayidle", "-w", "timeout", "3600", "'/s/flux-idle' suspend-if ac"]);
    }

    #[test]
    fn lid_nothing_runs_under_a_lid_inhibitor() {
        let config = PowerConfig { lid: Lid::Nothing, ..PowerConfig::default() };
        let args = command(&config, &programs(), true);
        assert_eq!(args[0], "/s/systemd-inhibit");
        assert_eq!(args[1], "--what=handle-lid-switch");
        assert_eq!(args[5], "/s/swayidle");
    }

    #[test]
    fn quoting_survives_single_quotes() {
        assert_eq!(quote("/a b/it's"), r"'/a b/it'\''s'");
    }

    #[test]
    fn mains_detection() {
        let entry = |kind: &str, online: &str| (kind.to_string(), online.to_string());
        assert!(on_mains(&[]));
        assert!(on_mains(&[entry("Battery\n", "")]));
        assert!(on_mains(&[entry("Battery", ""), entry("Mains", "1\n")]));
        assert!(!on_mains(&[entry("Battery", ""), entry("Mains", "0\n")]));
        assert!(on_mains(&[entry("Mains", "0"), entry("Mains", "1")]));
    }
}
