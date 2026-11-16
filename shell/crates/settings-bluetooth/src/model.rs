//! Bluetooth state as plain data: devices, their kind, ordering, labels and the
//! pairing flow. Nothing here touches D-Bus or GTK.

#[cfg(test)]
mod tests;

/// What a device is, as far as its icon and label go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Headphones,
    Speaker,
    Keyboard,
    Mouse,
    Phone,
    Controller,
    Computer,
    Other,
}

impl Kind {
    /// The symbolic icon from the Adwaita theme.
    pub fn icon(self) -> &'static str {
        match self {
            Kind::Headphones => "audio-headphones-symbolic",
            Kind::Speaker => "audio-speakers-symbolic",
            Kind::Keyboard => "input-keyboard-symbolic",
            Kind::Mouse => "input-mouse-symbolic",
            Kind::Phone => "phone-symbolic",
            Kind::Controller => "input-gaming-symbolic",
            Kind::Computer => "computer-symbolic",
            Kind::Other => "bluetooth-active-symbolic",
        }
    }

    /// The kind BlueZ's `Icon` property names, when it names one we draw.
    pub fn from_icon(icon: &str) -> Option<Kind> {
        let kind = match icon {
            "audio-headphones" | "audio-headset" => Kind::Headphones,
            "audio-card" | "audio-speakers" | "multimedia-player" => Kind::Speaker,
            "input-keyboard" => Kind::Keyboard,
            "input-mouse" | "input-tablet" => Kind::Mouse,
            "input-gaming" => Kind::Controller,
            "phone" | "modem" => Kind::Phone,
            "computer" => Kind::Computer,
            _ => return None,
        };
        Some(kind)
    }

    /// The kind a classic Class of Device encodes: major class in bits 8-12,
    /// minor class in bits 2-7.
    pub fn from_class(class: u32) -> Option<Kind> {
        let major = (class >> 8) & 0x1f;
        let minor = (class >> 2) & 0x3f;
        match major {
            1 => Some(Kind::Computer),
            2 => Some(Kind::Phone),
            4 => match minor {
                1 | 2 | 6 => Some(Kind::Headphones),
                5 | 7 | 8 | 10 => Some(Kind::Speaker),
                _ => None,
            },
            5 => match ((minor >> 4) & 0x3, minor & 0xf) {
                (_, 1 | 2) => Some(Kind::Controller),
                (1 | 3, _) => Some(Kind::Keyboard),
                (2, _) => Some(Kind::Mouse),
                _ => None,
            },
            _ => None,
        }
    }

    /// The kind a Bluetooth LE `Appearance` value encodes.
    pub fn from_appearance(appearance: u16) -> Option<Kind> {
        match appearance {
            0x0040..=0x007f => Some(Kind::Phone),
            0x0080..=0x00bf => Some(Kind::Computer),
            0x03c1 => Some(Kind::Keyboard),
            0x03c2 | 0x03c5 => Some(Kind::Mouse),
            0x03c3 | 0x03c4 => Some(Kind::Controller),
            0x0940..=0x097f => Some(Kind::Headphones),
            0x0840..=0x087f => Some(Kind::Speaker),
            _ => None,
        }
    }
}

/// One remote device as BlueZ reports it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Device {
    pub path: String,
    pub adapter: String,
    pub address: String,
    pub address_type: String,
    pub name: Option<String>,
    pub alias: Option<String>,
    pub icon: Option<String>,
    pub class: Option<u32>,
    pub appearance: Option<u16>,
    pub paired: bool,
    pub trusted: bool,
    pub connected: bool,
    pub rssi: Option<i16>,
    pub battery: Option<u8>,
}

impl Device {
    /// The kind from the icon, then the class, then the LE appearance.
    pub fn kind(&self) -> Kind {
        self.icon
            .as_deref()
            .and_then(Kind::from_icon)
            .or_else(|| self.class.and_then(Kind::from_class))
            .or_else(|| self.appearance.and_then(Kind::from_appearance))
            .unwrap_or(Kind::Other)
    }

