//! Arrangement math: where a dragged output lands next to the others.
//!
//! niri places outputs in logical pixels, refuses overlaps by moving the
//! offender to the right of everything, and only lets the cursor cross
//! between outputs that touch. So a drop always ends touching at least one
//! other output on a full edge, never overlapping, with near-aligned edges
//! pulled flush.

/// A rectangle in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Rect {
        Rect { x, y, width: width.max(1), height: height.max(1) }
    }

    pub fn right(&self) -> i32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height
    }

    pub fn at(&self, x: i32, y: i32) -> Rect {
        Rect { x, y, ..*self }
    }

    /// Whether the two share any area. Touching edges do not overlap.
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.x < other.right() && other.x < self.right() && self.y < other.bottom() && other.y < self.bottom()
    }

    /// Whether the two share a stretch of edge, which is what lets the
    /// cursor cross.
    #[cfg(test)]
    pub fn touches(&self, other: &Rect) -> bool {
        let side = (self.right() == other.x || other.right() == self.x)
            && self.y < other.bottom()
            && other.y < self.bottom();
        let stacked = (self.bottom() == other.y || other.bottom() == self.y)
            && self.x < other.right()
            && other.x < self.right();
        side || stacked
    }
}

/// Pull `value` onto the nearest of `targets` within `threshold`.
fn snap_axis(value: i32, targets: &[i32], threshold: i32) -> i32 {
    targets
        .iter()
        .copied()
        .filter(|target| (target - value).abs() <= threshold)
        .min_by_key(|target| (target - value).abs())
        .unwrap_or(value)
}

/// The positions that put `moving` flush against one side of `other`, with
/// the free axis kept where it was dropped, clamped so the two still share an
/// edge, and snapped to aligned starts, ends or centres within `threshold`.
fn candidates(moving: &Rect, other: &Rect, threshold: i32) -> [(i32, i32); 4] {
    let y_targets = [
        other.y,
        other.bottom() - moving.height,
        other.y + (other.height - moving.height) / 2,
    ];
    let x_targets = [
        other.x,
        other.right() - moving.width,
        other.x + (other.width - moving.width) / 2,
    ];
    let free_y = snap_axis(moving.y, &y_targets, threshold).clamp(other.y - moving.height + 1, other.bottom() - 1);
    let free_x = snap_axis(moving.x, &x_targets, threshold).clamp(other.x - moving.width + 1, other.right() - 1);
    [
        (other.x - moving.width, free_y),
        (other.right(), free_y),
        (free_x, other.y - moving.height),
        (free_x, other.bottom()),
    ]
}

/// Where `moving`, dropped at its own `x, y`, should land among `others`:
/// the closest spot that touches one of them and overlaps none. With nothing
/// else on screen it goes to the origin.
pub fn place(moving: &Rect, others: &[Rect], threshold: i32) -> (i32, i32) {
    if others.is_empty() {
        return (0, 0);
    }
    let distance = |(x, y): (i32, i32)| {
        let dx = i64::from(x - moving.x);
        let dy = i64::from(y - moving.y);
        dx * dx + dy * dy
    };
    others
        .iter()
        .flat_map(|other| candidates(moving, other, threshold))
        .filter(|&(x, y)| {
            let spot = moving.at(x, y);
            others.iter().all(|other| !spot.overlaps(other))
        })
        .min_by_key(|&spot| distance(spot))
        .unwrap_or_else(|| {
            let right = others.iter().map(Rect::right).max().unwrap_or(0);
            let top = others.iter().map(|other| other.y).min().unwrap_or(0);
            (right, top)
        })
}

/// Shift every rectangle so the arrangement starts at the origin, which keeps
/// the numbers written to the config small and stable.
pub fn normalize(rects: &mut [Rect]) {
    let Some(min_x) = rects.iter().map(|r| r.x).min() else {
        return;
    };
    let min_y = rects.iter().map(|r| r.y).min().unwrap_or(0);
    for rect in rects.iter_mut() {
        rect.x -= min_x;
        rect.y -= min_y;
    }
}

/// The bounding box of an arrangement.
pub fn bounds(rects: &[Rect]) -> Option<Rect> {
    let min_x = rects.iter().map(|r| r.x).min()?;
    let min_y = rects.iter().map(|r| r.y).min()?;
    let max_x = rects.iter().map(Rect::right).max()?;
    let max_y = rects.iter().map(Rect::bottom).max()?;
    Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
}

/// How to draw an arrangement inside a canvas of `width` by `height` pixels
/// with `padding` on every side: the factor from logical pixels to canvas
/// pixels and the canvas offset of the bounding box's corner, centred.
pub fn fit(bounds: &Rect, width: f64, height: f64, padding: f64) -> (f64, f64, f64) {
    let room_w = (width - 2.0 * padding).max(1.0);
    let room_h = (height - 2.0 * padding).max(1.0);
    let factor = (room_w / f64::from(bounds.width)).min(room_h / f64::from(bounds.height));
    let offset_x = (width - f64::from(bounds.width) * factor) / 2.0 - f64::from(bounds.x) * factor;
    let offset_y = (height - f64::from(bounds.height) * factor) / 2.0 - f64::from(bounds.y) * factor;
    (factor, offset_x, offset_y)
}
