use super::*;

const LSBLK: &str = r#"{
   "blockdevices": [
      {"name":"nvme0n1", "path":"/dev/nvme0n1", "size":512110190592, "model":"Samsung SSD 980 PRO 512GB", "type":"disk", "rm":false, "tran":"nvme"},
      {"name":"sda", "path":"/dev/sda", "size":31914983424, "model":"SanDisk Ultra", "type":"disk", "rm":true, "tran":"usb"},
      {"name":"sda1", "path":"/dev/sda1", "size":2862612480, "model":null, "type":"part", "rm":true, "tran":null},
      {"name":"sr0", "path":"/dev/sr0", "size":1073741312, "model":"QEMU DVD-ROM", "type":"rom", "rm":true, "tran":"sata"},
      {"name":"loop0", "path":"/dev/loop0", "size":2500000000, "model":null, "type":"loop", "rm":false, "tran":null},
      {"name":"zram0", "path":"/dev/zram0", "size":8000000000, "model":null, "type":"disk", "rm":false, "tran":null},
      {"name":"sdb", "path":"/dev/sdb", "size":0, "model":"Card Reader", "type":"disk", "rm":true, "tran":"usb"}
   ]
}"#;

const LSBLK_OLD: &str = r#"{"blockdevices": [
   {"name":"vda", "size":"21474836480", "model":null, "type":"disk", "rm":"0", "tran":null}
]}"#;

#[test]
fn disks_keep_whole_disks_only() {
    let disks = parse_disks(LSBLK, &[]);
    let names: Vec<&str> = disks.iter().map(|disk| disk.name.as_str()).collect();
    assert_eq!(names, ["nvme0n1", "sda"]);
    assert!(!disks[0].removable);
    assert!(disks[1].removable);
    assert_eq!(disks[1].transport, "usb");
    assert!(disks.iter().all(|disk| !disk.live));
}

#[test]
fn the_live_medium_is_flagged() {
    let disks = parse_disks(LSBLK, &["sda".to_string()]);
    assert!(!disks[0].live);
    assert!(disks[1].live);
}

#[test]
fn old_lsblk_strings_still_parse() {
    let disks = parse_disks(LSBLK_OLD, &[]);
    assert_eq!(disks.len(), 1);
    assert_eq!(disks[0].path, "/dev/vda");
    assert_eq!(disks[0].size, 21474836480);
    assert!(!disks[0].removable);
    assert_eq!(disks[0].label(), "vda, 21.5 GB (/dev/vda)");
}

#[test]
fn broken_lsblk_output_gives_no_disks() {
    assert!(parse_disks("", &[]).is_empty());
    assert!(parse_disks("{}", &[]).is_empty());
}

#[test]
fn the_iso_mount_names_the_boot_medium() {
    let mounts = "proc /proc proc rw 0 0\n/dev/sda1 /iso iso9660 ro 0 0\ntmpfs / tmpfs rw 0 0\n";
    assert_eq!(iso_source(mounts).as_deref(), Some("/dev/sda1"));
    assert_eq!(iso_source("tmpfs / tmpfs rw 0 0\n"), None);
}

#[test]
fn sizes_read_like_a_disk_box() {
    assert_eq!(human_size(512110190592), "512 GB");
    assert_eq!(human_size(31914983424), "31.9 GB");
    assert_eq!(human_size(2000398934016), "2.0 TB");
    assert_eq!(human_size(512), "512 B");
}

#[test]
fn usernames_come_from_the_first_name() {
    assert_eq!(derive_username("Ada Lovelace"), "ada");
    assert_eq!(derive_username("  Élodie   Durand "), "elodie");
    assert_eq!(derive_username("Jean-Noël Œuvray"), "jean-noel");
    assert_eq!(derive_username("Søren"), "soren");
    assert_eq!(derive_username("2Pac"), "u2pac");
    assert_eq!(derive_username("李"), "");
    assert_eq!(derive_username(""), "");
    assert_eq!(derive_username(&"a".repeat(40)).len(), 32);
    assert!(valid_username(&derive_username("Ada Lovelace")));
}

