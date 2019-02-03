
use crate::geo::Bounds;

/// A border that is sensitive to points falling within
/// the region determined by its bounds and width.
pub struct Border {
    pub width: f32,
    pub scale: f32,
    pub bounds: Bounds,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Delta {
    pub dx: f32,
    pub dy: f32,
}

impl Border {
    /// Evaluate the given point against this border, yielding a
    /// delta whose amplitude is determined by the proximity of the point
    /// to the bounds of the border.
    pub fn eval(&self, x: f32, y: f32) -> Delta {
        let left_min_x   = self.bounds.position.x;
        let left_max_x   = left_min_x + self.width;
        let right_min_x  = self.bounds.position.x + self.bounds.width - self.width;
        let right_max_x  = right_min_x + self.width;
        let top_min_y    = self.bounds.position.y;
        let top_max_y    = top_min_y + self.width;
        let bottom_min_y = self.bounds.position.y + self.bounds.height - self.width;
        let bottom_max_y = bottom_min_y + self.width;

        let dx = if left_min_x <= x && x <= left_max_x {
            (x - left_max_x - 1.) * self.scale
        }
        else if right_min_x <= x && x <= right_max_x {
            (self.width + 1. - (left_min_x + self.bounds.width) + x) * self.scale
        }
        else {
            0.0
        };

        let dy = if top_min_y <= y && y <= top_max_y {
            (y - top_max_y - 1.) * self.scale
        }
        else if bottom_min_y <= y && y <= bottom_max_y {
            (self.width + 1. - (top_min_y + self.bounds.height) + y) * self.scale
        }
        else {
            0.0
        };

        Delta { dx, dy }
    }
}

