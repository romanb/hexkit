//! Hexagonal grids with overlaid coordinate systems.
pub mod axial;
pub mod cube;
pub mod offset;

pub use self::cube::*;
pub use crate::geo::*;

use num_traits::bounds::Bounded;
use std::collections::HashMap;
use std::hash::Hash;

/// A grid is a contiguous arrangement of hexagonal tiles with
/// an overlaid coordinate system.
pub trait Grid<C> where C: Coords + Hash {
    fn schema(&self) -> &Schema;
    fn tiles(&self) -> &HashMap<C, Hexagon>; // HashMap<C, Tile<S>>

    fn visible_tiles<'a>(&'a self, vp: &'a Viewport)
            -> Box<Iterator<Item=(&C,&Hexagon)> + 'a> {
        Box::new(self.tiles().iter().filter(move |(_, hex)|
            vp.visible(&self.schema().bounds(&hex))))
    }
}

pub struct Tile<S> {
    pub hex: Hexagon,
    pub state: S,
}

/// Coordinates on a grid. A grid coordinate system must support
/// conversion to and from cube coordinates.
pub trait Coords: Eq + Copy {
    /// The type of the underlying grid.
    type Grid;

    /// Convert the coordinates to cube coordinates. This conversion
    /// must not fail, i.e. every coordinate system must be fully
    /// "embedded" in the cube coordinate system.
    fn to_cube(self, grid: &Self::Grid) -> Cube;

    /// Convert from cube coordinates. If the cube coordinates do not
    /// represent valid coordinates for this coordinate system and
    /// the given grid, `None` should be returned.
    fn from_cube(cube: Cube, grid: &Self::Grid) -> Option<Self>;
}

/// A viewport defines the visible region of a grid.
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32
}

impl Viewport {
    pub fn visible(&self, b: &HexBounds) -> bool {
        self.x               < b.x + b.width  &&
        self.x + self.width  > b.x            &&
        self.y               < b.y + b.height &&
        self.y + self.height > b.y
    }
}

/// A fraction in the unit interval `[0,1]`.
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Frac1(f32);

impl Frac1 {
    /// Create a new fraction in the unit interval [0,1].
    /// If the numerator is greater than the denominator or if
    /// the denominator is zero, a panic is triggered.
    pub fn new(numer: f32, denom: f32) -> Frac1 {
        if numer > denom {
            panic!("numer > denom");
        }
        if denom == 0. {
            panic!("denom == 0");
        }
        Frac1(numer / denom)
    }
}

impl Bounded for Frac1 {
    fn min_value() -> Frac1 {
        Frac1(0.)
    }
    fn max_value() -> Frac1 {
        Frac1(1.)
    }
}

impl From<Frac1> for f32 {
    fn from(Frac1(f): Frac1) -> f32 { f }
}


