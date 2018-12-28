//! Hexagonal grids with overlaid coordinate systems.
pub mod axial;
pub mod cube;
pub mod offset;
pub mod shape;

pub use self::cube::*;
pub use crate::geo::*;
    use crate::grid::shape::Shape;

use nalgebra::core::Vector2;
use nalgebra::geometry::Point2;
use num_traits::bounds::Bounded;
use std::hash::Hash;
use std::fmt::{ Debug, Display };
use std::collections::HashMap;

pub trait Coords:
    From<Cube> + Into<Cube> + Eq + Copy + Debug + Display + Hash {
}

/// A grid is a contiguous arrangement of hexagonal tiles with
/// an overlaid coordinate system.
#[derive(Clone, Debug)]
pub struct Grid<C: Coords> {
    schema: Schema,
    store: HashMap<C, Hexagon>, // TODO: Configurable spatial hashing.
    dimensions: Dimensions,
}

#[derive(Clone, Debug)]
pub struct Dimensions {
    pub width: f32,
    pub height: f32,
    pub pixel_offset: Vector2<f32>
}

impl<C: Coords> Grid<C> {
    pub fn new<I>(schema: Schema, shape: Shape<I>) -> Grid<C>
    where I: IntoIterator<Item=Cube> + Clone {
        let num_hexagons = shape.total;
        let (ps, cs): (Vec<Point2<f32>>, Vec<C>) =
            shape.into_iter().map(|c| (c.to_pixel(&schema), C::from(c))).unzip();
        let dimensions = Self::measure(&schema, &ps);
        let offset = dimensions.pixel_offset;
        let store = {
            let mut store = HashMap::with_capacity(num_hexagons);
            let hexagons = ps.iter().map(|c| schema.hexagon(c + offset));
            store.extend(cs.into_iter().zip(hexagons));
            store
        };
        Grid {
            schema,
            store,
            dimensions,
        }
    }

    fn measure(schema: &Schema, centers: &Vec<Point2<f32>>) -> Dimensions {
        let min_max = (Point2::origin(), Point2::origin());
        let (min, max) = centers.iter().fold(min_max, |(min, max), c| {
             let new_min_x = f32::min(min.x, c.x);
             let new_max_x = f32::max(max.x, c.x);
             let new_min_y = f32::min(min.y, c.y);
             let new_max_y = f32::max(max.y, c.y);
             let new_min   = Point2::new(new_min_x, new_min_y);
             let new_max   = Point2::new(new_max_x, new_max_y);
             (new_min, new_max)
        });
        let offset_x = (min.x - schema.width / 2.).abs();
        let offset_y = (min.y - schema.height / 2.).abs();
        Dimensions {
            width:  max.x - min.x + schema.width,
            height: max.y - min.y + schema.height,
            pixel_offset: Vector2::new(offset_x, offset_y),
        }
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn from_pixel(&self, p: Point2<f32>) -> Option<(C, &Hexagon)> {
        let offset = self.dimensions.pixel_offset;
        let c = C::from(Cube::from_pixel(p - offset, &self.schema));
        self.store.get(&c).map(|h| (c,h))
    }

    pub fn to_pixel(&self, c: C) -> Point2<f32> {
        let offset = self.dimensions.pixel_offset;
        c.into().to_pixel(&self.schema) + offset
    }

    pub fn get(&self, c: C) -> Option<&Hexagon> {
        self.store.get(&c)
    }

    pub fn iter(&self) -> impl Iterator<Item=(&C, &Hexagon)> + '_ {
        self.store.iter()
    }

    pub fn iter_within<'a>(&'a self, b: &'a Bounds)
        -> impl Iterator<Item=(&C, &Hexagon)> + 'a
    {
        self.iter().filter(
            move |(_, hex)|
                b.intersects(&self.schema.bounds(&hex)))
    }

    pub fn dimensions(&self) -> &Dimensions {
        &self.dimensions
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

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::*;

    impl<C: Coords + Send + 'static> Arbitrary for Grid<C> {
        fn arbitrary<G: Gen>(g: &mut G) -> Grid<C> {
            let size = SideLength::arbitrary(g);
            let schema = Schema::new(size, Orientation::arbitrary(g));
            let shape = Shape::<Vec<Cube>>::arbitrary(g);
            Grid::new(schema, shape)
        }
    }

    #[test]
    fn prop_new_grid() {
        fn prop(g: Grid<Cube>) -> bool {
            g.iter().all(|(c,h)| {
                let b = Bounds {
                    position: Point2::origin(),
                    width: g.dimensions.width,
                    height: g.dimensions.height
                };
                g.schema().bounds(&h).inner().within(&b.outer())
                    &&
                g.from_pixel(h.center).is_some()
                    &&
                g.from_pixel(g.to_pixel(*c)) == Some((*c,h))
            })
        }
        quickcheck(prop as fn(_) -> _);
    }
}

