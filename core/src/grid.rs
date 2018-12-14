//! Hexagonal grids with overlaid coordinate systems.
pub mod axial;
pub mod cube;
pub mod offset;
pub mod shape;

pub use self::cube::*;
pub use geo::*;
    use grid::shape::Shape;

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
    width: f32,
    height: f32,
    pixel_offset: Vector2<f32>,
}

impl<C: Coords> Grid<C> {
    pub fn new<I>(schema: Schema, shape: Shape<I>) -> Grid<C>
    where I: IntoIterator<Item=Cube> + Clone {
        let (min, max)   = Self::measure(&schema, shape.clone().into_iter());
        let offset_x     = (min.x - schema.width / 2.).abs();
        let offset_y     = (min.y - schema.height / 2.).abs();
        let width        = max.x - min.x + schema.width;
        let height       = max.y - min.y + schema.height;
        let pixel_offset = Vector2::new(offset_x, offset_y);
        let mut store    = HashMap::with_capacity(shape.total);
        store.extend(shape.into_iter().map(|c| {
            let p = c.to_pixel(&schema) + pixel_offset;
            (C::from(c), schema.hexagon(p))
        }));
        Grid {
            schema,
            store,
            width,
            height,
            pixel_offset,
        }
    }

    fn measure<I>(schema: &Schema, shape: I) -> (Point2<f32>, Point2<f32>)
    where I: Iterator<Item=Cube> {
        let centers = shape.map(|c| c.to_pixel(&schema));
        let min_max = (Point2::origin(), Point2::origin());
        centers.fold(min_max, |(min, max), c| {
             let new_min_x = f32::min(min.x, c.x);
             let new_max_x = f32::max(max.x, c.x);
             let new_min_y = f32::min(min.y, c.y);
             let new_max_y = f32::max(max.y, c.y);
             let new_min = Point2::new(new_min_x, new_min_y);
             let new_max = Point2::new(new_max_x, new_max_y);
             (new_min, new_max)
        })
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn from_pixel(&self, p: Point2<f32>) -> Option<(C, &Hexagon)> {
        let c = C::from(Cube::from_pixel(p - self.pixel_offset, &self.schema));
        self.store.get(&c).map(|h| (c,h))
    }

    pub fn to_pixel(&self, c: C) -> Point2<f32> {
        c.into().to_pixel(&self.schema) + self.pixel_offset
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

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
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
                    width: g.width().ceil(),
                    height: g.height().ceil()
                };
                g.schema().bounds(&h).floor().within(&b)
                    &&
                g.from_pixel(h.center).is_some()
                    &&
                g.from_pixel(g.to_pixel(*c)) == Some((*c,h))
            })
        }
        quickcheck(prop as fn(_) -> _);
    }
}

