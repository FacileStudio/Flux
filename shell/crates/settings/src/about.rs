//! The About page: the Flux mark and release, this machine's hardware, and links.

mod info;

use std::path::Path;

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib, Orientation};
use shell_common::{palette, theme, ui};

use crate::{i18n, preview};
use info::Details;

pub const ID: &str = "about";
pub const ICON: &str = "help-about-symbolic";

const MARK: &str = include_str!("../data/flux-mark.svg");
const MARK_SIZE: i32 = 64;
const SITE: &str = "https://flux.facile.studio";
const DOCS: &str = "https://flux.facile.studio/docs";
const ISSUES: &str = "https://github.com/FacileStudio/Flux/issues";

/// The page's name in the navigation.
pub fn title() -> String {
    i18n::text().about_title.to_string()
}

/// What a preview shows in place of this machine.
fn sample() -> Details {
    Details {
        system: Some("Flux 0.11.1".to_string()),
        version: Some("0.11.1".to_string()),
        hostname: Some("ada-flux".to_string()),
        processor: Some("AMD Ryzen 7 7840U w/ Radeon 780M Graphics × 16".to_string()),
        memory: Some(32 * 1024 * 1024 * 1024),
        disk: Some(1_000_204_886_016),
        graphics: Some("AMD Radeon 780M".to_string()),
        kernel: Some("6.12.63".to_string()),
    }
}

/// The mark SVG in `color`, rendered large enough to stay sharp at twice the scale.
fn mark_svg(color: &gdk::RGBA) -> String {
    let hex = |channel: f32| (channel.clamp(0.0, 1.0) * 255.0).round() as u8;
    let fill = format!(
        "#{:02x}{:02x}{:02x}",
        hex(color.red()),
        hex(color.green()),
        hex(color.blue())
    );
    let size = MARK_SIZE * 4;
    MARK.replace("currentColor", &fill).replace(
        "width=\"24\" height=\"24\"",
        &format!("width=\"{size}\" height=\"{size}\""),
    )
}

/// Draw the mark into `image` in the palette's text colour, decoding off the main thread.
fn paint_mark(image: &gtk4::Image) {
    let svg = mark_svg(&palette::fg_rgba().unwrap_or(gdk::RGBA::BLACK));
    let image = image.clone();
    glib::spawn_future_local(async move {
        let decoded = gio::spawn_blocking(move || {
            gdk::Texture::from_bytes(&glib::Bytes::from_owned(svg.into_bytes())).ok()
        })
        .await;
        if let Ok(Some(texture)) = decoded {
            image.set_paintable(Some(&texture));
        }
    });
}

fn mark() -> gtk4::Image {
    let image = gtk4::Image::new();
    image.add_css_class("fx-mark");
    image.set_pixel_size(MARK_SIZE);
    image.set_valign(gtk4::Align::Center);
    paint_mark(&image);
    let weak = image.downgrade();
    theme::on_reload(move || {
        if let Some(image) = weak.upgrade() {
            paint_mark(&image);
        }
    });
    image
}

/// `value`, or "Unknown".
fn or_unknown(value: Option<String>) -> String {
    value.unwrap_or_else(|| i18n::text().unknown.to_string())
}

/// The plain-text summary "Copy details" puts on the clipboard.
fn summary(details: &Details) -> String {
    let text = i18n::text();
    [
        (text.version, or_unknown(details.system.clone())),
        (text.hostname, or_unknown(details.hostname.clone())),
        (text.processor, or_unknown(details.processor.clone())),
        (
            text.memory,
            or_unknown(details.memory.map(info::format_bytes)),
        ),
        (text.disk, or_unknown(details.disk.map(info::format_bytes))),
        (text.graphics, or_unknown(details.graphics.clone())),
        (text.kernel, or_unknown(details.kernel.clone())),
    ]
    .iter()
    .map(|(label, value)| format!("{label}: {value}"))
    .collect::<Vec<_>>()
    .join("\n")
}