#[test]
fn hostnames_come_from_the_username() {
    assert_eq!(derive_hostname("ada"), "ada-flux");
    assert_eq!(derive_hostname("_ada_"), "ada-flux");
    assert_eq!(derive_hostname(""), "flux");
    assert_eq!(derive_hostname(&"a".repeat(70)).len(), 63);
    assert!(valid_hostname(&derive_hostname("jean_noel")));
}

#[test]
fn names_are_validated_like_the_backend() {
    assert!(valid_username("ada"));
    assert!(valid_username("_svc-1"));
    assert!(!valid_username("wheel"));
    assert!(!valid_username("networkmanager"));
    assert!(!valid_username("systemd-oom"));
    assert!(!valid_username("Ada"));
    assert!(!valid_username("1ada"));
    assert!(!valid_username("root"));
    assert!(!valid_username(&"a".repeat(33)));
    assert!(valid_hostname("flux"));
    assert!(!valid_hostname("-flux"));
    assert!(!valid_hostname("flux.local"));
    assert!(!valid_hostname(&"a".repeat(64)));
    assert!(valid_full_name("Ada O'Brien-Lovelace"));
    assert!(!valid_full_name("Ada \"the\" Lovelace"));
    assert!(!valid_full_name("Ada $HOME"));
    assert!(!valid_full_name("   "));
}

#[test]
fn zones_are_sorted_with_utc_first() {
    let tab = "# comment\nFR\t+4852+00220\tEurope/Paris\nDE,DK\t+5230+01322\tEurope/Berlin\tmost of Germany\nFR\t+4852+00220\tEurope/Paris\n";
    assert_eq!(parse_zone_tab(tab), ["UTC", "Europe/Berlin", "Europe/Paris"]);
}

#[test]
fn niri_layout_replaces_the_xkb_block() {
    let config = "input {\n    keyboard {\n        xkb {\n        }\n    }\n}\n";
    let result = niri_with_layout(config, "fr").expect("xkb block");
    assert!(result.contains("xkb {\n            layout \"fr\"\n        }\n    }"));
    let again = niri_with_layout(&result, "de").expect("xkb block");
    assert!(again.contains("layout \"de\""));
    assert!(!again.contains("layout \"fr\""));
    assert_eq!(niri_with_layout("input {}", "fr"), None);
}

