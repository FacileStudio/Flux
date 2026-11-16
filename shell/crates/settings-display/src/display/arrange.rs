//! The arrangement canvas: one rounded plate per enabled output, drawn to
//! scale, that the person drags into place. A drop snaps the plate flush
//! against its neighbours with [`place`] and reports every output's new
//! logical position.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{DrawingArea, Fixed, Label, Overlay};

use crate::model::geometry::{bounds, fit, normalize, place, Rect};

/// How far, in logical pixels, an edge is pulled to line up with another.
const SNAP: i32 = 96;
const HEIGHT: i32 = 220;
const PADDING: f64 = 24.0;

/// One output on the canvas: its niri identity, label and logical rectangle.
#[derive(Clone)]
pub struct Plate {
    pub identity: String,
    pub label: String,
    pub rect: Rect,
}

struct Canvas {
    fixed: Fixed,
    plates: RefCell<Vec<Plate>>,
    widgets: RefCell<Vec<gtk4::Box>>,
    view: Cell<(f64, f64, f64)>,
    dragging: Cell<Option<(usize, f64, f64)>>,
}

impl Canvas {
    fn to_canvas(&self, rect: &Rect) -> (f64, f64, f64, f64) {
        let (factor, ox, oy) = self.view.get();
        (
            ox + f64::from(rect.x) * factor,
            oy + f64::from(rect.y) * factor,
            f64::from(rect.width) * factor,
            f64::from(rect.height) * factor,
        )
    }

    fn layout(&self, width: i32, height: i32) {
        let plates = self.plates.borrow();
        let rects: Vec<Rect> = plates.iter().map(|plate| plate.rect).collect();
        let Some(whole) = bounds(&rects) else {
            return;
        };
        self.view.set(fit(&whole, f64::from(width), f64::from(height), PADDING));
        for (plate, widget) in plates.iter().zip(self.widgets.borrow().iter()) {
            let (x, y, w, h) = self.to_canvas(&plate.rect);
            widget.set_size_request((w - 4.0).max(8.0) as i32, (h - 4.0).max(8.0) as i32);
            self.fixed.move_(widget, x + 2.0, y + 2.0);
        }
    }

    fn hit(&self, x: f64, y: f64) -> Option<usize> {
        self.plates.borrow().iter().position(|plate| {
            let (px, py, w, h) = self.to_canvas(&plate.rect);
            x >= px && x <= px + w && y >= py && y <= py + h
        })
    }

    fn drag_to(&self, index: usize, dx: f64, dy: f64) {
        let plates = self.plates.borrow();
        let (Some(plate), Some(widget)) = (plates.get(index), self.widgets.borrow().get(index).cloned()) else {
            return;
        };
        let (x, y, _, _) = self.to_canvas(&plate.rect);
        self.fixed.move_(&widget, x + 2.0 + dx, y + 2.0 + dy);
    }

    fn drop_at(&self, index: usize, dx: f64, dy: f64) -> Option<Vec<Plate>> {
        let (factor, _, _) = self.view.get();
        if factor <= 0.0 {
            return None;
        }
        let mut plates = self.plates.borrow_mut();
        let moving = plates.get(index)?.rect;
        let dropped = moving.at(moving.x + (dx / factor).round() as i32, moving.y + (dy / factor).round() as i32);
        let others: Vec<Rect> =
            plates.iter().enumerate().filter(|(i, _)| *i != index).map(|(_, plate)| plate.rect).collect();
        let (x, y) = place(&dropped, &others, SNAP);
        plates[index].rect = moving.at(x, y);
        let mut rects: Vec<Rect> = plates.iter().map(|plate| plate.rect).collect();
        normalize(&mut rects);
        for (plate, rect) in plates.iter_mut().zip(rects) {
            plate.rect = rect;
        }
        Some(plates.clone())
    }
}

fn plate_widget(plate: &Plate) -> gtk4::Box {
    let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    widget.add_css_class("fx-monitor");
    let label = Label::builder()
        .label(&plate.label)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .valign(gtk4::Align::Center)
        .vexpand(true)
        .build();
    widget.append(&label);
    widget
}

/// The canvas for `plates`. `on_drop` receives every plate with its new
/// position after a drag ends.
pub fn canvas(plates: Vec<Plate>, on_drop: impl Fn(Vec<Plate>) + 'static) -> gtk4::Widget {
    let fixed = Fixed::new();
    let widgets: Vec<gtk4::Box> = plates.iter().map(plate_widget).collect();
    for widget in &widgets {
        fixed.put(widget, 0.0, 0.0);
    }
    let state = Rc::new(Canvas {
        fixed: fixed.clone(),
        plates: RefCell::new(plates),
        widgets: RefCell::new(widgets),
        view: Cell::new((0.0, 0.0, 0.0)),
        dragging: Cell::new(None),
    });
    let area = DrawingArea::builder().content_height(HEIGHT).hexpand(true).build();
    let resized = state.clone();
    area.connect_resize(move |_, width, height| resized.layout(width, height));
    let overlay = Overlay::builder().child(&area).css_classes(["fx-canvas"]).build();
    overlay.add_overlay(&fixed);

    let gesture = gtk4::GestureDrag::new();
    let begin = state.clone();
    gesture.connect_drag_begin(move |_, x, y| {
        let index = begin.hit(x, y);
        begin.dragging.set(index.map(|i| (i, 0.0, 0.0)));
        if let Some(widget) = index.and_then(|i| begin.widgets.borrow().get(i).cloned()) {
            widget.add_css_class("dragging");
        }
    });
    let update = state.clone();
    gesture.connect_drag_update(move |_, dx, dy| {
        if let Some((index, _, _)) = update.dragging.get() {
            update.dragging.set(Some((index, dx, dy)));
            update.drag_to(index, dx, dy);
        }
    });
    let end = state.clone();
    let area_end = area.clone();
    gesture.connect_drag_end(move |_, dx, dy| {
        let Some((index, _, _)) = end.dragging.take() else {
            return;
        };
        if let Some(widget) = end.widgets.borrow().get(index) {
            widget.remove_css_class("dragging");
        }
        let moved = dx.abs() > 2.0 || dy.abs() > 2.0;
        let result = if moved { end.drop_at(index, dx, dy) } else { None };
        end.layout(area_end.width(), area_end.height());
        if let Some(plates) = result {
            on_drop(plates);
        }
    });
    overlay.add_controller(gesture);
    overlay.upcast()
}
