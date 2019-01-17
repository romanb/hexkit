
use crate::geo::{ Bounds, Hexagon };
use crate::grid::{ Grid, Coords };

use nalgebra::Point2;

/// The state of a grid view.
pub struct State<C: Coords> {
    grid: Grid<C>,
    viewport: Bounds,
    position: Point2<f32>,
    update: Update,
}

/// Scheduled changes for the next update of a grid view.
struct Update {
    scroll: Scroll, // Option<Scroll>,
    resize: Option<(u32,u32)>,
}

impl Update {
    fn empty() -> Update {
        Update {
            scroll: Scroll::empty(),
            resize: None,
        }
    }

    fn is_pending(&self) -> bool {
        self.scroll.is_pending() || self.resize.is_some()
    }
}

struct Scroll {
    dx: f32,
    dy: f32,
    repeat: bool,
}

impl Scroll {
    fn empty() -> Scroll {
        Scroll { dx: 0., dy: 0., repeat: false }
    }

    fn is_pending(&self) -> bool {
        self.dx != 0. || self.dy != 0.
    }
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
            update: Update::empty(),
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
        self.grid.from_pixel(p - self.position.coords + self.viewport.position.coords)
    }

    /// Apply all pending updates to the view. Returns `Ok(true)` if
    /// the view has been updated and needs redrawing.
    pub fn update(&mut self) -> bool {
        if !self.update.is_pending() {
            return false
        }
        let mut resized = false;
        if let Some((w,h)) = self.update.resize {
            self.viewport.width  = w as f32;
            self.viewport.height = h as f32;
            self.update.resize = None;
            resized = true;
        }
        let grid  = self.grid.dimensions();
        let old_p = self.viewport.position;
        let new_x = old_p.x + self.update.scroll.dx;
        let new_y = old_p.y + self.update.scroll.dy;
        let max_x = grid.width  - self.viewport.width;
        let max_y = grid.height - self.viewport.height;
        self.viewport.position.x = f32::min(max_x, f32::max(0., new_x));
        self.viewport.position.y = f32::min(max_y, f32::max(0., new_y));
        if !resized && old_p == self.viewport.position {
            self.update = Update::empty();
            return false
        }
        if !self.update.scroll.repeat {
            self.update = Update::empty();
        }
        true
    }

    /// Get an iterator over the hexagons currently in the viewport.
    pub fn iter_viewport(&self) -> impl Iterator<Item=(&C, &Hexagon)> + '_ {
        self.grid.iter_within(&self.viewport)
    }

    /// Schedule a one-time scroll / shift of the viewport's x-coordinate
    /// for the next update.
    pub fn scroll_x(&mut self, dx: f32) {
        self.update.scroll.dx = dx;
        self.update.scroll.repeat = false;
    }

    /// Schedule a one-time scroll / shift of the viewport's y-coordinate
    /// for the next update.
    pub fn scroll_y(&mut self, dy: f32) {
        self.update.scroll.dy = dy;
        self.update.scroll.repeat = false;
    }

    /// Schedule repeated scrolling of the viewport's x/y coordinates
    /// based on the proximity of the given point to the border of the given
    /// bounds. Typically, the given point represents mouse coordinates and the
    /// bounds are those of the drawable region of the window, thus realising
    /// "border scrolling" based on mouse movements near a border of the screen.
    ///
    /// Border scroll updates are sticky, i.e. once initiated, every call to
    /// `update` will result in an updated (scrolled) view. Scrolling stops only
    /// once `scroll_border` is called with a position that lies either outside
    /// of the given bounds or within the given bounds but outside of the border
    /// region.
    pub fn scroll_border(&mut self, x: f32, y: f32, bounds: &Bounds, border: f32, scale: f32) {
        let left_min_x   = bounds.position.x;
        let left_max_x   = left_min_x + border;
        let right_min_x  = bounds.position.x + bounds.width - border;
        let right_max_x  = right_min_x + border;
        let top_min_y    = bounds.position.y;
        let top_max_y    = top_min_y + border;
        let bottom_min_y = bounds.position.y + bounds.height - border;
        let bottom_max_y = bottom_min_y + border;

        self.update.scroll.repeat = true;

        if left_min_x <= x && x <= left_max_x {
            self.update.scroll.dx = (x - left_max_x - 1.) * scale;
        }
        else if right_min_x <= x && x <= right_max_x {
            self.update.scroll.dx = (border + 1. - (left_min_x + bounds.width) + x) * scale;
        }
        else {
            self.update.scroll.dx = 0.;
        }

        if top_min_y <= y && y <= top_max_y {
            self.update.scroll.dy = (y - top_max_y - 1.) * scale;
        }
        else if bottom_min_y <= y && y <= bottom_max_y {
            self.update.scroll.dy = (border + 1. - (top_min_y + bounds.height) + y) * scale;
        }
        else {
            self.update.scroll.dy = 0.;
        }
    }

    /// Schedule a resize of the view for the next update.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.update.resize = Some((width, height));
    }

    /// The current position of the grid (i.e. the top-left corner of the
    /// grid's bounding box) on the screen coordinate system.
    /// Rendering the grid at this position "pulls" the viewport, which
    /// moves across the grid, into the grid view.
    pub fn grid_position(&self) -> Point2<f32> {
        -self.viewport.position + self.position.coords
    }
}
