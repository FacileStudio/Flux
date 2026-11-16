//! What the About page says about this machine, read from `/etc/os-release`,
//! `/proc` and `/sys`. The parsers take text so they are tested on fixtures.

use std::collections::HashMap;
use std::path::Path;

/// Everything the About page lists. `None` reads as "Unknown".
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Details {
    pub system: Option<String>,
    pub version: Option<String>,
    pub hostname: Option<String>,
    pub processor: Option<String>,
    pub memory: Option<u64>,
    pub disk: Option<u64>,
    pub graphics: Option<String>,
    pub kernel: Option<String>,
}

/// The `KEY=value` pairs of an os-release file, quotes and escapes removed.
pub fn parse_os_release(text: &str) -> HashMap<String, String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim().to_string(), unquote(value.trim())))
        .collect()
}

fn unquote(value: &str) -> String {
    let inner = ['"', '\'']
        .iter()
        .find_map(|quote| {
            value
                .strip_prefix(*quote)
                .and_then(|rest| rest.strip_suffix(*quote))
        })
        .unwrap_or(value);
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(letter) = chars.next() {
        if letter == '\\' {
            out.extend(chars.next());
        } else {
            out.push(letter);
        }
    }
    out
}

/// The processor's name and how many threads it runs, e.g. `AMD Ryzen 7 7840U × 16`.
pub fn processor(cpuinfo: &str) -> Option<String> {
    let field = |names: &[&str]| {
        cpuinfo.lines().find_map(|line| {
            let (key, value) = line.split_once(':')?;
            (names.contains(&key.trim()) && !value.trim().is_empty())
                .then(|| value.trim().to_string())
        })
    };
    let threads = cpuinfo
        .lines()
        .filter(|line| {
            line.split_once(':')
                .is_some_and(|(key, _)| key.trim() == "processor")
        })
        .count();
    let name = field(&["model name", "Model", "Hardware", "cpu model"])?;
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    Some(if threads > 1 {
        format!("{name} × {threads}")
    } else {
        name
    })
}

/// `MemTotal` in bytes.
pub fn memory(meminfo: &str) -> Option<u64> {
    meminfo.lines().find_map(|line| {
        let rest = line.strip_prefix("MemTotal:")?;
        let kib: u64 = rest.trim().trim_end_matches("kB").trim().parse().ok()?;
        Some(kib * 1024)
    })
}

/// Whether a `/sys/block` device is a real, fixed disk: no loop, RAM, zram, mapper, RAID or optical device.
pub fn is_disk(name: &str, removable: bool) -> bool {
    const VIRTUAL: [&str; 7] = ["loop", "ram", "zram", "dm-", "md", "sr", "fd"];
    !removable && !VIRTUAL.iter().any(|prefix| name.starts_with(prefix))
}

/// The total size of the fixed disks in `/sys/block`. Each holds its size in 512-byte sectors.
fn disk(root: &Path) -> Option<u64> {
    let total: u64 = std::fs::read_dir(root.join("sys/block"))
        .ok()?
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            let removable = read(&path.join("removable")).is_some_and(|value| value.trim() == "1");
            if !is_disk(&name, removable) {
                return None;
            }
            read(&path.join("size"))?.trim().parse::<u64>().ok()
        })
        .sum();
    (total > 0).then_some(total * 512)
}

/// The vendor and device names for `vendor:device` (hex) in a `pci.ids` database.
pub fn pci_name(database: &str, vendor: &str, device: &str) -> Option<String> {
    let vendor = format!("{} ", vendor.to_ascii_lowercase());
    let device = format!("{} ", device.to_ascii_lowercase());
    let mut lines = database
        .lines()
        .skip_while(|line| !line.starts_with(&vendor));
    let vendor_name = lines.next()?.get(4..)?.trim().to_string();
    let device_name = lines
        .take_while(|line| line.starts_with('\t') || line.starts_with('#'))
        .find_map(|line| {
            line.strip_prefix('\t')?
                .strip_prefix(&device)
                .map(|name| name.trim().to_string())
        });
    Some(match device_name {
        Some(device_name) => format!("{vendor_name} {device_name}"),
        None => vendor_name,
    })
}

/// A readable name for a DRM driver, for when no `pci.ids` names the card.
pub fn driver_name(driver: &str) -> String {
    match driver {
        "i915" | "xe" => "Intel Graphics".to_string(),
        "amdgpu" | "radeon" => "AMD Radeon Graphics".to_string(),
        "nouveau" | "nvidia" => "NVIDIA Graphics".to_string(),
        "virtio-pci" | "virtio_gpu" | "virtio-gpu" => "Virtio GPU".to_string(),
        "bochs-drm" | "bochs" => "QEMU Standard VGA".to_string(),
        "vmwgfx" => "VMware SVGA".to_string(),
        "qxl" => "QXL".to_string(),
        "apple-dcp" | "asahi" => "Apple GPU".to_string(),
        other => other.to_string(),
    }
}

/// The `DRIVER` and `PCI_ID` (`VVVV:DDDD`) lines of a device `uevent`.
pub fn uevent(text: &str) -> (Option<String>, Option<(String, String)>) {
    let value = |key: &str| {
        text.lines().find_map(|line| {
            line.strip_prefix(key)?
                .strip_prefix('=')
                .map(str::to_string)
        })
    };
    let id = value("PCI_ID").and_then(|id| {
        id.split_once(':')
            .map(|(vendor, device)| (vendor.to_string(), device.to_string()))
    });
    (value("DRIVER"), id)
}

const PCI_IDS: [&str; 3] = [
    "run/current-system/sw/share/hwdata/pci.ids",
    "usr/share/hwdata/pci.ids",
    "usr/share/misc/pci.ids",
];

/// The first graphics card under `/sys/class/drm`, named from `pci.ids` or its driver.
fn graphics(root: &Path) -> Option<String> {
    let mut cards: Vec<_> = std::fs::read_dir(root.join("sys/class/drm"))
        .ok()?
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with("card") && !name.contains('-'))
        .collect();
    cards.sort();
    let card = cards.first()?;
    let (driver, id) = uevent(&read(
        &root.join("sys/class/drm").join(card).join("device/uevent"),
    )?);
    let named = id.and_then(|(vendor, device)| {
        PCI_IDS
            .iter()
            .find_map(|path| pci_name(&read(&root.join(path))?, &vendor, &device))
    });
    named.or_else(|| driver.map(|driver| driver_name(&driver)))
}

fn read(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// Read every detail from the system mounted at `root` (`/` in a real run). Blocking.
pub fn gather(root: &Path) -> Details {
    let release = parse_os_release(&read(&root.join("etc/os-release")).unwrap_or_default());
    let field = |key: &str| release.get(key).filter(|value| !value.is_empty()).cloned();
    let line = |path: &str| {
        read(&root.join(path))
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
    };
    Details {
        system: field("PRETTY_NAME").or_else(|| field("NAME")),
        version: field("VERSION")
            .or_else(|| field("VERSION_ID"))
            .or_else(|| field("BUILD_ID")),
        hostname: line("proc/sys/kernel/hostname"),
        processor: read(&root.join("proc/cpuinfo"))
            .as_deref()
            .and_then(processor),
        memory: read(&root.join("proc/meminfo")).as_deref().and_then(memory),
        disk: disk(root),
        graphics: graphics(root),
        kernel: line("proc/sys/kernel/osrelease"),
    }
}

/// `bytes` in the largest binary unit that keeps a number above one, e.g. `15.5 GiB`.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests;
