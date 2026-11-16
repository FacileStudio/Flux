//! The blurred wallpaper behind the lock.
//!
//! The image is decoded off the main thread with GTK's own loaders (not
//! gdk-pixbuf, whose sandboxed glycin loaders can stall), shrunk to a small
//! width and box-blurred there; GTK scales the result up to the output, which
//! smooths it further. The worker returns plain bytes and the main thread
//! builds the texture.

use std::path::PathBuf;

use gtk4::prelude::*;
use gtk4::{gdk, glib};

/// The width the wallpaper is shrunk to before blurring.
const WIDTH: usize = 240;
/// The box radius, in pixels of the small image, of each of the three passes.
const RADIUS: usize = 4;

/// Shrunk, blurred pixels in GDK's default download format (BGRA, premultiplied).
pub struct Blurred {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

/// Decode `path`, shrink and blur it, or say why it could not be decoded.
pub fn load(path: PathBuf) -> Result<Blurred, String> {
    let texture = gdk::Texture::from_filename(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let (width, height) = (texture.width() as usize, texture.height() as usize);
    if width == 0 || height == 0 {
        return Err(format!("{}: empty image", path.display()));
    }
    let mut full = vec![0u8; width * height * 4];
    texture.download(&mut full, width * 4);
    let small_width = WIDTH.min(width);
    let small_height = (height * small_width / width).max(1);
    let mut pixels = shrink(&full, width, height, small_width, small_height);
    drop(full);
    for _ in 0..3 {
        pixels = blur_pass(&pixels, small_width, small_height, RADIUS, true);
        pixels = blur_pass(&pixels, small_width, small_height, RADIUS, false);
    }
    Ok(Blurred { width: small_width, height: small_height, pixels })
}

/// Shrink 4-byte pixels from `width`x`height` to `to_width`x`to_height` by
/// averaging every source pixel that falls in each target pixel.
pub fn shrink(pixels: &[u8], width: usize, height: usize, to_width: usize, to_height: usize) -> Vec<u8> {
    let mut sums = vec![0u64; to_width * to_height * 4];
    let mut counts = vec![0u64; to_width * to_height];
    for y in 0..height {
        let row = y * to_height / height;
        for x in 0..width {
            let target = row * to_width + x * to_width / width;
            counts[target] += 1;
            for channel in 0..4 {
                sums[target * 4 + channel] += u64::from(pixels[(y * width + x) * 4 + channel]);
            }
        }
    }
    sums.iter()
        .enumerate()
        .map(|(index, sum)| (sum / counts[index / 4].max(1)) as u8)
        .collect()
}

/// One horizontal or vertical box blur pass over RGBA pixels.
pub fn blur_pass(pixels: &[u8], width: usize, height: usize, radius: usize, horizontal: bool) -> Vec<u8> {
    let mut out = vec![0u8; pixels.len()];
    let (lines, length) = if horizontal { (height, width) } else { (width, height) };
    let index = |line: usize, at: usize| {
        if horizontal {
            (line * width + at) * 4
        } else {
            (at * width + line) * 4
        }
    };
    for line in 0..lines {
        for at in 0..length {
            let low = at.saturating_sub(radius);
            let high = (at + radius).min(length - 1);
            let count = (high - low + 1) as u32;
            for channel in 0..4 {
                let sum: u32 = (low..=high).map(|k| u32::from(pixels[index(line, k) + channel])).sum();
                out[index(line, at) + channel] = (sum / count) as u8;
            }
        }
    }
    out
}

/// The texture GTK draws from blurred pixels.
pub fn texture(blurred: Blurred) -> gdk::MemoryTexture {
    gdk::MemoryTexture::new(
        blurred.width as i32,
        blurred.height as i32,
        gdk::MemoryFormat::B8g8r8a8Premultiplied,
        &glib::Bytes::from_owned(blurred.pixels),
        blurred.width * 4,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrinking_averages_blocks() {
        let mut pixels = vec![0u8; 4 * 2 * 4];
        pixels[0] = 100;
        pixels[4] = 50;
        pixels[8] = 30;
        pixels[12] = 20;
        assert_eq!(shrink(&pixels, 4, 2, 2, 1)[0], 37);
        assert_eq!(shrink(&pixels, 4, 2, 2, 1)[4], 12);
        assert_eq!(shrink(&pixels, 4, 2, 4, 2), pixels);
    }

    #[test]
    fn a_box_blur_averages_and_keeps_flat_areas_flat() {
        let flat = vec![100u8; 4 * 4 * 4];
        assert_eq!(blur_pass(&flat, 4, 4, 1, true), flat);
        let mut spike = vec![0u8; 3 * 4];
        spike[4] = 90;
        let blurred = blur_pass(&spike, 3, 1, 1, true);
        assert_eq!(blurred[0], 45);
        assert_eq!(blurred[4], 30);
        assert_eq!(blurred[8], 45);
        let column = blur_pass(&spike, 1, 3, 1, false);
        assert_eq!(column[4], 30);
    }
}