fn value(text: &str) -> gtk4::Label {
    let label = gtk4::Label::new(Some(text));
    label.add_css_class("fx-value");
    label.set_selectable(true);
    label.set_wrap(true);
    label.set_xalign(1.0);
    label.set_justify(gtk4::Justification::Right);
    label.set_max_width_chars(40);
    label.set_natural_wrap_mode(gtk4::NaturalWrapMode::None);
    label
}

/// The card at the top: the mark, the release, and "Copy details".
fn hero(details: &Details) -> gtk4::Box {
    let text = i18n::text();
    let (root, card) = ui::section("");
    let line = gtk4::Box::new(Orientation::Horizontal, 20);
    line.append(&mark());
    let labels = gtk4::Box::new(Orientation::Vertical, 4);
    labels.set_hexpand(true);
    labels.set_valign(gtk4::Align::Center);
    let name = gtk4::Label::new(Some(&or_unknown(details.system.clone())));
    name.add_css_class("fx-hero-title");
    name.set_xalign(0.0);
    labels.append(&name);
    let release = gtk4::Label::new(Some(&format!(
        "{} {}",
        text.version,
        or_unknown(details.version.clone())
    )));
    release.add_css_class("fx-subtitle");
    release.set_xalign(0.0);
    labels.append(&release);
    line.append(&labels);
    let copy = ui::secondary_button(text.copy_details);
    copy.set_valign(gtk4::Align::Center);
    let report = summary(details);
    copy.connect_clicked(move |button| {
        button.clipboard().set_text(&report);
        button.set_label(i18n::text().copied);
        let button = button.clone();
        glib::timeout_add_local_once(std::time::Duration::from_secs(2), move || {
            button.set_label(i18n::text().copy_details);
        });
    });
    line.append(&copy);
    card.append(&line);
    root
}

fn facts(title: &str, rows: &[(&str, String)]) -> gtk4::Box {
    let (root, card) = ui::section(title);
    for (label, text) in rows {
        card.append(&ui::row(label, "", &value(text)));
    }
    root
}

fn link_row(title: &str, subtitle: &str, uri: &'static str) -> gtk4::Box {
    let button = ui::secondary_button(i18n::text().open);
    button.update_property(&[gtk4::accessible::Property::Label(title)]);
    button.connect_clicked(move |button| {
        let window = button.root().and_downcast::<gtk4::Window>();
        gtk4::UriLauncher::new(uri).launch(window.as_ref(), gio::Cancellable::NONE, |_| {});
    });
    ui::row(title, subtitle, &button)
}

fn fill(column: &gtk4::Box, details: &Details) {
    let text = i18n::text();
    column.append(&hero(details));
    column.append(&facts(
        text.system,
        &[
            (text.hostname, or_unknown(details.hostname.clone())),
            (text.kernel, or_unknown(details.kernel.clone())),
        ],
    ));
    column.append(&facts(
        text.hardware,
        &[
            (text.processor, or_unknown(details.processor.clone())),
            (
                text.memory,
                or_unknown(details.memory.map(info::format_bytes)),
            ),
            (text.disk, or_unknown(details.disk.map(info::format_bytes))),
            (text.graphics, or_unknown(details.graphics.clone())),
        ],
    ));
    let (links, card) = ui::section(text.links);
    card.append(&link_row(text.website, text.website_body, SITE));
    card.append(&link_row(text.docs, text.docs_body, DOCS));
    card.append(&link_row(text.report_bug, text.report_bug_body, ISSUES));
    column.append(&links);
}

/// The whole page. The details are read off the main thread behind a spinner.
pub fn page() -> gtk4::Widget {
    let text = i18n::text();
    let (root, column) = ui::page(text.about_title, text.about_description);
    if preview::active() {
        fill(&column, &sample());
        return root.upcast();
    }
    let waiting = ui::spinner_row(text.loading);
    column.append(&waiting);
    glib::spawn_future_local(async move {
        let details = gio::spawn_blocking(|| info::gather(Path::new("/")))
            .await
            .unwrap_or_default();
        column.remove(&waiting);
        fill(&column, &details);
    });
    root.upcast()
}
