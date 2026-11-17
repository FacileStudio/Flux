//! The shared visual language of every widget.
//!
//! One base stylesheet defines what every widget agrees on — transparent
//! windows, muted text, filled buttons, panel surfaces — so a widget's own
//! stylesheet only has to say what makes it different. Colours come from the
//! muse palette (`/etc/flux/palette.css`, overridden by
//! `~/.config/flux/palette.css`), never from literals.
//!
//! [`init`] loads the three layers into a single provider, then watches the
//! user's palette and sheet and follows the light or dark scheme, so a running
//! widget restyles itself instead of waiting for a restart. The reload rereads
//! what it is built from (see [`rebuild`]) because a document captured at
//! start-up would be loaded back unchanged, which is the same as not watching.
//!
//! Motion lives here too: every button eases its fill on hover and press, and
//! the `fx-reveal` classes give [`crate::motion`] its slide and fade. All of it
//! is CSS transitions, which GTK drops when `gtk-enable-animations` is off.

use std::cell::RefCell;

mod watch;

use crate::palette;

/// The gap around a floating widget, matching `gaps 10` in `niri.kdl`. The bar
/// is inset by it on every side, and the sidebar and its toasts keep it off the
/// right and bottom edges so they float the same way.
///
/// niri gaps and layer-shell margins are both logical pixels, scaled by the
/// output, so the two numbers stay literally equal at any scale.
pub const GAP: i32 = 10;

/// The gap between two modules on a widget and between the items inside one.
///
/// One measure for both, from the 4/8/12/16/24 scale the shared sheet follows,
/// so a row reads as a single rhythm: the statusbar's workspace chips, its CPU
/// and memory pills and its control pills are all this far apart, and one module
/// is this far from the next. It used to be 6 inside a module and 0 between two
/// of them, which left the CPU and memory pills welded to the control row while
/// every other pair on the bar had a gap.
pub const MODULE_GAP: i32 = 8;

/// Height of the statusbar, and the strip it reserves at the top of the
/// screen. A widget that docks to an edge keeps its content out of that strip
/// so it never sits underneath the bar. The bar itself is a floating pill, so
/// this is the pill (32px) plus a `GAP` above and below it.
///
/// The bar's window is exactly this tall, so this is not an estimate the
/// sidebar has to guess around: `.bar` fills 32px of content between two `GAP`
/// margins and no vertical padding, which sums to this. `statusbar` reserves it
/// as its exclusive zone, so windows and the sidebar both start below it.
pub const BAR_HEIGHT: i32 = 32 + GAP;
pub const WINDOW_TOP_GAP: i32 = BAR_HEIGHT + GAP;

/// Loaded before every widget stylesheet, which may override any of it.
///
/// The rules follow the muse charter so the shell reads as one system with the
/// Facile apps: colours only through the `@fc_*` palette names, Inter as the UI
/// face, a 4/8/12/16/24 spacing scale, fill steps instead of borders (none
/// anywhere, inputs included), `fc_ring` as the one focus outline, and the muse
/// radii (12px on a surface, 8px on what it holds).
pub const BASE_CSS: &str = r#"
* {
  border: none;
  box-shadow: none;
}

window {
  background-color: transparent;
  font-family: "Inter", sans-serif;
  font-size: 14px;
  color: @fc_fg;
}

window.backdrop {
  background-color: transparent;
}

.dim {
  color: @fc_fg_muted;
  font-size: 12px;
}

.surface {
  background-color: @fc_component;
  color: @fc_fg;
  border-radius: 12px;
}

button {
  background-color: @fc_surface;
  background-image: none;
  color: @fc_fg;
  font-size: 12px;
  font-weight: 500;
  padding: 2px 8px;
  min-height: 20px;
  min-width: 20px;
  border-radius: 8px;
  outline: 0 solid transparent;
  transition: background-color 160ms ease-out, color 160ms ease-out, opacity 160ms ease-out;
}

