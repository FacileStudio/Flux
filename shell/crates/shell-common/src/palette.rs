//! The palette files and the one place a stylesheet reaches the display.
//!
//! Flux installs `/etc/flux/palette-light.css` and `palette-dark.css`, plus
//! `palette.css` for the starting scheme, generated from the muse design tokens:
//! one `@define-color fc_<token>` line per token (`--color-fc-page` becomes
//! `fc_page`). [`crate::scheme`] picks the light or dark file from the portal.
//! Every widget stylesheet references only those `@fc_*` names. A
//! built-in copy of the muse light palette stands in when the system file is
//! missing, so a widget never starts unstyled.
//!
//! `~/.config/flux/palette.css` is read after the system palette. GTK keeps
//! the last `@define-color` for a name, so any `fc_*` line in the user file
//! overrides the system one.
//!
//! Everything a widget draws hangs off a single [`gtk4::CssProvider`] that this
//! module hands back to [`crate::theme`]. Keeping the provider around is what
//! lets a running widget reread the palette and restyle itself — see
//! [`crate::theme::init`].

use std::path::{Path, PathBuf};

use crate::scheme::{self, Scheme};

/// Where Flux installs the palettes it generates from the muse tokens:
/// `palette-light.css`, `palette-dark.css`, and `palette.css` for the scheme
/// the system starts in.
const SYSTEM_DIR: &str = "/etc/flux";

/// Where the user's palette override and widget sheets live, under `$HOME`.
const CONFIG_DIR: &str = ".config/flux";

/// The user's palette override inside that directory.
const PALETTE_FILE: &str = "palette.css";

/// The muse light palette, used when `/etc/flux/palette.css` is missing.
pub(crate) const FALLBACK: &str = r#"
@define-color fc_page #ffffff;
@define-color fc_bg #ffffff;
@define-color fc_surface #eeeeee;
@define-color fc_component #f7f7f7;
@define-color fc_fg #0a0a0a;
@define-color fc_fg_muted #636363;
@define-color fc_accent #0a0a0a;
@define-color fc_accent_fg #ffffff;
@define-color fc_border #dedede;
@define-color fc_ring #484848;
@define-color fc_danger #d00405;
@define-color fc_danger_fg #ffffff;
@define-color fc_success #21763c;
@define-color fc_info #2366b4;
@define-color fc_warning #935b00;
@define-color fc_owner #9b5500;
@define-color fc_admin #684bb5;
@define-color fc_scrim rgba(0, 0, 0, 0.5);
"#;

/// `$HOME/.config/flux`, where the user's override and sheets live.
pub(crate) fn config_dir() -> Option<PathBuf> {
    let mut path = PathBuf::from(std::env::var_os("HOME")?);
    path.push(CONFIG_DIR);
    Some(path)
}

/// The user's palette override, `$HOME/.config/flux/palette.css`.
pub(crate) fn palette_file() -> Option<PathBuf> {
    Some(config_dir()?.join(PALETTE_FILE))
}

/// A widget's own sheet, `$HOME/.config/flux/<name>.css`. When the user places
/// one it replaces the sheet the widget ships and is watched for hot-reload,
/// mirroring the palette hot-reload that already exists for `palette.css`.
pub(crate) fn sheet_file(name: &str) -> Option<PathBuf> {
    Some(config_dir()?.join(format!("{name}.css")))
}

/// The system palette in `dir` for `scheme`: `palette-<scheme>.css` when the
/// scheme is known and that file exists, `palette.css` otherwise.
pub(crate) fn system_palette_in(dir: &Path, scheme: Option<Scheme>) -> PathBuf {
    scheme
        .map(|scheme| dir.join(format!("palette-{}.css", scheme.name())))
        .filter(|path| path.is_file())
        .unwrap_or_else(|| dir.join("palette.css"))
}

/// The system palette for the scheme the portal reports.
fn system_palette_file() -> PathBuf {
    system_palette_in(Path::new(SYSTEM_DIR), scheme::current())
}

/// The palette document: the system palette, or the built-in fallback when it
/// is missing, followed by the user's override when one exists.
pub(crate) fn palette_css() -> String {
    let system = std::fs::read_to_string(system_palette_file())
        .unwrap_or_else(|_| FALLBACK.to_string());
    let user = palette_file()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .unwrap_or_default();
    format!("{system}\n{user}")
}

/// The `fc_fg` colour from the palette, as a GDK colour.
///
/// A widget that rasterizes a bitmap (an icon, say) cannot pull its colour from
/// the stylesheet cascade the way a label does, so it reads the token here
/// instead. When the palette's value cannot be parsed, the built-in fallback's
/// `fc_fg` is used, so this is `None` only if both fail.
pub fn fg_rgba() -> Option<gtk4::gdk::RGBA> {
    named_color(&palette_css(), "fc_fg")
        .and_then(|value| gtk4::gdk::RGBA::parse(value.as_str()).ok())
        .or_else(|| {
            let value = named_color(FALLBACK, "fc_fg")?;
            gtk4::gdk::RGBA::parse(value.as_str()).ok()
        })
}

/// Pull the last `@define-color <name> <value>;` declaration out of a
/// stylesheet document, the one GTK applies.
fn named_color(sheet: &str, name: &str) -> Option<String> {
    let prefix = format!("@define-color {name} ");
    sheet.lines().rev().find_map(|line| {
        let rest = line.trim().strip_prefix(&prefix)?;
        let value = rest.trim().trim_end_matches(';').trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

/// Register the provider that holds a widget's whole stylesheet on the display.
///
/// One provider holds the whole document so the cascade inside it is decided
/// by source order instead of by registration order.
pub(crate) fn register(provider: &gtk4::CssProvider) {
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
