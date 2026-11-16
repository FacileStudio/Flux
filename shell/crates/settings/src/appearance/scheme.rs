//! The two theme tiles. Each draws a small window in its scheme's own palette,
//! read from `/etc/flux/palette-<scheme>.css`, and choosing one runs `flux-scheme`.

use std::f64::consts::{FRAC_PI_2, PI};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib, Orientation};
use shell_common::scheme::{self, Scheme};
use shell_common::{theme, ui};

use crate::{i18n, preview};

/// The palette colours a tile draws with.
struct Swatch {
    page: gdk::RGBA,
    component: gdk::RGBA,
    surface: gdk::RGBA,
    fg: gdk::RGBA,
    muted: gdk::RGBA,
}

/// muse's values, for a system without the palette files.
const FALLBACK_LIGHT: [&str; 5] = ["#ffffff", "#f7f7f7", "#eeeeee", "#0a0a0a", "#636363"];
const FALLBACK_DARK: [&str; 5] = ["#020202", "#0d0d0d", "#181818", "#fafafa", "#8c8c8c"];
const TOKENS: [&str; 5] = [
    "fc_page",
    "fc_component",
    "fc_surface",
    "fc_fg",
    "fc_fg_muted",
];

/// The value of the last `@define-color <name> <value>;` in `sheet`, the one GTK keeps.
pub fn color_in(sheet: &str, name: &str) -> Option<String> {
    let prefix = format!("@define-color {name} ");
    sheet.lines().rev().find_map(|line| {
        let value = line
            .trim()
            .strip_prefix(&prefix)?
            .trim()
            .trim_end_matches(';')
            .trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn swatch(scheme: Scheme) -> Swatch {
    let sheet = std::fs::read_to_string(format!("/etc/flux/palette-{}.css", scheme.name()))
        .unwrap_or_default();
    let fallback = match scheme {
        Scheme::Light => FALLBACK_LIGHT,
        Scheme::Dark => FALLBACK_DARK,
    };
    let colors: Vec<gdk::RGBA> = TOKENS
        .iter()
        .zip(fallback)
        .map(|(token, fallback)| {
            color_in(&sheet, token)
                .and_then(|value| gdk::RGBA::parse(value.as_str()).ok())
                .or_else(|| gdk::RGBA::parse(fallback).ok())
                .unwrap_or(gdk::RGBA::BLACK)
        })
        .collect();
    Swatch {
        page: colors[0],
        component: colors[1],
        surface: colors[2],
        fg: colors[3],
        muted: colors[4],
    }
}

fn rounded(cr: &gtk4::cairo::Context, (x, y, width, height): (f64, f64, f64, f64), radius: f64) {
    cr.new_sub_path();
    cr.arc(x + width - radius, y + radius, radius, -FRAC_PI_2, 0.0);
    cr.arc(
        x + width - radius,
        y + height - radius,
        radius,
        0.0,
        FRAC_PI_2,
    );
    cr.arc(x + radius, y + height - radius, radius, FRAC_PI_2, PI);
    cr.arc(x + radius, y + radius, radius, PI, PI + FRAC_PI_2);
    cr.close_path();
}

fn fill(cr: &gtk4::cairo::Context, color: &gdk::RGBA) {
    cr.set_source_rgba(
        color.red().into(),
        color.green().into(),
        color.blue().into(),
        color.alpha().into(),
    );
    let _ = cr.fill();
}

/// A miniature settings window: a rail with rows, a title, and a card holding two rows.
fn draw(cr: &gtk4::cairo::Context, width: f64, height: f64, swatch: &Swatch) {
    rounded(cr, (0.0, 0.0, width, height), 12.0);
    fill(cr, &swatch.page);
    rounded(cr, (0.0, 0.0, width * 0.3, height), 12.0);
    cr.rectangle(width * 0.3 - 12.0, 0.0, 12.0, height);
    fill(cr, &swatch.component);
    rounded(cr, (10.0, 16.0, width * 0.3 - 20.0, 12.0), 4.0);
    fill(cr, &swatch.fg);
    for row in 0..3 {
        rounded(
            cr,
            (14.0, 38.0 + f64::from(row) * 16.0, width * 0.3 - 34.0, 6.0),
            3.0,
        );
        fill(cr, &swatch.muted);
    }
    let left = width * 0.3 + 16.0;
    let inner = width - left - 16.0;
    rounded(cr, (left, 18.0, inner * 0.45, 10.0), 4.0);
    fill(cr, &swatch.fg);
    rounded(cr, (left, 40.0, inner, height - 56.0), 8.0);
    fill(cr, &swatch.component);
    for row in 0..2 {
        let y = 54.0 + f64::from(row) * 26.0;
        rounded(cr, (left + 12.0, y, inner * 0.4, 6.0), 3.0);
        fill(cr, &swatch.muted);
        rounded(cr, (left + inner - 40.0, y - 4.0, 28.0, 14.0), 7.0);
        fill(
            cr,
            if row == 0 {
                &swatch.fg
            } else {
                &swatch.surface
            },
        );
    }
}

/// The scheme to show as chosen: the portal's, or the preview's.
fn chosen() -> Scheme {
    match preview::current() {
        Some(preview) if preview.dark => Scheme::Dark,
        Some(_) => Scheme::Light,
        None => scheme::current().unwrap_or(Scheme::Light),
    }
}

fn tile(scheme: Scheme, label: &str) -> gtk4::Button {
    let swatch = swatch(scheme);
    let area = gtk4::DrawingArea::new();
    area.add_css_class("fx-theme-preview");
    area.set_content_width(260);
    area.set_content_height(150);
    area.set_draw_func(move |_, cr, width, height| draw(cr, width.into(), height.into(), &swatch));
    let column = gtk4::Box::new(Orientation::Vertical, 12);
    column.append(&area);
    let name = gtk4::Label::new(Some(label));
    name.add_css_class("fx-theme-label");
    name.set_xalign(0.0);
    column.append(&name);
    let button = gtk4::Button::new();
    button.add_css_class("fx-theme");
    button.set_child(Some(&column));
    button.update_property(&[gtk4::accessible::Property::Label(label)]);
    button
}

/// Ring the tile of `scheme`, and only that one.
fn mark(tiles: &[(Scheme, gtk4::Button)], scheme: Scheme) {
    for (candidate, button) in tiles {
        if *candidate == scheme {
            button.add_css_class("selected");
        } else {
            button.remove_css_class("selected");
        }
    }
}

/// Switch to `scheme` with `flux-scheme`, off the main thread. The portal then restyles every window.
fn apply(tiles: Rc<Vec<(Scheme, gtk4::Button)>>, scheme: Scheme) {
    mark(&tiles, scheme);
    if preview::active() {
        return;
    }
    glib::spawn_future_local(async move {
        let outcome =
            gio::spawn_blocking(move || super::run("flux-scheme", &[scheme.name()])).await;
        if !matches!(outcome, Ok(Ok(_))) {
            mark(&tiles, chosen());
        }
    });
}

/// The Theme section.
pub fn section() -> gtk4::Box {
    let text = i18n::text();
    let (root, card) = ui::section(text.theme);
    let line = gtk4::Box::new(Orientation::Horizontal, 20);
    line.set_homogeneous(true);
    let tiles: Rc<Vec<(Scheme, gtk4::Button)>> = Rc::new(vec![
        (Scheme::Light, tile(Scheme::Light, text.light)),
        (Scheme::Dark, tile(Scheme::Dark, text.dark)),
    ]);
    for (scheme, button) in tiles.iter() {
        line.append(button);
        let tiles = tiles.clone();
        let scheme = *scheme;
        button.connect_clicked(move |_| apply(tiles.clone(), scheme));
    }
    mark(&tiles, chosen());
    let weak = Rc::downgrade(&tiles);
    theme::on_reload(move || {
        if let Some(tiles) = weak.upgrade() {
            mark(&tiles, chosen());
        }
    });
    card.append(&line);
    root
}

#[cfg(test)]
mod tests {
    use super::color_in;

    #[test]
    fn the_last_definition_of_a_colour_wins() {
        let sheet = "@define-color fc_page #ffffff;\n@define-color fc_fg #0a0a0a;\n@define-color fc_page #020202;\n";
        assert_eq!(color_in(sheet, "fc_page").as_deref(), Some("#020202"));
        assert_eq!(color_in(sheet, "fc_fg").as_deref(), Some("#0a0a0a"));
        assert_eq!(color_in(sheet, "fc_ring"), None);
    }
}