    /// Whether the device announced a name of its own.
    pub fn has_name(&self) -> bool {
        self.name.as_deref().is_some_and(|name| !name.trim().is_empty()) || self.real_alias().is_some()
    }

    /// The alias, unless it is BlueZ's stand-in made from the address.
    fn real_alias(&self) -> Option<&str> {
        let alias = self.alias.as_deref()?.trim();
        let standin = alias.replace('-', ":").eq_ignore_ascii_case(&self.address);
        (!alias.is_empty() && !standin).then_some(alias)
    }

    /// What to call the device: its alias, its name, or its address.
    pub fn display_name(&self) -> String {
        self.real_alias()
            .or_else(|| self.name.as_deref().map(str::trim).filter(|name| !name.is_empty()))
            .unwrap_or(&self.address)
            .to_string()
    }
}

/// The local Bluetooth controller.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Adapter {
    pub path: String,
    pub name: String,
    pub address: String,
    pub powered: bool,
    pub discoverable: bool,
    pub discovering: bool,
}

/// The adapter the page drives: the first one by object path, so `hci0`.
pub fn pick_adapter(mut adapters: Vec<Adapter>) -> Option<Adapter> {
    adapters.sort_by(|a, b| a.path.cmp(&b.path));
    adapters.into_iter().next()
}

/// One line of `/sys/class/rfkill`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RfkillEntry {
    pub kind: String,
    pub soft: bool,
    pub hard: bool,
}

/// Whether the kernel lets the Bluetooth radio transmit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Radio {
    #[default]
    Free,
    SoftBlocked,
    HardBlocked,
}

/// The radio state the rfkill switches for Bluetooth add up to.
pub fn radio(entries: &[RfkillEntry]) -> Radio {
    let bluetooth = || entries.iter().filter(|entry| entry.kind == "bluetooth");
    if bluetooth().any(|entry| entry.hard) {
        Radio::HardBlocked
    } else if bluetooth().any(|entry| entry.soft) {
        Radio::SoftBlocked
    } else {
        Radio::Free
    }
}

/// Everything the page draws, rebuilt on every change.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub service: bool,
    pub adapter: Option<Adapter>,
    pub devices: Vec<Device>,
    pub radio: Radio,
}

/// Which screen the page shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Availability {
    NoService,
    NoAdapter,
    HardBlocked,
    SoftBlocked,
    Off,
    On,
}

impl Snapshot {
    /// What the state allows: an empty state or the device lists.
    pub fn availability(&self) -> Availability {
        let Some(adapter) = self.adapter.as_ref().filter(|_| self.service) else {
            return if self.service { Availability::NoAdapter } else { Availability::NoService };
        };
        match (self.radio, adapter.powered) {
            (Radio::HardBlocked, _) => Availability::HardBlocked,
            (_, true) => Availability::On,
            (Radio::SoftBlocked, false) => Availability::SoftBlocked,
            (Radio::Free, false) => Availability::Off,
        }
    }

    fn on_adapter(&self) -> impl Iterator<Item = &Device> {
        let adapter = self.adapter.as_ref().map(|adapter| adapter.path.as_str());
        self.devices.iter().filter(move |device| Some(device.adapter.as_str()) == adapter)
    }

    /// Paired devices, connected ones first, then by name.
    pub fn paired(&self) -> Vec<&Device> {
        let mut devices: Vec<&Device> = self.on_adapter().filter(|device| device.paired).collect();
        devices.sort_by_cached_key(|device| (!device.connected, device.display_name().to_lowercase()));
        devices
    }

