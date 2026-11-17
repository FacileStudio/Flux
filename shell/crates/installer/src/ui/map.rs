//! The world map of the time zone screen: land from Natural Earth (public domain), one dot per
//! zone at its main city, the chosen zone marked. A click picks the nearest city.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{cairo, DrawingArea, GestureClick};

use crate::model::ZonePoint;

const WORLD: &str = include_str!("../../data/world.txt");
const NORTH: f64 = 84.0;
const SOUTH: f64 = -58.0;

/// The land outlines, each a ring of (longitude, latitude).
fn land() -> Vec<Vec<(f64, f64)>> {
    WORLD
        .lines()
        .map(|line| {
            line.split_whitespace()
                .filter_map(|pair| {
                    let (lon, lat) = pair.split_once(',')?;
                    Some((lon.parse().ok()?, lat.parse().ok()?))
                })
                .collect()
        })
        .filter(|ring: &Vec<(f64, f64)>| ring.len() >= 3)
        .collect()
}

/// Maps longitude and latitude to widget pixels, keeping the map's proportions.
#[derive(Clone, Copy)]
struct Projection {
    scale: f64,
    left: f64,
    top: f64,
}

impl Projection {
    fn new(width: f64, height: f64) -> Self {
        let scale = (width / 360.0).min(height / (NORTH - SOUTH));
        Projection {
            scale,
            left: (width - 360.0 * scale) / 2.0,
            top: (height - (NORTH - SOUTH) * scale) / 2.0,
        }
    }

    fn point(self, longitude: f64, latitude: f64) -> (f64, f64) {
        (self.left + (longitude + 180.0) * self.scale, self.top + (NORTH - latitude) * self.scale)
    }
}

/// The zone whose city lies nearest to the pixel (`x`, `y`).
fn nearest(points: &[ZonePoint], projection: Projection, x: f64, y: f64) -> Option<String> {
    points
        .iter()
        .map(|point| {
            let (px, py) = projection.point(point.longitude, point.latitude);
            ((px - x).powi(2) + (py - y).powi(2), &point.zone)
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, zone)| zone.clone())
}

fn draw(cr: &cairo::Context, area: &DrawingArea, rings: &[Vec<(f64, f64)>], points: &[ZonePoint], chosen: &str) {
    let projection = Projection::new(f64::from(area.width()), f64::from(area.height()));
    let ink = area.color();
    let (red, green, blue) = (f64::from(ink.red()), f64::from(ink.green()), f64::from(ink.blue()));
    cr.set_fill_rule(cairo::FillRule::EvenOdd);
    for ring in rings {
        for (index, (lon, lat)) in ring.iter().enumerate() {
            let (x, y) = projection.point(*lon, *lat);
            if index == 0 {
                cr.move_to(x, y);
            } else {
                cr.line_to(x, y);
            }
        }
        cr.close_path();
    }
    cr.set_source_rgba(red, green, blue, 0.12);
    let _ = cr.fill();
    cr.set_source_rgba(red, green, blue, 0.32);
    for point in points {
        let (x, y) = projection.point(point.longitude, point.latitude);
        cr.arc(x, y, 1.6, 0.0, std::f64::consts::TAU);
        let _ = cr.fill();
    }
    if let Some(point) = points.iter().find(|point| point.zone == chosen) {
        let (x, y) = projection.point(point.longitude, point.latitude);
        cr.set_source_rgba(red, green, blue, 0.22);
        cr.arc(x, y, 13.0, 0.0, std::f64::consts::TAU);
        let _ = cr.fill();
        cr.set_source_rgba(red, green, blue, 1.0);
        cr.arc(x, y, 5.5, 0.0, std::f64::consts::TAU);
        let _ = cr.fill();
    }
}

/// The map, drawing `chosen` as the marked zone and calling `pick` with the zone a click lands on.
pub fn build(points: Rc<Vec<ZonePoint>>, chosen: Rc<RefCell<String>>, pick: impl Fn(String) + 'static) -> DrawingArea {
    let area = DrawingArea::new();
    area.add_css_class("map");
    area.set_content_height(200);
    area.set_hexpand(true);
    let rings = Rc::new(land());
    let (drawn, marked) = (points.clone(), chosen.clone());
    area.set_draw_func(move |area, cr, _, _| draw(cr, area, &rings, &drawn, &marked.borrow()));
    let click = GestureClick::new();
    click.connect_pressed(move |gesture, _, x, y| {
        let Some(widget) = gesture.widget() else {
            return;
        };
        let projection = Projection::new(f64::from(widget.width()), f64::from(widget.height()));
        if let Some(zone) = nearest(&points, projection, x, y) {
            pick(zone);
            widget.queue_draw();
        }
    });
    area.add_controller(click);
    area
}

#[cfg(test)]
mod tests;
