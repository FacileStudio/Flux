"""Turn the muse color tokens into GTK, Ghostty and fuzzel themes for Flux."""

import math
import re
import sys
from pathlib import Path

BLOCK = re.compile(r"(@theme(?:\s+static)?|:root\.dark)\s*\{([^{}]*)\}")
COLOR = re.compile(r"--color-(fc-[a-z0-9-]+)\s*:\s*([^;]+);")
OKLCH = re.compile(
    r"oklch\(\s*([\d.]+)(%?)\s+([\d.]+)\s+([\d.]+)\s*(?:/\s*([\d.]+)(%?))?\s*\)"
)


def parse_tokens(css):
    """Return (light, dark) dicts of token name to OKLCH tuple (L, C, H, alpha)."""
    css = re.sub(r"/\*.*?\*/", "", css, flags=re.S)
    light, dark = {}, {}
    for selector, body in BLOCK.findall(css):
        target = dark if selector == ":root.dark" else light
        for name, value in COLOR.findall(body):
            target[name] = parse_oklch(name, value.strip())
    assert light, "no --color-fc-* tokens found in tokens.css"
    unknown = set(dark) - set(light)
    assert not unknown, f"dark tokens without a light value: {sorted(unknown)}"
    return light, {**light, **dark}


def parse_oklch(name, value):
    """Parse one oklch() value, failing the build on anything else."""
    match = OKLCH.fullmatch(value)
    assert match, f"--color-{name}: cannot convert {value!r}, only oklch() is supported"
    lightness, lpct, chroma, hue, alpha, apct = match.groups()
    lightness = float(lightness) / (100 if lpct else 1)
    alpha = 1.0 if alpha is None else float(alpha) / (100 if apct else 1)
    return (lightness, float(chroma), float(hue), alpha)


def oklch_to_linear(lightness, chroma, hue):
    """Convert OKLCH to linear sRGB, unclipped."""
    a = chroma * math.cos(math.radians(hue))
    b = chroma * math.sin(math.radians(hue))
    l = (lightness + 0.3963377774 * a + 0.2158037573 * b) ** 3
    m = (lightness - 0.1055613458 * a - 0.0638541728 * b) ** 3
    s = (lightness - 0.0894841775 * a - 1.2914855480 * b) ** 3
    return (
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    )


def in_gamut(rgb):
    """True when every linear channel sits inside [0, 1]."""
    return all(-1e-4 <= c <= 1 + 1e-4 for c in rgb)


def to_rgb(color):
    """Map an OKLCH tuple to 8-bit sRGB, reducing chroma until it fits the gamut."""
    lightness, chroma, hue, _ = color
    lightness = min(max(lightness, 0.0), 1.0)
    rgb = oklch_to_linear(lightness, chroma, hue)
    if not in_gamut(rgb):
        low, high = 0.0, chroma
        for _ in range(40):
            mid = (low + high) / 2
            low, high = (mid, high) if in_gamut(oklch_to_linear(lightness, mid, hue)) else (low, mid)
        rgb = oklch_to_linear(lightness, low, hue)
    out = []
    for c in rgb:
        c = min(max(c, 0.0), 1.0)
        c = 12.92 * c if c <= 0.0031308 else 1.055 * c ** (1 / 2.4) - 0.055
        out.append(round(c * 255))
    assert all(0 <= c <= 255 for c in out), f"conversion failed for {color}"
    return tuple(out)


def hex_of(color):
    """Return #rrggbb, ignoring alpha."""
    return "#{:02x}{:02x}{:02x}".format(*to_rgb(color))


def css_of(color):
    """Return #rrggbb when opaque, rgba() when translucent."""
    if color[3] >= 1:
        return hex_of(color)
    r, g, b = to_rgb(color)
    return f"rgba({r}, {g}, {b}, {color[3]:g})"


def gtk_name(token):
    """fc-fg-muted becomes fc_fg_muted."""
    return token.replace("-", "_")


def palette_css(tokens):
    """The shell contract: one @define-color per token."""
    lines = [f"@define-color {gtk_name(t)} {css_of(c)};" for t, c in sorted(tokens.items())]
    return "\n".join(lines) + "\n"


