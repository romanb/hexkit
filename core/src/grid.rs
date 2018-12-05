//! Hexagonal grids with overlaid coordinate systems.
pub mod axial;
pub mod cube;
pub mod offset;
pub mod shape;

pub use self::cube::*;
pub use geo::*;

use nalgebra::geometry::Point2;
use num_traits::bounds::Bounded;
use std::fmt::Display;

use std::hash::Hash;
use std::fmt::Debug;
use std::collections::HashMap;
use nalgebra::core::Vector2;
use grid::shape::*;

pub trait Coords: From<Cube> + Into<Cube>
    + Eq + Copy + Debug + Display + Hash {}

pub trait Store<C: Coords> {
    fn schema(&self) -> &Schema;

    fn get(&self, o: C) -> Option<&Hexagon>;

    fn iter(&self) -> Box<dyn Iterator<Item=(&C, &Hexagon)> + '_>;

    fn iter_visible<'a>(&'a self, vp: &'a Viewport)
        -> Box<dyn Iterator<Item=(&C, &Hexagon)> + 'a>
    {
        Box::new(self.iter().filter(
            move |(_, hex)|
                vp.visible(&self.schema().bounds(&hex))))
    }
}

pub struct HashMapStore<C: Coords + Hash> {
    schema: Schema,
    hexagons: HashMap<C, Hexagon>,
}

impl<C: Coords + Hash> Store<C> for HashMapStore<C> {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn get(&self, c: C) -> Option<&Hexagon> {
        self.hexagons.get(&c)
    }

    fn iter(&self) -> Box<dyn Iterator<Item=(&C, &Hexagon)> + '_> {
        Box::new(self.hexagons.iter())
    }
}

/// A grid is a contiguous arrangement of hexagonal tiles with
/// an overlaid coordinate system.
#[derive(Clone, Debug)]
pub struct Grid<C: Coords> {
    schema: Schema,
    hexagons: HashMap<C, Hexagon>, // impl Store<C>
    width: f32,
    height: f32,
    pixel_offset: Vector2<f32>,
}

impl<C: Coords> Grid<C> {
    pub fn new(schema: Schema, shape: ShapeIter<impl Iterator<Item=Cube> + Clone>) -> Grid<C> {
        let mut hexagons = HashMap::with_capacity(shape.total);
        let bounds = {
            let centers = shape.clone().map(|c| c.to_pixel(&schema));
            let min_max = ((0.,0.), (0.,0.));
            centers.fold(min_max, |((min_x, max_x),(min_y, max_y)), c| {
                 let new_min_x = f32::min(min_x, c.x);
                 let new_max_x = f32::max(max_x, c.x);
                 let new_min_y = f32::min(min_y, c.y);
                 let new_max_y = f32::max(max_y, c.y);
                 ((new_min_x, new_max_x), (new_min_y, new_max_y))
            })
        };
        let offset_x = ((bounds.0).0 - schema.width / 2.).abs();
        let offset_y = ((bounds.1).0 - schema.height / 2.).abs();
        let width = (bounds.0).1 - (bounds.0).0 + schema.width;
        let height = (bounds.1).1 - (bounds.1).0 + schema.height;
        let pixel_offset = Vector2::new(offset_x, offset_y);
        hexagons.extend(shape.map(|c| {
            let p = c.to_pixel(&schema) + pixel_offset;
            (C::from(c), schema.hexagon(p))
        }));
        Grid {
            schema,
            hexagons,
            width,
            height,
            pixel_offset,
        }
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn from_pixel(&self, p: Point2<f32>) -> Option<C> {
        // TODO: lookup coords
        Some(C::from(Cube::from_pixel(p - self.pixel_offset, self.schema())))
    }

    pub fn to_pixel(&self, c: C) -> Point2<f32> {
        c.into().to_pixel(self.schema())
    }

    pub fn tile(&self, o: C) -> Option<&Hexagon> {
        self.hexagons.get(&o)
    }

    pub fn tiles(&self) -> Box<dyn Iterator<Item=(&C, &Hexagon)> + '_> {
        Box::new(self.hexagons.iter())
    }

    pub fn visible_tiles<'a>(&'a self, vp: &'a Viewport)
        -> Box<dyn Iterator<Item=(&C, &Hexagon)> + 'a>
    {
        Box::new(self.tiles().filter(
            move |(_, hex)| vp.visible(&self.schema().bounds(&hex))
        ))
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }
}

/// A viewport defines a visible, rectangular region of a grid.
#[derive(Copy, Clone, Debug)]
pub struct Viewport { // TODO: newtype on Bounds?
    pub position: Point2<f32>,
    pub width: f32,
    pub height: f32
}

impl Viewport {
    /// Check whether the given bounds intersect with the viewport,
    /// i.e. if any points within the bounds are visible.
    pub fn visible(&self, b: &Bounds) -> bool {
        self.position.x               < b.position.x + b.width  &&
        self.position.x + self.width  > b.position.x            &&
        self.position.y               < b.position.y + b.height &&
        self.position.y + self.height > b.position.y
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


