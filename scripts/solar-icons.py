"""Write the Flux icon theme from the Solar icon set.

Usage: python3 -I scripts/solar-icons.py <solar icons.json> <mdi icons.json>

The two files come from the @iconify-json/solar and @iconify-json/mdi npm packages. The
script writes pkgs/flux-icons/icons/<name>-symbolic.svg for every freedesktop name in MAP,
and for every name in MDI from Material Design Icons, whose chevrons muse uses,
pkgs/flux-icons/places/<name>.svg and places-dark/<name>.svg in the PLACE_INK of
each scheme for every name in PLACES (the full-colour folders Files draws), then rewrites the statusbar masks from
STATUSBAR with ImageMagick.
"""

import json
import pathlib
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
STYLE = "bold-duotone"
BAR_STYLE = "bold"
INK = "#2e3436"
PLACE_INK = {"places": "#737373", "places-dark": "#a3a3a3"}

MAP = {
    "airplane-mode": "airplane",
    "applications-engineering": "settings",
    "application-x-executable": "widget-5",
    "audio-card": "soundwave",
    "audio-headphones": "headphones-round",
    "audio-headset": "headphones-round-sound",
    "audio-input-microphone": "microphone",
    "microphone-sensitivity-muted": "microphone-off",
    "audio-speakers": "speaker",
    "audio-volume-high": "volume-loud",
    "audio-volume-medium": "volume-small",
    "audio-volume-low": "volume",
    "audio-volume-muted": "volume-cross",
    "battery-missing": "battery-low",
    "bluetooth-active": "bluetooth",
    "bluetooth-disabled": "bluetooth-off",
    "bluetooth": "bluetooth",
    "computer": "monitor",
    "display-brightness": "sun",
    "document-open-recent": "history",
    "document-open": "folder-open",
    "document-save": "diskette",
    "drive-harddisk": "ssd-round",
    "drive-removable-media": "flash-drive",
    "media-removable": "flash-drive",
    "go-home": "home-2",
    "help-about": "info-circle",
    "dialog-information": "info-circle",
    "dialog-warning": "danger-triangle",
    "dialog-error": "danger-circle",
    "dialog-question": "question-circle",
    "image-x-generic": "gallery",
    "input-gaming": "gamepad",
    "input-keyboard": "keyboard",
    "input-mouse": "mouse",
    "list-add": "add-circle",
    "list-remove": "minus-circle",
    "media-playback-start": "play",
    "media-playback-pause": "pause",
    "media-skip-forward": "skip-next",
    "media-skip-backward": "skip-previous",
    "network-wireless": "wi-fi",
    "network-wireless-signal-excellent": "wi-fi-high",
    "network-wireless-signal-good": "wi-fi-high",
    "network-wireless-signal-ok": "wi-fi-low",
    "network-wireless-signal-weak": "wi-fi-low",
    "network-wireless-signal-none": "wi-fi-none",
    "network-wireless-offline": "wi-fi-off",
    "network-wireless-disabled": "wi-fi-off",
    "network-wired": "plug-circle",
    "network-offline": "wi-fi-cross",
    "object-select": "check-circle",
    "emblem-ok": "check-circle",
    "phone": "smartphone",
    "preferences-desktop-appearance": "palette",
    "preferences-system": "settings",
    "software-update-available": "refresh-circle",
    "system-lock-screen": "lock-keyhole",
    "system-log-out": "logout-2",
    "system-reboot": "restart",
    "system-shutdown": "power",
    "system-users": "users-group-rounded",
    "avatar-default": "user-circle",
    "video-display": "monitor",
    "weather-clear": "sun",
    "weather-clear-night": "moon",
    "window-close": "close-circle",
    "edit-clear": "close-circle",
    "edit-find": "magnifer",
    "system-search": "magnifer",
    "edit-copy": "copy",
    "edit-paste": "clipboard-paste",
    "edit-delete": "trash-bin-minimalistic",
    "user-trash": "trash-bin-minimalistic",
    "view-reveal": "eye",
    "view-conceal": "eye-closed",
    "open-menu": "hamburger-menu",
    "view-more": "menu-dots",
    "folder": "folder",
    "folder-open": "folder-open",
    "folder-documents": "folder-with-files",
    "folder-download": "folder-down",
    "folder-music": "folder",
    "folder-pictures": "folder",
    "folder-videos": "folder",
    "folder-templates": "folder",
    "folder-publicshare": "folder-cloud",
    "folder-remote": "folder-cloud",
    "user-desktop": "folder",
    "inode-directory": "folder",
    "user-home": "home-2",
    "mail-unread": "letter-unread",
    "notifications": "bell",
    "notifications-disabled": "bell-off",
    "x-office-calendar": "calendar",
    "preferences-system-time": "clock-circle",
}

MDI = {
    "go-up": "chevron-up",
    "go-down": "chevron-down",
    "go-next": "chevron-right",
    "go-previous": "chevron-left",
    "pan-up": "chevron-up",
    "pan-down": "chevron-down",
    "pan-end": "chevron-right",
    "pan-start": "chevron-left",
}

