//! Solar icons for the right-hand control pills.
//!
//! The bodies are the Solar set by 480 Design (CC BY 4.0), the same set as the
//! Flux icon theme, compiled in as the 24×24 alpha masks in [`masks`] and
//! painted from those into a texture. Nothing is rasterized at run time: the SVG
//! loader (librsvg) is a separate package on most distributions, and a machine
//! without it drew a pill's label and nothing beside it, silently.
//!
//! The tint is the palette's `fc_fg`. A texture is a fixed bitmap and cannot
//! follow the stylesheet cascade the way the label next to it does, so the icons
//! are painted once and painted again by [`follow_palette`] whenever the
//! stylesheet is rebuilt — which is what moves the glyphs and the text together
//! after the palette changes.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::gdk::{MemoryFormat, MemoryTexture, Texture, RGBA};
use gtk4::glib;
use gtk4::prelude::*;

mod masks;

/// The side of every icon, in pixels.
const SIDE: i32 = 24;

/// The icons the bar draws. Only the ones it actually uses are compiled in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Battery,
    BatteryHalf,
    BatteryLow,
    BatteryCharge,
    Brightness,
    Volume,
    Network,
    Logout,
    Apps,
    Close,
}

/// Every icon, in the order the control row reads them. One list, so a new icon
/// is added in one place and the checks and the diagnostic report both follow.
pub const ALL: [Icon; 10] = [
    Icon::Battery,
    Icon::BatteryHalf,
    Icon::BatteryLow,
    Icon::BatteryCharge,
    Icon::Brightness,
    Icon::Volume,
    Icon::Network,
    Icon::Logout,
    Icon::Apps,
    Icon::Close,
];

impl Icon {
    /// The icon's 24×24 mask, the artwork it paints from.
    fn mask(self) -> &'static str {
        match self {
            Self::Battery => masks::BATTERY,
            Self::BatteryHalf => masks::BATTERY_HALF,
            Self::BatteryLow => masks::BATTERY_LOW,
            Self::BatteryCharge => masks::BATTERY_CHARGE,
            Self::Brightness => masks::BRIGHTNESS,
            Self::Volume => masks::VOLUME,
            Self::Network => masks::NETWORK,
            Self::Logout => masks::LOGOUT,
            Self::Apps => masks::APPS,
            Self::Close => masks::CLOSE,
        }
    }
}

/// The icons a widget has painted, remembered so a palette change can repaint
/// them.
///
/// A pill asks for its image once, when the bar is built; the images it gets back
/// stay in here, which is what lets [`retint`](Painted::retint) find them again
/// after the palette has moved.
/// The images are held weakly: a bar that leaves with its monitor takes its
/// glyphs with it, and the next repaint forgets them.
#[derive(Default)]
pub struct Painted(Vec<(Icon, glib::WeakRef<gtk4::Image>)>);

impl Painted {
    /// Paint `icon` in the palette's current foreground, and remember it.
    pub fn image(&mut self, icon: Icon) -> gtk4::Image {
        let image = gtk4::Image::from_paintable(Some(&texture(icon, &tint())));
        self.0.retain(|(_, image)| image.upgrade().is_some());
        self.0.push((icon, image.downgrade()));
        image
    }

    /// Make `image` show `icon` from now on, so a later [`retint`](Painted::retint)
    /// paints the new glyph. An image that already shows it is left alone.
    pub fn swap(&mut self, image: &gtk4::Image, icon: Icon) {
        let Some(entry) = self.0.iter_mut().find(|(_, kept)| kept.upgrade().as_ref() == Some(image)) else {
            return;
        };
        if entry.0 != icon {
            entry.0 = icon;
            image.set_paintable(Some(&texture(icon, &tint())));
        }
    }

    /// Paint every icon again in the palette's current foreground.
    pub fn retint(&self) {
        let tint = tint();
        for (icon, image) in &self.0 {
            if let Some(image) = image.upgrade() {
                image.set_paintable(Some(&texture(*icon, &tint)));
            }
        }
    }
}

/// Repaint `icons` every time the stylesheet is rebuilt.
///
/// The labels follow the cascade on their own; a texture cannot, so this is what
/// brings the glyphs along when the palette or the scheme changes, and what
/// keeps them from being a restart behind the bar they sit on.
pub fn follow_palette(icons: Rc<RefCell<Painted>>) {
    shell_common::theme::on_reload(move || icons.borrow().retint());
}

/// The ink the glyphs are drawn in: the palette's foreground, or a neutral grey
/// when it cannot be parsed, so a pill is never left without its icon.
pub(crate) fn tint() -> RGBA {
    shell_common::palette::fg_rgba().unwrap_or_else(|| RGBA::new(0.5, 0.5, 0.5, 1.0))
}

/// The 24×24 texture one icon paints as.
pub(crate) fn texture(icon: Icon, tint: &RGBA) -> Texture {
    let pixels = glib::Bytes::from(&rgba(icon, tint)[..]);
    let stride = SIDE as usize * 4;
    MemoryTexture::new(
        SIDE,
        SIDE,
        MemoryFormat::R8g8b8a8Premultiplied,
        &pixels,
        stride,
    )
    .upcast()
}

/// The mask's pixels as premultiplied RGBA bytes, in reading order: `tint`
/// scaled by each pixel's coverage, so the anti-aliased edges and the duotone
/// half-tones keep their weight.
fn rgba(icon: Icon, tint: &RGBA) -> Vec<u8> {
    let ink = [tint.red(), tint.green(), tint.blue()].map(|channel| f32::from(channel_byte(channel)));
    let mut pixels = Vec::with_capacity((SIDE * SIDE * 4) as usize);
    for row in icon.mask().lines() {
        for alpha in row.as_bytes().chunks(2).map(alpha_byte) {
            let scale = f32::from(alpha) / 255.0;
            let [red, green, blue] = ink.map(|channel| (channel * scale).round() as u8);
            pixels.extend_from_slice(&[red, green, blue, alpha]);
        }
    }
    pixels
}

fn channel_byte(channel: f32) -> u8 {
    (channel.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// The coverage one pair of hex digits of a mask stands for.
fn alpha_byte(pair: &[u8]) -> u8 {
    std::str::from_utf8(pair).ok().and_then(|hex| u8::from_str_radix(hex, 16).ok()).unwrap_or(0)
}

#[cfg(test)]
mod tests;
