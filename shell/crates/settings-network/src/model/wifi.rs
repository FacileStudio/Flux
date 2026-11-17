//! The rules of the Wi-Fi list, apart from any I/O.

use std::collections::{HashMap, HashSet};

use super::{AccessPoint, Network, Security};

const AP_PRIVACY: u32 = 0x1;
const KEY_PSK: u32 = 0x100;
const KEY_8021X: u32 = 0x200;
const KEY_SAE: u32 = 0x400;
const KEY_OWE: u32 = 0x800;
const KEY_OWE_TM: u32 = 0x1000;
const KEY_SUITE_B: u32 = 0x2000;

/// The kind of protection, from `Flags`, `WpaFlags` and `RsnFlags`.
///
/// A WPA2/WPA3 transition network reads as WPA2 personal, which both kinds of
/// client can join with a PSK.
pub fn security(flags: u32, wpa: u32, rsn: u32) -> Security {
    let both = wpa | rsn;
    if both & (KEY_8021X | KEY_SUITE_B) != 0 {
        Security::Enterprise
    } else if both & KEY_PSK != 0 {
        Security::Personal
    } else if both & KEY_SAE != 0 {
        Security::Wpa3
    } else if both & (KEY_OWE | KEY_OWE_TM) != 0 {
        Security::Owe
    } else if flags & AP_PRIVACY != 0 {
        Security::Wep
    } else {
        Security::Open
    }
}

/// Signal strength in percent to 0 to 4 bars.
pub fn bars(strength: u8) -> u8 {
    match strength {
        81.. => 4,
        56..=80 => 3,
        31..=55 => 2,
        6..=30 => 1,
        _ => 0,
    }
}

/// One network per SSID, keeping its strongest access point, hidden ones dropped.
///
/// The active access point always represents its SSID, even when another
/// radio of the same network is louder, so the row that says "Connected" is
/// the one the adapter is on.
pub fn networks(
    aps: &[AccessPoint],
    active_ap: Option<&str>,
    known: &HashSet<String>,
) -> Vec<Network> {
    let mut best: HashMap<&str, &AccessPoint> = HashMap::new();
    for ap in aps.iter().filter(|ap| !ap.ssid.is_empty()) {
        let entry = best.entry(ap.ssid.as_str()).or_insert(ap);
        let keep_current = Some(entry.path.as_str()) == active_ap;
        if !keep_current && (Some(ap.path.as_str()) == active_ap || ap.strength > entry.strength) {
            *entry = ap;
        }
    }
    let mut list: Vec<Network> = best
        .into_values()
        .map(|ap| Network {
            ssid: ap.ssid.clone(),
            ap_path: ap.path.clone(),
            strength: ap.strength,
            security: ap.security,
            frequency: ap.frequency,
            known: known.contains(&ap.ssid),
            active: Some(ap.path.as_str()) == active_ap,
        })
        .collect();
    sort(&mut list);
    list
}

/// Connected first, then known networks, then by signal, then by name.
pub fn sort(list: &mut [Network]) {
    list.sort_by(|a, b| {
        b.active
            .cmp(&a.active)
            .then(b.known.cmp(&a.known))
            .then(bars(b.strength).cmp(&bars(a.strength)))
            .then(b.strength.cmp(&a.strength))
            .then_with(|| a.ssid.to_lowercase().cmp(&b.ssid.to_lowercase()))
    });
}

/// Why a password cannot be sent yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasswordProblem {
    Empty,
    TooShort,
    TooLong,
    NotAscii,
}

/// Check a password before NetworkManager sees it.
///
/// WPA2 personal takes 8 to 63 printable ASCII characters or exactly 64
/// hexadecimal digits. WPA3 (SAE) takes any non-empty password.
pub fn check_password(security: Security, password: &str) -> Result<(), PasswordProblem> {
    if password.is_empty() {
        return Err(PasswordProblem::Empty);
    }
    if security != Security::Personal {
        return Ok(());
    }
    let length = password.chars().count();
    if length == 64 {
        return if password.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(())
        } else {
            Err(PasswordProblem::TooLong)
        };
    }
    if !password
        .chars()
        .all(|c| c.is_ascii() && !c.is_ascii_control())
    {
        return Err(PasswordProblem::NotAscii);
    }
    match length {
        0..=7 => Err(PasswordProblem::TooShort),
        8..=63 => Ok(()),
        _ => Err(PasswordProblem::TooLong),
    }
}

/// Why joining a network failed, from the `NMDeviceStateReason` of the failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    WrongPassword,
    Timeout,
    NotFound,
    NoAddress,
    Other(u32),
}

/// The failure a device `StateChanged` reason stands for.
pub fn failure(reason: u32) -> Failure {
    match reason {
        7..=10 => Failure::WrongPassword,
        11 => Failure::Timeout,
        53 => Failure::NotFound,
        5 | 6 => Failure::NoAddress,
        other => Failure::Other(other),
    }
}

/// The band a frequency in MHz belongs to.
pub fn band(frequency: u32) -> &'static str {
    match frequency {
        5925.. => "6 GHz",
        4900..=5924 => "5 GHz",
        1..=4899 => "2.4 GHz",
        0 => "",
    }
}