button:hover {
  background-color: mix(@fc_surface, @fc_fg, 0.08);
}

button:active {
  background-color: mix(@fc_surface, @fc_fg, 0.14);
}

.fx-fade,
.fx-reveal,
.fx-reveal-side {
  opacity: 0;
  transition: opacity 160ms ease-out, transform 160ms ease-out;
}

.fx-reveal {
  transform: translateY(-8px);
}

.fx-reveal-side {
  transform: translateX(16px);
}

.fx-fade.fx-shown,
.fx-reveal.fx-shown,
.fx-reveal-side.fx-shown {
  opacity: 1;
  transform: none;
}

button:focus-visible {
  outline: 2px solid @fc_ring;
  outline-offset: 2px;
}

button:disabled {
  background-color: transparent;
  color: @fc_fg_muted;
}
"#;

thread_local! {
    /// The callbacks to run after the stylesheet is rebuilt, in the order they
    /// were registered. See [`on_reload`].
    static RELOADS: RefCell<Vec<Box<dyn Fn()>>> = const { RefCell::new(Vec::new()) };
}

/// Style a widget: palette, then the shared base, then its own stylesheet.
///
/// The three layers go into a single provider, so the widget sheet always wins
/// over the base and both win over nothing else. The provider is then tied to
/// `~/.config/flux/palette.css`: when the file changes, the document is
/// rebuilt from the new palette and loaded back into the same provider, which
/// restyles every widget on the display without a restart.
///
/// If `~/.config/flux/<name>.css` exists it is used as the widget sheet
/// instead of the compiled-in default, and it is watched for hot-reload — edit
/// the file and the widget restyles itself in place.
pub fn init(widget_css: &'static str, name: &str) {
    let provider = gtk4::CssProvider::new();
    let scheme_provider = provider.clone();
    let scheme_name = name.to_string();
    crate::scheme::follow(move || watch::reload(&scheme_provider, &scheme_name, widget_css));
    provider.load_from_data(&rebuild(name, widget_css));
    palette::register(&provider);
    watch::watch(provider, name, widget_css);
}

/// Run `callback` after every stylesheet rebuild: a colour scheme switch, a palette edit, or an edit
/// to the user's own sheet.
///
/// Most of what a widget draws follows the cascade by itself and needs nothing
/// here. This is for the parts that cannot: a bitmap is a fixed texture rather
/// than a rule, so the only way it takes the new palette is to be drawn again.
/// The statusbar's icons are the case that made this necessary.
///
/// A callback must not register another one — the list is borrowed while they
/// run.
pub fn on_reload(callback: impl Fn() + 'static) {
    RELOADS.with(|slot| slot.borrow_mut().push(Box::new(callback)));
}

/// The stylesheet a widget draws itself from, read from disk now: the palette,
/// the shared base, and the widget's own sheet.
///
/// Read rather than remembered. Both `init` and the reload go through here, so
/// what is loaded on a change is the files as they are, not a copy of what they
/// said when the widget started.
pub(crate) fn rebuild(name: &str, default: &str) -> String {
    document(&user_sheet(name).unwrap_or_else(|| default.to_string()))
}

/// The full stylesheet: the palette first, then the base, then the widget's.
fn document(widget_css: &str) -> String {
    format!("{}\n{}\n{}", palette::palette_css(), BASE_CSS, widget_css)
}

/// The sheet the user placed at `$HOME/.config/flux/<name>.css`. When present
/// it replaces the compiled-in default so the user's file is what the widget
/// renders.
fn user_sheet(name: &str) -> Option<String> {
    std::fs::read_to_string(palette::sheet_file(name)?).ok()
}

/// Tell every listener the stylesheet moved under them.
fn notify_reload() {
    RELOADS.with(|slot| {
        for callback in slot.borrow().iter() {
            callback();
        }
    });
}

#[cfg(test)]
mod tests;
