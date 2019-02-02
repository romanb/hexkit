//! Hexagonal grids with overlaid coordinate systems.
pub mod shape;
pub mod coords;
pub use coords::*;

use crate::geo::*;
use crate::grid::shape::Shape;

use nalgebra::core::Vector2;
use nalgebra::geometry::Point2;
use std::collections::HashMap;

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
    where I: IntoIterator<Item=Cube> {
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

