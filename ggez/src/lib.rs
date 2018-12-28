
extern crate ggez;
extern crate hexworld;

use hexworld::geo::{ Bounds, Hexagon };
use hexworld::grid::{ Grid, Cube, Coords };

use ggez::*;
use ggez::graphics::*;

pub struct GridView<C: Coords> {
    grid: Grid<C>,
    viewport: Bounds,
    position: Point2,
    update: Update,
    updated: bool,
}

pub trait TileDrawer<C> {
    fn draw_tile(&mut self,
                 ctx: &mut Context,
                 coords: C,
                 hex: &Hexagon,
                 mb: &mut MeshBuilder) -> GameResult<()>;
}

/// Scheduled changes for the next update.
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

impl<C: Coords> GridView<C> {

    pub fn new(grid: Grid<C>, bounds: Bounds) -> GridView<C> {
        GridView {
            grid,
            position: bounds.position,
            viewport: Bounds {
                position: Point2::origin(),
                width: bounds.width,
                height: bounds.height
            },
            update: Update::empty(),
            updated: false,
        }
    }

    pub fn position(&self) -> Point2 {
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

    /// Whether the view has been updated and needs redrawing.
    pub fn updated(&self) -> bool {
        self.updated
    }

    /// Get a reference to the bounds of the viewport. The position
    /// is relative to the position of the grid, i.e. scrolling moves
    /// the viewport over the grid. The width and height of the viewport
    /// correspond to the width and height of the grid view.
    pub fn viewport(&self) -> &Bounds {
        &self.viewport
    }

    pub fn from_pixel(&self, x: i32, y: i32) -> Option<(C, &Hexagon)> {
        self.grid.from_pixel(Point2::new(
            x as f32 - self.position.x + self.viewport.position.x,
            y as f32 - self.position.y + self.viewport.position.y
        ))
    }

    /// Apply all pending updates to the view. Returns `Ok(true)` if
    /// the view has been updated and needs redrawing.
    pub fn update(&mut self, _ctx: &mut Context) -> GameResult<bool> {
        if !self.update.is_pending() {
            return Ok(false)
        }
        if let Some((w,h)) = self.update.resize {
            self.viewport.width  = w as f32;
            self.viewport.height = h as f32;
            self.update.resize = None;
            self.updated = true;
        }
        let grid  = self.grid.dimensions();
        let old_p = self.viewport.position;
        let new_x = old_p.x + self.update.scroll.dx;
        let new_y = old_p.y + self.update.scroll.dy;
        let max_x = grid.width  - self.viewport.width;
        let max_y = grid.height - self.viewport.height;
        self.viewport.position.x = f32::min(max_x, f32::max(0., new_x));
        self.viewport.position.y = f32::min(max_y, f32::max(0., new_y));
        if !self.updated && old_p == self.viewport.position {
            self.update = Update::empty();
            return Ok(false)
        }
        if !self.update.scroll.repeat {
            self.update = Update::empty();
        }
        self.updated = true;
        Ok(true)
    }

    /// Draw the view in the given context.
    pub fn draw(&mut self, ctx: &mut Context, tdr: &mut impl TileDrawer<C>) -> GameResult<()> {
        let grid_pos = self.grid_position();
        let mut mesh = MeshBuilder::new();
        for (coords, hex) in self.grid.iter_within(&self.viewport) {
            tdr.draw_tile(ctx, *coords, hex, &mut mesh)?;
        }
        let grid = mesh.build(ctx)?;
        graphics::draw(ctx, &grid, grid_pos, 0.0)?;
        self.updated = false;
        Ok(())
    }

    pub fn scroll_x(&mut self, dx: f32) {
        self.update.scroll.dx += dx;
        self.update.scroll.repeat = false;
    }

    pub fn scroll_y(&mut self, dy: f32) {
        self.update.scroll.dy += dy;
        self.update.scroll.repeat = false;
    }

    /// Border scroll updates are sticky, i.e. once initiated,
    /// every call to `update` will result in an updated (scrolled)
    /// view. Scrolling stops only once `scroll_border` is called
    /// with a position that lies either outside of the given bounds or
    /// within the given bounds but outside of the border region.
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

    pub fn draw_hexagons(
        &self,
        ctx: &mut Context,
        mesh: &mut MeshBuilder,
        it: impl Iterator<Item=Cube>,
        mode: DrawMode
    ) -> GameResult<()> {
        for cc in it {
            if let Some(c) = Some(C::from(cc)) {
                if let Some(hex) = self.grid.get(c) {
                    let hex_bounds = self.grid.schema().bounds(hex);
                    if self.viewport.intersects(&hex_bounds) {
                        mesh.polygon(mode, hex.corners());
                    }
                }
            }
        }
        let m = mesh.build(ctx)?;
        graphics::draw(ctx, &m, self.grid_position(), 0.0)
    }

    /// The current position of the grid (i.e. the top-left corner of the
    /// grid's bounding box) on the screen coordinate system.
    /// Rendering the grid at this position "pulls" the viewport, which
    /// moves across the grid, into the grid view.
    pub fn grid_position(&self) -> Point2 {
        -self.viewport.position + self.position.coords
    }
}
