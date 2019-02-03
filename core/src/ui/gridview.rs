
use crate::geo::{ Bounds, Hexagon };
use crate::grid::{ Grid, Coords };
use crate::ui::scroll;

use nalgebra::Point2;

/// The state of a grid view.
pub struct State<C: Coords> {
    grid: Grid<C>,
    viewport: Bounds,
    position: Point2<f32>,
}

impl<C: Coords> State<C> {

    pub fn new(grid: Grid<C>, bounds: Bounds) -> State<C> {
        State {
            grid,
            position: bounds.position,
            viewport: Bounds {
                position: Point2::origin(),
                width: bounds.width,
                height: bounds.height
            },
        }
    }

    pub fn position(&self) -> Point2<f32> {
        self.position
    }

    pub fn grid(&self) -> &Grid<C> {
        &self.grid
    }

    pub fn width(&self) -> f32 {
        self.viewport.width
    }

    pub fn height(&self) -> f32 {
        self.viewport.height
    }

    /// Get a reference to the bounds of the viewport. The position
    /// is relative to the position of the grid, i.e. scrolling moves
    /// the viewport over the grid. The width and height of the viewport
    /// correspond to the width and height of the grid view.
    pub fn viewport(&self) -> &Bounds {
        &self.viewport
    }

    pub fn from_pixel(&self, p: Point2<f32>) -> Option<(C, &Hexagon)> {
        // FIXME: Turn self.position into bounds. self.viewport and
        // self.bounds differ only in position - width and height must
        // be kept in-sync.
        let bounds = Bounds {
            position: self.position,
            width: self.width(),
            height: self.height()
        };
        if !bounds.contains(p) {
            return None
        }
        self.grid.from_pixel(p - self.position.coords + self.viewport.position.coords)
    }

    /// Get an iterator over the hexagons currently in the viewport.
    pub fn iter_viewport(&self) -> impl Iterator<Item=(&C, &Hexagon)> + '_ {
        self.grid.iter_within(&self.viewport)
    }

    /// Scroll the viewport over the grid.
    pub fn scroll(&mut self, scroll: scroll::Delta) {
        let grid  = self.grid.dimensions();
        let old_p = self.viewport.position;
        let new_x = old_p.x + scroll.dx;
        let new_y = old_p.y + scroll.dy;
        let max_x = grid.width  - self.viewport.width;
        let max_y = grid.height - self.viewport.height;
        self.viewport.position.x = f32::min(max_x, f32::max(0., new_x));
        self.viewport.position.y = f32::min(max_y, f32::max(0., new_y));
    }

    /// Schedule a resize of the view for the next update.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.viewport.width  = width as f32;
        self.viewport.height = height as f32;
        // Adjust the viewport position according to the new size,
        // so it doesn't "jump" on the next scroll.
        self.scroll(scroll::Delta { dx: 0.0, dy: 0.0 });
    }

    /// The current position of the grid (i.e. the top-left corner of the
    /// grid's bounding box) on the screen coordinate system.
    /// Rendering the grid at this position "pulls" the viewport, which
    /// moves across the grid, into the grid view.
    pub fn grid_position(&self) -> Point2<f32> {
        -self.viewport.position + self.position.coords
    }
}
