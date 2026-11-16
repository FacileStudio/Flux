use super::*;

const OS_RELEASE: &str = r#"ANSI_COLOR="0;38;2;126;186;228"
BUILD_ID="25.11.20261115.abcdef"
ID=flux
NAME="Flux"
PRETTY_NAME="Flux 0.11.1 (Ondée)"
VERSION="0.11.1 (Ondée)"
VERSION_ID="0.11.1"
HOME_URL="https://flux.facile.studio"
# a comment
QUOTED='single \"escaped\"'
"#;

const CPUINFO_X86: &str = "processor\t: 0\nvendor_id\t: AuthenticAMD\nmodel name\t: AMD Ryzen 7 7840U  w/ Radeon  780M Graphics\n\nprocessor\t: 1\nmodel name\t: AMD Ryzen 7 7840U  w/ Radeon  780M Graphics\n";

const CPUINFO_ARM: &str = "processor\t: 0\nBogoMIPS\t: 48.00\nCPU part\t: 0xd0b\n\nprocessor\t: 1\nBogoMIPS\t: 48.00\n\nModel\t\t: Raspberry Pi 5 Model B Rev 1.0\n";

const MEMINFO: &str = "MemTotal:       16243380 kB\nMemFree:         1234567 kB\n";

const PCI_IDS: &str = "# pci.ids fixture\n1002  Advanced Micro Devices, Inc. [AMD/ATI]\n\t15bf  Phoenix1\n\t\t1002 15bf  Phoenix1 subsystem\n1af4  Red Hat, Inc.\n\t1050  Virtio 1.0 GPU\n8086  Intel Corporation\n\t46a6  Alder Lake-P GT2 [Iris Xe Graphics]\n";

#[test]
fn os_release_fields_are_unquoted() {
    let release = parse_os_release(OS_RELEASE);
    assert_eq!(release["PRETTY_NAME"], "Flux 0.11.1 (Ondée)");
    assert_eq!(release["ID"], "flux");
    assert_eq!(release["VERSION_ID"], "0.11.1");
    assert_eq!(release["QUOTED"], "single \"escaped\"");
    assert!(!release.contains_key("# a comment"));
}

#[test]
fn processors_are_named_with_their_thread_count() {
    assert_eq!(
        processor(CPUINFO_X86).as_deref(),
        Some("AMD Ryzen 7 7840U w/ Radeon 780M Graphics × 2")
    );
    assert_eq!(
        processor(CPUINFO_ARM).as_deref(),
        Some("Raspberry Pi 5 Model B Rev 1.0 × 2")
    );
    assert_eq!(processor("processor : 0\n"), None);
}

#[test]
fn memory_is_read_in_bytes() {
    assert_eq!(memory(MEMINFO), Some(16_243_380 * 1024));
    assert_eq!(memory("MemFree: 12 kB\n"), None);
}

#[test]
fn only_fixed_disks_count() {
    assert!(is_disk("nvme0n1", false));
    assert!(is_disk("sda", false));
    assert!(is_disk("vda", false));
    assert!(!is_disk("sdb", true));
    assert!(!is_disk("loop0", false));
    assert!(!is_disk("zram0", false));
    assert!(!is_disk("dm-1", false));
    assert!(!is_disk("sr0", false));
}

#[test]
fn graphics_cards_are_named_from_pci_ids() {
    assert_eq!(
        pci_name(PCI_IDS, "1002", "15BF").as_deref(),
        Some("Advanced Micro Devices, Inc. [AMD/ATI] Phoenix1")
    );
    assert_eq!(
        pci_name(PCI_IDS, "8086", "46a6").as_deref(),
        Some("Intel Corporation Alder Lake-P GT2 [Iris Xe Graphics]")
    );
    assert_eq!(
        pci_name(PCI_IDS, "1af4", "ffff").as_deref(),
        Some("Red Hat, Inc.")
    );
    assert_eq!(pci_name(PCI_IDS, "10de", "2484"), None);
}

#[test]
fn uevents_give_the_driver_and_pci_id() {
    let (driver, id) =
        uevent("DRIVER=amdgpu\nPCI_CLASS=30000\nPCI_ID=1002:15BF\nPCI_SLOT_NAME=0000:c1:00.0\n");
    assert_eq!(driver.as_deref(), Some("amdgpu"));
    assert_eq!(id, Some(("1002".to_string(), "15BF".to_string())));
    assert_eq!(driver_name("i915"), "Intel Graphics");
    assert_eq!(driver_name("simpledrm"), "simpledrm");
}

#[test]
fn a_fake_root_is_read_end_to_end() {
    let root = std::env::temp_dir().join(format!("flux-settings-about-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let write = |path: &str, text: &str| {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("fixture dir");
        std::fs::write(path, text).expect("fixture file");
    };
    write("etc/os-release", OS_RELEASE);
    write("proc/cpuinfo", CPUINFO_X86);
    write("proc/meminfo", MEMINFO);
    write("proc/sys/kernel/hostname", "ada-flux\n");
    write("proc/sys/kernel/osrelease", "6.12.63\n");
    write("sys/block/nvme0n1/size", "1000215216\n");
    write("sys/block/nvme0n1/removable", "0\n");
    write("sys/block/loop0/size", "4096\n");
    write("sys/block/sdb/size", "60000000\n");
    write("sys/block/sdb/removable", "1\n");
    write(
        "sys/class/drm/card1/device/uevent",
        "DRIVER=amdgpu\nPCI_ID=1002:15BF\n",
    );
    write("sys/class/drm/card1-eDP-1/status", "connected\n");
    write("usr/share/hwdata/pci.ids", PCI_IDS);
    let details = gather(&root);
    assert_eq!(details.system.as_deref(), Some("Flux 0.11.1 (Ondée)"));
    assert_eq!(details.version.as_deref(), Some("0.11.1 (Ondée)"));
    assert_eq!(details.hostname.as_deref(), Some("ada-flux"));
    assert_eq!(details.kernel.as_deref(), Some("6.12.63"));
    assert_eq!(details.memory, Some(16_243_380 * 1024));
    assert_eq!(details.disk, Some(1_000_215_216 * 512));
    assert_eq!(
        details.graphics.as_deref(),
        Some("Advanced Micro Devices, Inc. [AMD/ATI] Phoenix1")
    );
    assert_eq!(gather(&root.join("missing")), Details::default());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn sizes_read_in_binary_units() {
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(16_243_380 * 1024), "15.5 GiB");
    assert_eq!(format_bytes(1_000_215_216 * 512), "476.9 GiB");
}
