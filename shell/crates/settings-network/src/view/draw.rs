//! Small glyphs drawn in the current text colour, so the page needs no icon theme.

use std::cell::Cell;
use std::f64::consts::{FRAC_PI_2, PI};
use std::rc::Rc;

use gtk4::cairo::Context;
use gtk4::prelude::*;
use gtk4::DrawingArea;

fn area(size: i32, class: &str) -> DrawingArea {
    let area = DrawingArea::new();
    area.add_css_class(class);
    area.set_content_width(size);
    area.set_content_height(size);
    area.set_valign(gtk4::Align::Center);
    area
}

fn source(cr: &Context, area: &DrawingArea, alpha: f64) {
    let color = area.color();
    cr.set_source_rgba(
        f64::from(color.red()),
        f64::from(color.green()),
        f64::from(color.blue()),
        f64::from(color.alpha()) * alpha,
    );
}

fn rounded(cr: &Context, x: f64, y: f64, width: f64, height: f64, radius: f64) {
    let radius = radius.min(width / 2.0).min(height / 2.0);
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
    cr.arc(x + radius, y + radius, radius, PI, 3.0 * FRAC_PI_2);
    cr.close_path();
}

/// Four rising bars, the first `level` of them filled.
pub fn bars(level: Rc<Cell<u8>>) -> DrawingArea {
    let area = area(18, "fx-bars");
    area.set_draw_func(move |area, cr, width, height| {
        let (width, height) = (f64::from(width), f64::from(height));
        let gap = width * 0.1;
        let bar = (width - 3.0 * gap) / 4.0;
        for index in 0..4u8 {
            let share = f64::from(index + 1) / 4.0;
            let top = height * (1.0 - share * 0.85) - height * 0.05;
            let x = f64::from(index) * (bar + gap);
            source(cr, area, if index < level.get() { 1.0 } else { 0.22 });
            rounded(cr, x, top, bar, height * 0.95 - top, bar / 2.0);
            let _ = cr.fill();
        }
    });
    area
}

/// A padlock.
pub fn lock() -> DrawingArea {
    let area = area(14, "fx-lock");
    area.set_draw_func(|area, cr, width, height| {
        let (width, height) = (f64::from(width), f64::from(height));
        source(cr, area, 1.0);
        cr.set_line_width(width * 0.13);
        cr.arc(width / 2.0, height * 0.42, width * 0.24, PI, 0.0);
        cr.move_to(width * 0.26, height * 0.42);
        cr.line_to(width * 0.26, height * 0.5);
        cr.move_to(width * 0.74, height * 0.42);
        cr.line_to(width * 0.74, height * 0.5);
        let _ = cr.stroke();
        rounded(
            cr,
            width * 0.14,
            height * 0.48,
            width * 0.72,
            height * 0.46,
            width * 0.12,
        );
        let _ = cr.fill();
    });
    area
}

/// A chevron that points down, or up while `open`.
pub fn chevron(open: Rc<Cell<bool>>) -> DrawingArea {
    let area = area(12, "fx-chevron");
    area.set_halign(gtk4::Align::Center);
    area.set_draw_func(move |area, cr, width, height| {
        let side = f64::from(width.min(height));
        cr.translate(
            (f64::from(width) - side) / 2.0,
            (f64::from(height) - side) / 2.0,
        );
        let (width, height) = (side, side);
        let (from, to) = if open.get() {
            (0.64, 0.36)
        } else {
            (0.36, 0.64)
        };
        source(cr, area, 1.0);
        cr.set_line_width(1.6);
        cr.set_line_cap(gtk4::cairo::LineCap::Round);
        cr.set_line_join(gtk4::cairo::LineJoin::Round);
        cr.move_to(width * 0.22, height * from);
        cr.line_to(width * 0.5, height * to);
        cr.line_to(width * 0.78, height * from);
        let _ = cr.stroke();
    });
    area
}
