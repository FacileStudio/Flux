//! Everything the installer decides without a display: languages, layouts,
//! disks, names, time zones and the backend's progress lines.

use serde_json::Value;

/// A language the installer offers, with the defaults it implies.
pub struct Language {
    pub code: &'static str,
    pub name: &'static str,
    pub locale: &'static str,
    pub keymap: &'static str,
    pub timezone: &'static str,
}

pub const LANGUAGES: &[Language] = &[
    Language { code: "en", name: "English", locale: "en_US.UTF-8", keymap: "us", timezone: "UTC" },
    Language { code: "fr", name: "Français", locale: "fr_FR.UTF-8", keymap: "fr", timezone: "Europe/Paris" },
    Language { code: "es", name: "Español", locale: "es_ES.UTF-8", keymap: "es", timezone: "Europe/Madrid" },
    Language { code: "de", name: "Deutsch", locale: "de_DE.UTF-8", keymap: "de", timezone: "Europe/Berlin" },
    Language { code: "en", name: "English (UK)", locale: "en_GB.UTF-8", keymap: "gb", timezone: "Europe/London" },
    Language { code: "fr", name: "Français (Belgique)", locale: "fr_BE.UTF-8", keymap: "be", timezone: "Europe/Brussels" },
    Language { code: "fr", name: "Français (Suisse)", locale: "fr_CH.UTF-8", keymap: "ch", timezone: "Europe/Zurich" },
    Language { code: "fr", name: "Français (Canada)", locale: "fr_CA.UTF-8", keymap: "ca", timezone: "America/Toronto" },
    Language { code: "de", name: "Deutsch (Schweiz)", locale: "de_CH.UTF-8", keymap: "ch", timezone: "Europe/Zurich" },
    Language { code: "it", name: "Italiano", locale: "it_IT.UTF-8", keymap: "it", timezone: "Europe/Rome" },
    Language { code: "pt", name: "Português", locale: "pt_PT.UTF-8", keymap: "pt", timezone: "Europe/Lisbon" },
    Language { code: "pt", name: "Português (Brasil)", locale: "pt_BR.UTF-8", keymap: "br", timezone: "America/Sao_Paulo" },
    Language { code: "nl", name: "Nederlands", locale: "nl_NL.UTF-8", keymap: "us", timezone: "Europe/Amsterdam" },
];

/// The xkb layouts `flux-install --keymap` accepts, with a readable name.
pub const KEYMAPS: &[(&str, &str)] = &[
    ("us", "English (US)"),
    ("gb", "English (UK)"),
    ("fr", "Français (AZERTY)"),
    ("de", "Deutsch (QWERTZ)"),
    ("es", "Español"),
    ("it", "Italiano"),
    ("pt", "Português"),
    ("br", "Português (Brasil)"),
    ("be", "Belgique"),
    ("ch", "Suisse / Schweiz"),
    ("ca", "Canada (français)"),
    ("nl", "Nederlands"),
    ("se", "Svenska"),
    ("no", "Norsk"),
    ("dk", "Dansk"),
    ("fi", "Suomi"),
    ("pl", "Polski"),
    ("cz", "Čeština"),
    ("latam", "Español (Latinoamérica)"),
];

/// The readable name of an xkb layout code.
pub fn keymap_name(code: &str) -> &'static str {
    KEYMAPS
        .iter()
        .find(|(id, _)| *id == code)
        .map(|(_, name)| *name)
        .unwrap_or("English (US)")
}

/// A whole disk as `lsblk -J -b -d` reports it.
#[derive(Clone, Debug, PartialEq)]
pub struct Disk {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub model: String,
    pub removable: bool,
    pub transport: String,
    pub live: bool,
}

impl Disk {
    /// How the disk is named to a person: its model and size, then its path.
    pub fn label(&self) -> String {
        let model = if self.model.is_empty() { self.name.as_str() } else { self.model.as_str() };
        format!("{model}, {} ({})", human_size(self.size), self.path)
    }
}

fn text(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(s)) => s.trim().to_string(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

fn flag(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => s == "1" || s == "true",
        Some(Value::Number(n)) => n.as_u64() == Some(1),
        _ => false,
    }
}

fn number(value: Option<&Value>) -> u64 {
    match value {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
        Some(Value::String(s)) => s.trim().parse().unwrap_or(0),
        _ => 0,
    }
}

/// The disks a person may pick from `lsblk -J -b -d -o NAME,PATH,SIZE,MODEL,TYPE,RM,TRAN`.
///
/// Partitions, loop devices, optical drives, zram and empty devices are left
/// out. A disk named in `live` (the medium Flux runs from) stays in the list,
/// flagged, so the screen can say why it cannot be picked.
pub fn parse_disks(json: &str, live: &[String]) -> Vec<Disk> {
    let Ok(root) = serde_json::from_str::<Value>(json) else {
        return Vec::new();
    };
    let Some(devices) = root.get("blockdevices").and_then(Value::as_array) else {
        return Vec::new();
    };
    devices
        .iter()
        .filter(|device| text(device.get("type")) == "disk")
        .map(|device| {
            let name = text(device.get("name"));
            let path = Some(text(device.get("path")))
                .filter(|path| !path.is_empty())
                .unwrap_or_else(|| format!("/dev/{name}"));
            Disk {
                live: live.contains(&name),
                size: number(device.get("size")),
                model: text(device.get("model")),
                removable: flag(device.get("rm")),
                transport: text(device.get("tran")),
                name,
                path,
            }
        })
        .filter(|disk| disk.size > 0 && !disk.name.starts_with("zram") && !disk.name.starts_with("loop"))
        .collect()
}