    /// Devices in range that are not paired, strongest signal first.
    pub fn nearby(&self, show_unnamed: bool) -> Vec<&Device> {
        let mut devices: Vec<&Device> = self
            .on_adapter()
            .filter(|device| !device.paired && (show_unnamed || device.has_name()))
            .collect();
        devices.sort_by_cached_key(|device| {
            (
                device.rssi.is_none(),
                std::cmp::Reverse(device.rssi.unwrap_or(i16::MIN)),
                device.display_name().to_lowercase(),
            )
        });
        devices
    }

    /// How many nearby devices the unnamed filter hides.
    pub fn unnamed_count(&self) -> usize {
        self.on_adapter().filter(|device| !device.paired && !device.has_name()).count()
    }

    /// The device at `path`.
    pub fn device(&self, path: &str) -> Option<&Device> {
        self.devices.iter().find(|device| device.path == path)
    }
}

/// The battery charge as text, with `pattern`'s `{n}` replaced.
pub fn battery_label(percent: u8, pattern: &str) -> String {
    pattern.replace("{n}", &percent.min(100).to_string())
}

/// The Adwaita battery icon for a charge, in steps of ten.
pub fn battery_icon(percent: u8) -> String {
    format!("battery-level-{}-symbolic", percent.min(100) / 10 * 10)
}

/// How strong a nearby device's signal is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    Strong,
    Good,
    Weak,
}

/// The signal level for an RSSI reading in dBm.
pub fn signal(rssi: i16) -> Signal {
    match rssi {
        -60.. => Signal::Strong,
        -75..=-61 => Signal::Good,
        _ => Signal::Weak,
    }
}

/// A six-digit passkey as it is read out: `012 345`.
pub fn passkey_text(passkey: u32) -> String {
    let digits = format!("{:06}", passkey.min(999_999));
    format!("{} {}", &digits[..3], &digits[3..])
}

/// A typed passkey: up to six digits, spaces ignored.
pub fn parse_passkey(text: &str) -> Option<u32> {
    let digits: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let valid = !digits.is_empty() && digits.len() <= 6 && digits.chars().all(|c| c.is_ascii_digit());
    valid.then(|| digits.parse().ok()).flatten()
}

/// Whether a legacy PIN code is one BlueZ accepts: 1 to 16 letters or digits.
pub fn valid_pin(text: &str) -> bool {
    (1..=16).contains(&text.len()) && text.chars().all(|c| c.is_ascii_alphanumeric())
}

/// What a device asks to use, from the profile UUID BlueZ passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Service {
    Audio,
    Calls,
    Input,
    Remote,
    Files,
    Network,
    Other,
}

/// The service a profile UUID belongs to.
pub fn service(uuid: &str) -> Service {
    let uuid = uuid.to_ascii_lowercase();
    let short = uuid
        .strip_suffix("-0000-1000-8000-00805f9b34fb")
        .and_then(|head| head.strip_prefix("0000"))
        .and_then(|short| u16::from_str_radix(short, 16).ok());
    match short {
        Some(0x110a | 0x110b | 0x110d) => Service::Audio,
        Some(0x1108 | 0x1112 | 0x111e | 0x111f) => Service::Calls,
        Some(0x1124 | 0x1812) => Service::Input,
        Some(0x110c | 0x110e | 0x110f) => Service::Remote,
        Some(0x1105 | 0x1106) => Service::Files,
        Some(0x1115 | 0x1116) => Service::Network,
        _ => Service::Other,
    }
}

/// Why pairing or connecting did not work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Rejected,
    Canceled,
    Timeout,
    WrongCode,
    Unreachable,
    Blocked,
    Busy,
    Other,
}

