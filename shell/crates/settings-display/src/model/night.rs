//! Night light through `wlsunset`, which niri drives over wlr-gamma-control.
//!
//! "Sunset to sunrise" needs a place on Earth. Flux takes it from the system
//! time zone's entry in `zone1970.tab`, so nothing asks for a location and
//! nothing leaves the machine.

use super::config::{NightLight, Schedule};

/// The warmest and coolest temperatures the slider offers, in kelvin.
pub const WARMEST: u32 = 2500;
pub const COOLEST: u32 = 6000;
/// Daylight, what the screen returns to in the morning.
pub const DAY: u32 = 6500;

/// The variable the user unit expands into `wlsunset`'s arguments.
pub const ENV_KEY: &str = "FLUX_NIGHT_LIGHT_ARGS";

/// The schedule used when "sunset to sunrise" has no coordinates.
pub const FALLBACK_HOURS: ((u8, u8), (u8, u8)) = ((20, 0), (7, 0));

/// Parse an ISO 6709 coordinate pair from `zone1970.tab`, like
/// `+4852+00220` or `-332508+1513000`, into decimal latitude and longitude.
pub fn parse_iso6709(text: &str) -> Option<(f64, f64)> {
    let split = text.get(1..)?.find(['+', '-'])? + 1;
    let (lat, lon) = text.split_at(split);
    Some((degrees(lat, 2)?, degrees(lon, 3)?))
}

fn degrees(text: &str, width: usize) -> Option<f64> {
    let (sign, digits) = text.split_at(1);
    let sign = match sign {
        "+" => 1.0,
        "-" => -1.0,
        _ => return None,
    };
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let part = |range: std::ops::Range<usize>| digits.get(range).map(|s| s.parse::<f64>().ok()).unwrap_or(Some(0.0));
    let whole = digits.get(0..width)?.parse::<f64>().ok()?;
    let minutes = part(width..width + 2)?;
    let seconds = part(width + 2..width + 4)?;
    if digits.len() != width + 2 && digits.len() != width + 4 {
        return None;
    }
    Some(sign * (whole + minutes / 60.0 + seconds / 3600.0))
}

/// The coordinates of `zone` in a `zone1970.tab` or `zone.tab` document.
pub fn zone_coordinates(tab: &str, zone: &str) -> Option<(f64, f64)> {
    tab.lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .find(|fields| fields.get(2).map(|name| name.trim()) == Some(zone))
        .and_then(|fields| parse_iso6709(fields.get(1)?.trim()))
}

/// The zone name from the target of `/etc/localtime`, such as
/// `/etc/zoneinfo/Europe/Paris`.
pub fn zone_from_link(target: &str) -> Option<String> {
    let index = target.find("zoneinfo/")? + "zoneinfo/".len();
    let zone = target[index..].trim_matches('/');
    (!zone.is_empty()).then(|| zone.to_string())
}

/// "20:00".
pub fn format_time((hour, minute): (u8, u8)) -> String {
    format!("{:02}:{:02}", hour.min(23), minute.min(59))
}

/// Parse "7:30" or "07:30".
pub fn parse_time(text: &str) -> Option<(u8, u8)> {
    let (hour, minute) = text.trim().split_once(':')?;
    let hour: u8 = hour.trim().parse().ok()?;
    let minute: u8 = minute.trim().parse().ok()?;
    (hour < 24 && minute < 60).then_some((hour, minute))
}

/// `wlsunset`'s arguments, or `None` when night light is off.
pub fn wlsunset_args(night: &NightLight, place: Option<(f64, f64)>) -> Option<Vec<String>> {
    if !night.enabled {
        return None;
    }
    let low = night.temperature.clamp(WARMEST, COOLEST);
    let mut args = vec!["-t".to_string(), low.to_string(), "-T".to_string(), DAY.to_string()];
    let manual = match (night.schedule, place) {
        (Schedule::Sun, Some((lat, lon))) => {
            args.extend(["-l".to_string(), format!("{lat:.2}"), "-L".to_string(), format!("{lon:.2}")]);
            None
        }
        (Schedule::Manual { from, to }, _) => Some((from, to)),
        (Schedule::Sun, None) => Some(FALLBACK_HOURS),
    };
    if let Some((from, to)) = manual {
        args.extend(["-S".to_string(), format_time(to), "-s".to_string(), format_time(from)]);
    }
    Some(args)
}

/// The environment file the user unit reads.
pub fn env_file(args: &[String]) -> String {
    format!("{ENV_KEY}={}\n", args.join(" "))
}