PLACES = {
    "folder": "folder",
    "folder-open": "folder-open",
    "folder-drag-accept": "folder-open",
    "folder-documents": "folder-with-files",
    "folder-download": "folder-down",
    "folder-music": "folder",
    "folder-pictures": "folder",
    "folder-videos": "folder",
    "folder-templates": "folder",
    "folder-publicshare": "folder-cloud",
    "folder-remote": "folder-cloud",
    "user-home": "folder",
    "user-desktop": "folder",
    "inode-directory": "folder",
}

for level in range(0, 101, 10):
    base = "battery-low" if level <= 20 else "battery-half" if level <= 60 else "battery-full"
    MAP[f"battery-level-{level}"] = base
    MAP[f"battery-level-{level}-charging"] = "battery-charge"
MAP["battery-level-100-charged"] = "battery-charge"

STATUSBAR = {
    "BATTERY": "battery-full",
    "BRIGHTNESS": "sun",
    "VOLUME": "volume-loud",
    "NETWORK": "wi-fi",
    "LOGOUT": "power",
    "APPS": "widget",
    "CLOSE": "close-circle",
    "BATTERY_LOW": "battery-low",
    "BATTERY_HALF": "battery-half",
    "BATTERY_CHARGE": "battery-charge",
}


def svg(body, size, ink=INK):
    """A standalone SVG of one Solar body in `ink`, by default GTK's symbolic ink."""
    body = body.replace("currentColor", ink)
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 24 24">{body}</svg>\n'


def body(icons, name, style=STYLE):
    """The SVG body of Solar icon `name` in `style`."""
    key = f"{name}-{style}"
    if key not in icons:
        sys.exit(f"solar has no {key}")
    return icons[key]["body"]


def alpha_rows(icons, name):
    """Solar icon `name` rasterized to 24x24 alpha, as 24 rows of hex bytes."""
    with tempfile.TemporaryDirectory() as tmp:
        source = pathlib.Path(tmp) / "icon.svg"
        source.write_text(svg(body(icons, name, BAR_STYLE), 24).replace(INK, "#000000"))
        raw = subprocess.run(
            ["magick", "-background", "none", "-density", "384", str(source),
             "-resize", "24x24", "-alpha", "extract", "-depth", "8", "gray:-"],
            check=True, capture_output=True,
        ).stdout
    if len(raw) != 24 * 24:
        sys.exit(f"{name}: expected 576 alpha bytes, got {len(raw)}")
    return [raw[row * 24:(row + 1) * 24].hex() for row in range(24)]


def write_masks(icons):
    """Rewrite the statusbar masks module from STATUSBAR."""
    path = ROOT / "shell/crates/statusbar/src/icons/masks.rs"
    out = [
        "//! The compiled-in artwork of every icon, rasterized from the Solar set by",
        "//! 480 Design (CC BY 4.0, https://www.figma.com/community/file/1166831539721848736)",
        "//! with `scripts/solar-icons.py`.",
        "//!",
        "//! Each mask is one icon's 24×24 coverage in reading order: 24 rows of 48 hex",
        "//! digits, one byte of alpha per pixel, top to bottom. Nothing is decoded from",
        "//! SVG at run time.",
        "",
    ]
    for constant, name in STATUSBAR.items():
        out.append(f"/// The {constant.lower()} mask: Solar `{name}`, {BAR_STYLE}.")
        out.append(f'pub(super) const {constant}: &str = "\\')
        out.extend(alpha_rows(icons, name))
        out[-1] += '";'
        out.append("")
    path.write_text("\n".join(out))


def main(source, mdi_source):
    icons = json.loads(pathlib.Path(source).read_text())["icons"]
    mdi = json.loads(pathlib.Path(mdi_source).read_text())["icons"]
    target = ROOT / "pkgs/flux-icons/icons"
    target.mkdir(parents=True, exist_ok=True)
    for old in target.glob("*.svg"):
        old.unlink()
    for name, solar in sorted(MAP.items()):
        (target / f"{name}-symbolic.svg").write_text(svg(body(icons, solar), 16))
    for name, chevron in sorted(MDI.items()):
        (target / f"{name}-symbolic.svg").write_text(svg(mdi[chevron]["body"], 16))
    for folder, ink in PLACE_INK.items():
        places = ROOT / "pkgs/flux-icons" / folder
        places.mkdir(parents=True, exist_ok=True)
        for old in places.glob("*.svg"):
            old.unlink()
        for name, solar in sorted(PLACES.items()):
            (places / f"{name}.svg").write_text(svg(body(icons, solar), 64, ink))
    write_masks(icons)
    print(f"wrote {len(MAP) + len(MDI)} icons, {len(PLACES)} places and {len(STATUSBAR)} masks")


if __name__ == "__main__":
    main(*sys.argv[1:3])