#[test]
fn progress_lines_parse() {
    assert_eq!(
        parse_progress(r#"{"step":"install","percent":42,"message":"Installing Flux"}"#),
        Some(Event::Progress { step: Step::Install, percent: 42, message: "Installing Flux".into() })
    );
    assert_eq!(
        parse_progress(r#"{"step":"done","percent":100,"message":"ok"}"#),
        Some(Event::Progress { step: Step::Done, percent: 100, message: "ok".into() })
    );
    assert_eq!(
        parse_progress(r#"{"step":"error","message":"the \"install\" step failed"}"#),
        Some(Event::Error { message: "the \"install\" step failed".into(), log_tail: String::new() })
    );
    assert_eq!(
        parse_progress(r#"{"step":"error","message":"no disk","log_tail":"a\nb"}"#),
        Some(Event::Error { message: "no disk".into(), log_tail: "a\nb".into() })
    );
    assert_eq!(
        parse_progress(r#"{"step":"bootloader","percent":250}"#),
        Some(Event::Progress { step: Step::Bootloader, percent: 100, message: String::new() })
    );
    assert_eq!(parse_progress("copying path '/nix/store/x'"), None);
    assert_eq!(parse_progress(r#"{"step":"dance","percent":1}"#), None);
    assert_eq!(parse_progress(""), None);
}

#[test]
fn install_args_follow_the_contract() {
    let choices = Choices {
        disk: "/dev/vda".into(),
        hostname: "flux".into(),
        full_name: "Ada Lovelace".into(),
        username: "ada".into(),
        timezone: "Europe/Paris".into(),
        keymap: "fr".into(),
        locale: "fr_FR.UTF-8".into(),
        software: Software::default(),
    };
    let args = install_args(&choices, "$6$salt$hash");
    assert_eq!(
        args.join(" "),
        "--disk /dev/vda --hostname flux --full-name Ada Lovelace --user ada --password-hash $6$salt$hash \
         --timezone Europe/Paris --keymap fr --locale fr_FR.UTF-8 --facile yes --apps yes --gaming no --nvidia no \
         --yes --progress json"
    );
}

#[test]
fn tail_keeps_the_last_lines() {
    assert_eq!(tail("a\nb\nc\n", 2), "b\nc");
    assert_eq!(tail("a", 5), "a");
    assert_eq!(tail("", 5), "");
}

#[test]
fn every_language_layout_is_offered() {
    for language in LANGUAGES {
        assert!(KEYMAPS.iter().any(|(code, _)| *code == language.keymap), "{}", language.name);
    }
}

#[test]
fn nvidia_needs_a_turing_or_newer_display_card() {
    assert!(nvidia_display("0x10de\n", "0x2684\n", "0x030000\n"));
    assert!(nvidia_display("0x10de", "0x1e04", "0x030200"));
    assert!(!nvidia_display("0x10de", "0x1b80", "0x030000"), "a GTX 1080 needs the legacy driver");
    assert!(!nvidia_display("0x10de", "0x2684", "0x040300"), "the card's audio function is not a display");
    assert!(!nvidia_display("0x1002", "0x73bf", "0x030000"), "AMD is not NVIDIA");
}

#[test]
fn install_args_carry_the_software_choices() {
    let choices = Choices {
        disk: "/dev/vda".to_string(),
        software: Software { facile: false, apps: true, gaming: true, nvidia: false },
        ..Choices::default()
    };
    let args = install_args(&choices, "$y$hash");
    let pair = |flag: &str| args.iter().position(|arg| arg == flag).and_then(|at| args.get(at + 1)).cloned();
    assert_eq!(pair("--disk").as_deref(), Some("/dev/vda"));
    assert_eq!(pair("--facile").as_deref(), Some("no"));
    assert_eq!(pair("--apps").as_deref(), Some("yes"));
    assert_eq!(pair("--gaming").as_deref(), Some("yes"));
    assert_eq!(pair("--nvidia").as_deref(), Some("no"));
}

#[test]
fn zone1970_coordinates_parse_both_precisions() {
    let (lat, lon) = parse_coordinates("+4852+00220").unwrap_or_default();
    assert!((lat - 48.866_666).abs() < 1e-3 && (lon - 2.333_333).abs() < 1e-3, "Paris: {lat} {lon}");
    let (lat, lon) = parse_coordinates("-334807+1510636").unwrap_or_default();
    assert!((lat + 33.801_944).abs() < 1e-3 && (lon - 151.11).abs() < 1e-2, "Sydney: {lat} {lon}");
    let (lat, lon) = parse_coordinates("+4043-07400").unwrap_or_default();
    assert!((lat - 40.716_666).abs() < 1e-3 && (lon + 74.0).abs() < 1e-3, "New York: {lat} {lon}");
    assert_eq!(parse_coordinates("4852+00220"), None);
    assert_eq!(parse_coordinates("+48x2+00220"), None);
}

#[test]
fn zone_points_skip_comments_and_keep_the_zone_name() {
    let tab = "# comment\nFR,MC\t+4852+00220\tEurope/Paris\nAU\t-3352+15113\tAustralia/Sydney\tNew South Wales\n";
    let points = parse_zone_points(tab);
    assert_eq!(points.len(), 2);
    assert_eq!(points[0].zone, "Europe/Paris");
    assert_eq!(points[1].zone, "Australia/Sydney");
    assert!(points[1].latitude < 0.0);
}