def adwaita_vars(t):
    """libadwaita 1.6+ CSS variables mapped onto muse."""
    c = {k: css_of(v) for k, v in t.items()}
    clear = "transparent"
    pairs = {
        "window-bg-color": c["fc-page"],
        "window-fg-color": c["fc-fg"],
        "view-bg-color": c["fc-bg"],
        "view-fg-color": c["fc-fg"],
        "headerbar-bg-color": c["fc-page"],
        "headerbar-fg-color": c["fc-fg"],
        "headerbar-backdrop-color": c["fc-page"],
        "headerbar-border-color": clear,
        "headerbar-shade-color": clear,
        "headerbar-darker-shade-color": clear,
        "sidebar-bg-color": c["fc-component"],
        "sidebar-fg-color": c["fc-fg"],
        "sidebar-backdrop-color": c["fc-component"],
        "sidebar-border-color": clear,
        "sidebar-shade-color": clear,
        "secondary-sidebar-bg-color": c["fc-component"],
        "secondary-sidebar-fg-color": c["fc-fg"],
        "secondary-sidebar-backdrop-color": c["fc-component"],
        "secondary-sidebar-border-color": clear,
        "secondary-sidebar-shade-color": clear,
        "card-bg-color": c["fc-component"],
        "card-fg-color": c["fc-fg"],
        "card-shade-color": clear,
        "dialog-bg-color": c["fc-page"],
        "dialog-fg-color": c["fc-fg"],
        "popover-bg-color": c["fc-bg"],
        "popover-fg-color": c["fc-fg"],
        "thumbnail-bg-color": c["fc-component"],
        "thumbnail-fg-color": c["fc-fg"],
        "accent-bg-color": c["fc-accent"],
        "accent-fg-color": c["fc-accent-fg"],
        "accent-color": c["fc-accent"],
        "destructive-bg-color": c["fc-danger"],
        "destructive-fg-color": c["fc-danger-fg"],
        "destructive-color": c["fc-danger"],
        "error-bg-color": c["fc-danger"],
        "error-fg-color": c["fc-danger-fg"],
        "error-color": c["fc-danger"],
        "success-bg-color": c["fc-success"],
        "success-fg-color": c["fc-danger-fg"],
        "success-color": c["fc-success"],
        "warning-bg-color": c["fc-warning"],
        "warning-fg-color": c["fc-danger-fg"],
        "warning-color": c["fc-warning"],
        "border-color": clear,
        "scrollbar-outline-color": clear,
    }
    return "".join(f"  --{k}: {v};\n" for k, v in pairs.items())


NO_BORDERS = """
entry, spinbutton, textview, searchbar > revealer > box, dropdown > button, combobox > box > button,
headerbar, .titlebar, .toolbar, actionbar > revealer > box, .card, list.boxed-list, frame, separator,
popover > contents, window.dialog, .sidebar-pane, paned > separator {
  border: none;
  border-color: transparent;
  box-shadow: none;
}

separator {
  background: none;
  min-width: 0;
  min-height: 0;
}
"""


def gtk4_css(light, dark):
    """User gtk.css for GTK 4: light by default, dark under prefers-color-scheme."""
    return (
        ":root {\n" + adwaita_vars(light) + "}\n\n"
        "@media (prefers-color-scheme: dark) {\n  :root {\n"
        + adwaita_vars(dark).replace("  --", "    --")
        + "  }\n}\n" + NO_BORDERS
    )


def gtk3_css(t, dark):
    """A GTK 3 theme layered over the built-in Adwaita."""
    c = {k: css_of(v) for k, v in t.items()}
    base = "gtk-contained-dark.css" if dark else "gtk-contained.css"
    named = {
        "theme_bg_color": "fc-page",
        "theme_fg_color": "fc-fg",
        "theme_base_color": "fc-bg",
        "theme_text_color": "fc-fg",
        "theme_selected_bg_color": "fc-accent",
        "theme_selected_fg_color": "fc-accent-fg",
        "theme_unfocused_bg_color": "fc-page",
        "theme_unfocused_fg_color": "fc-fg",
        "theme_unfocused_base_color": "fc-bg",
        "theme_unfocused_text_color": "fc-fg",
        "theme_unfocused_selected_bg_color": "fc-accent",
        "theme_unfocused_selected_fg_color": "fc-accent-fg",
        "insensitive_bg_color": "fc-component",
        "insensitive_fg_color": "fc-fg-muted",
        "insensitive_base_color": "fc-component",
        "warning_color": "fc-warning",
        "error_color": "fc-danger",
        "success_color": "fc-success",
    }
    defines = "".join(f"@define-color {k} {c[v]};\n" for k, v in named.items())
    defines += "@define-color borders transparent;\n@define-color unfocused_borders transparent;\n"
    rules = f"""
window, .background, headerbar, .titlebar, dialog {{
  background-color: {c["fc-page"]};
  background-image: none;
  color: {c["fc-fg"]};
}}

.view, textview text, iconview, treeview, list, row {{
  background-color: {c["fc-bg"]};
  color: {c["fc-fg"]};
}}

entry, spinbutton, button, menu, .menu, popover, .sidebar, .card {{
  background-color: {c["fc-component"]};
  background-image: none;
  color: {c["fc-fg"]};
}}

button:hover {{
  background-color: {c["fc-surface"]};
}}

button.suggested-action, *:selected, selection {{
  background-color: {c["fc-accent"]};
  color: {c["fc-accent-fg"]};
}}

button.destructive-action {{
  background-color: {c["fc-danger"]};
  color: {c["fc-danger-fg"]};
}}

entry, spinbutton, button, menu, .menu, popover, headerbar, .titlebar, .sidebar, .card, frame,
scrolledwindow, notebook, treeview, list, separator, tooltip {{
  border: none;
  border-color: transparent;
  box-shadow: none;
  text-shadow: none;
  -gtk-icon-shadow: none;
}}

*:focus {{
  outline-color: {c["fc-ring"]};
}}
"""
    return f'@import url("resource:///org/gtk/libgtk/theme/Adwaita/{base}");\n\n' + defines + rules


