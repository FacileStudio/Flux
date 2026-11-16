//! Wallpaper thumbnails: decoded from disk off the main thread, or drawn as
//! sample gradients in a preview.

use std::path::PathBuf;

use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::{gdk, gio, glib};

pub const THUMB_WIDTH: i32 = 152;
pub const THUMB_HEIGHT: i32 = 96;

/// A soft two-colour gradient standing in for an image in a preview.
pub fn sample_texture(index: usize) -> gdk::MemoryTexture {
    const PAIRS: [([u8; 3], [u8; 3]); 6] = [
        ([38, 38, 46], [120, 112, 150]),
        ([226, 214, 196], [150, 128, 108]),
        ([24, 52, 60], [92, 150, 150]),
        ([238, 238, 238], [176, 176, 182]),
        ([60, 28, 40], [190, 98, 108]),
        ([30, 44, 30], [128, 160, 96]),
    ];
    let (from, to) = PAIRS[index % PAIRS.len()];
    let (width, height) = (THUMB_WIDTH as usize, THUMB_HEIGHT as usize);
    let mut bytes = Vec::with_capacity(width * height * 3);
    for y in 0..height {
        for x in 0..width {
            let t = (x + y) as f32 / (width + height) as f32;
            bytes.extend((0..3).map(|channel| {
                (f32::from(from[channel]) + (f32::from(to[channel]) - f32::from(from[channel])) * t)
                    as u8
            }));
        }
    }
    gdk::MemoryTexture::new(
        THUMB_WIDTH,
        THUMB_HEIGHT,
        gdk::MemoryFormat::R8g8b8,
        &glib::Bytes::from_owned(bytes),
        width * 3,
    )
}

/// Decode `path` at twice the tile size on the blocking pool. A pixbuf cannot cross threads, so the worker returns pixels.
pub fn load_thumbnail(picture: &gtk4::Picture, path: PathBuf) {
    let picture = picture.clone();
    glib::spawn_future_local(async move {
        let decoded = gio::spawn_blocking(move || {
            let pixbuf =
                Pixbuf::from_file_at_scale(&path, THUMB_WIDTH * 2, THUMB_HEIGHT * 2, true).ok()?;
            let format = if pixbuf.has_alpha() {
                gdk::MemoryFormat::R8g8b8a8
            } else {
                gdk::MemoryFormat::R8g8b8
            };
            let stride = usize::try_from(pixbuf.rowstride()).ok()?;
            Some((
                pixbuf.read_pixel_bytes(),
                pixbuf.width(),
                pixbuf.height(),
                stride,
                format,
            ))
        })
        .await;
        if let Ok(Some((bytes, width, height, stride, format))) = decoded {
            picture.set_paintable(Some(&gdk::MemoryTexture::new(
                width, height, format, &bytes, stride,
            )));
        }
    });
}