/// The failure a BlueZ D-Bus error stands for.
pub fn failure(name: &str, message: &str) -> Failure {
    let short = name.rsplit('.').next().unwrap_or(name);
    let message = message.to_ascii_lowercase();
    match short {
        "AuthenticationRejected" | "Rejected" => Failure::Rejected,
        "AuthenticationCanceled" | "Canceled" => Failure::Canceled,
        "AuthenticationTimeout" | "NoReply" | "Timeout" => Failure::Timeout,
        "AuthenticationFailed" => Failure::WrongCode,
        "ConnectionAttemptFailed" | "NotAvailable" => Failure::Unreachable,
        "InProgress" | "Busy" => Failure::Busy,
        "NotReady" => Failure::Blocked,
        _ if message.contains("rfkill") || message.contains("blocked") => Failure::Blocked,
        _ if message.contains("page timeout") || message.contains("host is down") => Failure::Unreachable,
        _ if message.contains("timeout") || message.contains("timed out") => Failure::Timeout,
        _ => Failure::Other,
    }
}

/// What the device asks the person during pairing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Prompt {
    Confirm(u32),
    Authorize,
    AuthorizeService(Service),
    RequestPin,
    RequestPasskey,
    DisplayPin(String),
    DisplayPasskey { passkey: u32, entered: u16 },
}

impl Prompt {
    /// Whether BlueZ waits for an answer, rather than just showing a code.
    pub fn needs_answer(&self) -> bool {
        !matches!(self, Prompt::DisplayPin(_) | Prompt::DisplayPasskey { .. })
    }
}

/// Where an initiated pairing is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Pairing,
    Connecting,
}

/// The pairing flow, one device at a time.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Pairing {
    #[default]
    Idle,
    Working { device: String, stage: Stage },
    Asking { device: String, prompt: Prompt, initiated: bool },
    Failed { device: String, failure: Failure },
}

/// Something that moves the pairing flow along.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    Start(String),
    Prompt(String, Prompt),
    Answered,
    Cancelled,
    Paired(String),
    Finished(String),
    Failed(String, Failure),
    Dismiss,
}

impl Pairing {
    /// The device the flow is about.
    pub fn device(&self) -> Option<&str> {
        match self {
            Pairing::Idle => None,
            Pairing::Working { device, .. } | Pairing::Asking { device, .. } | Pairing::Failed { device, .. } => {
                Some(device)
            }
        }
    }

    fn is(&self, path: &str) -> bool {
        self.device() == Some(path)
    }

    /// The state after `step`. Steps about another device than the current one
    /// leave the state as it is.
    pub fn next(self, step: Step) -> Pairing {
        match step {
            Step::Start(device) if matches!(self, Pairing::Idle | Pairing::Failed { .. }) => Pairing::Working {
                device,
                stage: Stage::Pairing,
            },
            Step::Prompt(device, prompt) => self.prompt(device, prompt),
            Step::Answered | Step::Cancelled => match self {
                Pairing::Asking { device, initiated: true, .. } => Pairing::Working {
                    device,
                    stage: Stage::Pairing,
                },
                Pairing::Asking { initiated: false, .. } => Pairing::Idle,
                other => other,
            },
            Step::Paired(device) if self.is(&device) => match self {
                Pairing::Asking { initiated: false, .. } => Pairing::Idle,
                _ => Pairing::Working {
                    device,
                    stage: Stage::Connecting,
                },
            },
            Step::Finished(device) if self.is(&device) => Pairing::Idle,
            Step::Failed(device, Failure::Canceled) if self.is(&device) => Pairing::Idle,
            Step::Failed(device, failure) if self.is(&device) => Pairing::Failed { device, failure },
            Step::Dismiss => Pairing::Idle,
            _ => self,
        }
    }

    fn prompt(self, device: String, prompt: Prompt) -> Pairing {
        match self {
            Pairing::Idle | Pairing::Failed { .. } => Pairing::Asking {
                device,
                prompt,
                initiated: false,
            },
            Pairing::Working { device: current, .. } if current == device => Pairing::Asking {
                device,
                prompt,
                initiated: true,
            },
            Pairing::Asking {
                device: current,
                initiated,
                ..
            } if current == device => Pairing::Asking {
                device,
                prompt,
                initiated,
            },
            other => other,
        }
    }
}