/// The device mounted at `/iso` in a `/proc/mounts` document, the medium the
/// live system booted from.
pub fn iso_source(mounts: &str) -> Option<String> {
    mounts.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        let source = fields.next()?;
        (fields.next()? == "/iso").then(|| source.to_string())
    })
}

/// A size in bytes the way disk makers print it: decimal units.
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }
    if value >= 100.0 || unit == 0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
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

/// The username a full name suggests: the first name, folded to ASCII.
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

/// The computer name a username suggests, such as `ada-flux`.
pub fn derive_hostname(username: &str) -> String {
    let base: String = username
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let base = base.trim_matches('-');
    if base.is_empty() {
        return "flux".to_string();
    }
    let mut name = format!("{base}-flux");
    name.truncate(63);
    name.trim_end_matches('-').to_string()
}

/// The names the system keeps for itself, shared with `flux-install` and `flux-config` through `rules.json`.
fn reserved_users() -> &'static [String] {
    static NAMES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    NAMES.get_or_init(|| {
        serde_json::from_str::<Value>(include_str!("../../flux-config/rules.json"))
            .ok()
            .and_then(|rules| rules.get("reservedUsers").and_then(Value::as_array).cloned())
            .map(|names| names.iter().filter_map(Value::as_str).map(str::to_string).collect())
            .unwrap_or_else(|| vec!["root".to_string()])
    })
}

/// Whether `name` matches `^[a-z_][a-z0-9_-]{0,31}$` and is not a name the system keeps.
pub fn valid_username(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    name.len() <= 32
        && !reserved_users().iter().any(|reserved| reserved == name)
        && !name.starts_with("systemd-")
        && !name.starts_with("nixbld")
        && (first.is_ascii_lowercase() || first == '_')
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// Whether `name` is an RFC 1123 host label.
pub fn valid_hostname(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 63
        && !name.starts_with('-')
        && !name.ends_with('-')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// Whether the backend can store `name` as the account's full name.
pub fn valid_full_name(name: &str) -> bool {
    !name.trim().is_empty()
        && name.chars().count() <= 64
        && !name.chars().any(|c| c.is_control() || "\"\\$|&`".contains(c))
}

/// The canonical zones of a `zone1970.tab` or `zone.tab` document, sorted, with `UTC` first.
pub fn parse_zone_tab(tab: &str) -> Vec<String> {
    let mut zones: Vec<String> = tab
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split('\t').nth(2))
        .map(|zone| zone.trim().to_string())
        .filter(|zone| !zone.is_empty())
        .collect();
    zones.sort();
    zones.dedup();
    zones.insert(0, "UTC".to_string());
    zones
}

/// `/etc/niri/config.kdl` with its `xkb` block set to `layout`, for trying a
/// keyboard layout in the live session.
pub fn niri_with_layout(config: &str, layout: &str) -> Option<String> {
    let start = config.find("xkb {")? + "xkb {".len();
    let end = start + config[start..].find('}')?;
    Some(format!("{}\n            layout \"{layout}\"\n        {}", &config[..start], &config[end..]))
}

/// A stage of `flux-install --progress json`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Partition,
    Format,
    Config,
    Install,
    Bootloader,
    Done,
}

/// One line of `flux-install --progress json`.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Progress { step: Step, percent: u8, message: String },
    Error { message: String, log_tail: String },
}

/// Read one progress line. Anything that is not a progress object is `None`.
pub fn parse_progress(line: &str) -> Option<Event> {
    let value: Value = serde_json::from_str(line.trim()).ok()?;
    let message = value.get("message").and_then(Value::as_str).unwrap_or("").to_string();
    let step = match value.get("step")?.as_str()? {
        "error" => {
            let log_tail = value.get("log_tail").and_then(Value::as_str).unwrap_or("").to_string();
            return Some(Event::Error { message, log_tail });
        }
        "partition" => Step::Partition,
        "format" => Step::Format,
        "config" => Step::Config,
        "install" => Step::Install,
        "bootloader" => Step::Bootloader,
        "done" => Step::Done,
        _ => return None,
    };
    let percent = value.get("percent").and_then(Value::as_u64).unwrap_or(0).min(100) as u8;
    Some(Event::Progress { step, percent, message })
}

/// What the person chose, ready for the backend.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Choices {
    pub disk: String,
    pub hostname: String,
    pub full_name: String,
    pub username: String,
    pub timezone: String,
    pub keymap: String,
    pub locale: String,
}

/// The `flux-install` arguments for `choices` and a password hash.
pub fn install_args(choices: &Choices, hash: &str) -> Vec<String> {
    [
        ("--disk", choices.disk.as_str()),
        ("--hostname", choices.hostname.as_str()),
        ("--full-name", choices.full_name.as_str()),
        ("--user", choices.username.as_str()),
        ("--password-hash", hash),
        ("--timezone", choices.timezone.as_str()),
        ("--keymap", choices.keymap.as_str()),
        ("--locale", choices.locale.as_str()),
    ]
    .iter()
    .flat_map(|(flag, value)| [flag.to_string(), value.to_string()])
    .chain(["--yes", "--progress", "json"].map(String::from))
    .collect()
}

/// The last `count` lines of `log`.
pub fn tail(log: &str, count: usize) -> String {
    let lines: Vec<&str> = log.lines().collect();
    lines[lines.len().saturating_sub(count)..].join("\n")
}

#[cfg(test)]
mod tests;