def shift(color, delta):
    """Move a token's lightness, keeping hue and chroma."""
    return (min(max(color[0] + delta, 0.0), 1.0), color[1], color[2], 1.0)


def grey(lightness):
    """A chroma-zero muse grey."""
    return (lightness, 0.0, 0.0, 1.0)


def ansi(t, dark):
    """16 colors: status hues from muse, chart hues for cyan, greys from the neutral scale."""
    band = t["fc-info"][0]
    cyan = (band, t["fc-chart-3"][1], t["fc-chart-3"][2], 1.0)
    magenta = (band, t["fc-chart-6"][1], t["fc-chart-6"][2], 1.0)
    hues = [t["fc-danger"], t["fc-success"], t["fc-warning"], t["fc-info"], magenta, cyan]
    step = 0.08 if dark else -0.06
    if dark:
        normal = [t["fc-surface"], *hues, grey(0.85)]
        bright = [t["fc-fg-muted"], *[shift(h, step) for h in hues], t["fc-fg"]]
    else:
        normal = [t["fc-fg"], *hues, t["fc-fg-muted"]]
        bright = [grey(0.4), *[shift(h, step) for h in hues], grey(0.65)]
    return normal + bright


def ghostty_theme(t, dark):
    """A Ghostty theme file."""
    lines = [f"palette = {i}={hex_of(c)}" for i, c in enumerate(ansi(t, dark))]
    lines += [
        f"background = {hex_of(t['fc-page'])}",
        f"foreground = {hex_of(t['fc-fg'])}",
        f"cursor-color = {hex_of(t['fc-accent'])}",
        f"cursor-text = {hex_of(t['fc-accent-fg'])}",
        f"selection-background = {hex_of(t['fc-accent'])}",
        f"selection-foreground = {hex_of(t['fc-accent-fg'])}",
    ]
    return "\n".join(lines) + "\n"


def fuzzel_colors(t):
    """The [colors] section of a fuzzel config, as RRGGBBAA."""
    def rgba(token):
        return hex_of(t[token])[1:] + "ff"
    return (
        "[colors]\n"
        f"background={rgba('fc-page')}\n"
        f"text={rgba('fc-fg')}\n"
        f"prompt={rgba('fc-fg-muted')}\n"
        f"placeholder={rgba('fc-fg-muted')}\n"
        f"input={rgba('fc-fg')}\n"
        f"match={rgba('fc-info')}\n"
        f"selection={rgba('fc-accent')}\n"
        f"selection-text={rgba('fc-accent-fg')}\n"
        f"selection-match={rgba('fc-accent-fg')}\n"
        f"counter={rgba('fc-fg-muted')}\n"
        "border=00000000\n"
    )


def write(root, rel, text):
    """Write one output file, refusing an empty one."""
    assert text.strip(), f"{rel} is empty"
    path = root / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def main(tokens_path, out_dir):
    """Generate every theme file under out_dir."""
    light, dark = parse_tokens(Path(tokens_path).read_text())
    for name in ("fc-page", "fc-bg", "fc-fg", "fc-accent", "fc-accent-fg", "fc-ring"):
        assert name in light, f"--color-{name} is missing from tokens.css"
    root = Path(out_dir)
    palette = palette_css(light)
    assert palette.count("@define-color") == len(light)
    write(root, "flux/palette.css", palette)
    write(root, "flux/palette-light.css", palette)
    write(root, "flux/palette-dark.css", palette_css(dark))
    write(root, "flux/gtk-4.0/gtk.css", gtk4_css(light, dark))
    write(root, "themes/Flux/gtk-3.0/gtk.css", gtk3_css(light, False))
    write(root, "themes/Flux/gtk-3.0/gtk-dark.css", gtk3_css(dark, True))
    write(root, "themes/Flux-dark/gtk-3.0/gtk.css", gtk3_css(dark, True))
    write(root, "flux/ghostty/themes/flux-light", ghostty_theme(light, False))
    write(root, "flux/ghostty/themes/flux-dark", ghostty_theme(dark, True))
    write(root, "flux/fuzzel/light.ini", fuzzel_colors(light))
    write(root, "flux/fuzzel/dark.ini", fuzzel_colors(dark))


if __name__ == "__main__":
    main(*sys.argv[1:3])
