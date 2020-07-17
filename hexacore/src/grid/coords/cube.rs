//! Cube coordinates.

pub mod vec;
pub use vec::*;

use super::*;

use nalgebra::{ Point2, Point3 };

use std::ops::{ Add, Sub };
use std::cmp::{ Ordering };
use std::fmt;

/// Cube coordinates, i.e. points in 3d space, satisfying `x + y + z = 0`.
///
/// Cube coordinates are points on a diagonal plane that "cuts through"
/// a cube grid (a cube made of many smaller cubes). The cubes intersecting
/// the plane project regular hexagons onto the plane, allowing to see the
/// plane as a hexagonal grid whereby the coordinates of each hexagon can be
/// identified with the coordinates of the cube it is projected from.
/// This yields a coordinate system that simplifies many algorithms and
/// thus serves as the canonical coordinate system for any grid (see [`Coords`]).
///
/// The following illustrates the coordinate system with a flat-top orientation.
/// For a pointy-top orientation, it is to be rotated 30 degrees counterclockwise,
/// i.e. such that `+x/-y` and `+y/-x` are horizontally aligned.
///
/// ```raw
///         +y/-z
///  +y/-x   ___   +x/-z
///         /   \
///         \___/
///  +z/-x         +x/-y
///         +z/-y
/// ```
///
/// Guide: [Cube coordinates]
///
/// [Cube coordinates]: https://www.redblobgames.com/grids/hexagons/#coordinates-cube
/// [`Coords`]: ../trait.Coords.html
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, PartialOrd)]
pub struct Cube {
    pub(super) p: Point3<i32>,
}

impl Cube {
    pub fn origin() -> Cube {
        Self::mk(0, 0, 0)
    }

    pub fn new_xz(x: i32, z: i32) -> Cube {
        Self::mk(x, -x - z, z)
    }

    pub fn new_xy(x: i32, y: i32) -> Cube {
        Self::mk(x, y, -x - y)
    }

    pub fn new_yz(y: i32, z: i32) -> Cube {
        Self::mk(-y - z, y, z)
    }

    pub fn x(&self) -> i32 { self.p.x }
    pub fn y(&self) -> i32 { self.p.y }
    pub fn z(&self) -> i32 { self.p.z }

    /// Round to the nearest cube coordinate.
    pub(crate) fn round(x: f32, y: f32, z: f32) -> Cube {
        debug_assert!((x + y + z) as isize == 0);
        let (rx, ry, rz) = (x.round(), y.round(), z.round());
        let (dx, dy, dz) = ((x - rx).abs(), (y - ry).abs(), (z - rz).abs());
        if dx > dy && dx > dz {
            Cube::mk(-(ry+rz) as i32, ry as i32, rz as i32)
        }
        else if dy > dz {
            Cube::mk(rx as i32, -(rx+rz) as i32, rz as i32)
        }
        else {
            Cube::mk(rx as i32, ry as i32, -(rx+ry) as i32)
        }
    }

    pub(crate) fn mk(x: i32, y: i32, z: i32) -> Cube {
        debug_assert!(x + y + z == 0);
        Cube { p: Point3::new(x, y, z) }
    }

    /// Validity check for the cube coordinates, i.e. that they
    /// represent a point in the plane defined by `x + y + z = 0`.
    #[cfg(test)]
    fn is_valid(&self) -> bool {
        self.x() + self.y() + self.z() == 0
    }

    /// Compute the center of the hexagon with these cube coordinates
    /// in the context of the given geometric schema and satisfying
    /// ```ignore
    /// Cube::origin().to_pixel(&s) == Point2::origin()
    /// ```
    /// for every schema `s`.
    pub fn to_pixel(self, schema: &geo::Schema) -> Point2<f32> {
        schema.to_pixel(self)
    }

    /// Compute the (nearest) cube coordinates for a point in the
    /// context of the given geometric schema, satisfying
    /// ```ignore
    /// Cube::from_pixel(c.to_point(&s), &s) == c
    /// ```
    /// for any cube coordinates `c` and schema `s`.
    pub fn from_pixel(p: Point2<f32>, schema: &geo::Schema) -> Cube {
        schema.from_pixel(p)
    }

}

impl Coords for Cube {}

impl fmt::Display for Cube {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{},{})", self.p.x, self.p.y, self.p.z)
    }
}

impl From<Point2<f32>> for Cube {
    fn from(p: Point2<f32>) -> Cube {
        Cube::round(p.x, -p.x - p.y, p.y)
    }
}

impl From<Cube> for Point2<f32> {
    fn from(c: Cube) -> Point2<f32> {
        Point2::new(c.p.x as f32, c.p.z as f32)
    }
}

impl Ord for Cube {
    fn cmp(&self, other: &Cube) -> Ordering {
        self.partial_cmp(other).unwrap_or_else(||
            match self.p.x.cmp(&other.p.x) {
                Ordering::Equal => match self.p.y.cmp(&other.p.y) {
                    Ordering::Equal => self.p.z.cmp(&other.p.z),
                    ord => ord
                }
                ord => ord
            })
    }
}

impl Add<CubeVec> for Cube {
    type Output = Cube;

    fn add(self, v: CubeVec) -> Cube {
        Cube { p: self.p + v.0 }
    }
}

impl Sub<Cube> for Cube {
    type Output = CubeVec;

    fn sub(self, other: Cube) -> CubeVec {
        CubeVec(self.p - other.p)
    }
}

impl Sub<CubeVec> for Cube {
    type Output = Cube;

    fn sub(self, v: CubeVec) -> Cube {
        self + (-v)
    }
}

#[cfg(test)]
mod tests {
    use geo::*;
    use super::*;
    use quickcheck::*;
    use rand::{ Rng, thread_rng };
    use std::cmp::max;
    use std::collections::HashSet;
    use std::i32;

    impl Arbitrary for Cube {
        fn arbitrary<G: Gen>(g: &mut G) -> Cube {
            let (x, z) = (g.gen::<i16>(), g.gen::<i16>());
            Cube::new_xz(x as i32, z as i32)
        }
    }

    impl Arbitrary for CubeVec {
        fn arbitrary<G: Gen>(g: &mut G) -> CubeVec {
            let (x, z) = (g.gen::<i16>(), g.gen::<i16>());
            CubeVec::new_xz(x as i32, z as i32)
        }
    }

    impl Arbitrary for Frac1 {
        fn arbitrary<G: Gen>(g: &mut G) -> Frac1 {
            let (a, b) = (g.gen(), g.gen());
            if a == 0. {
                Frac1::new(a,1.)
            }
            else if a > b {
                Frac1::new(b, a)
            } else {
                Frac1::new(a, b)
            }
        }
    }

    #[test]
    fn prop_new_cube() {
        fn prop(c: Cube) -> bool {
            c.is_valid()
        }
        quickcheck(prop as fn(_) -> _);
    }

    #[test]
    fn prop_cube_neighbours() {
        fn prop(c: Cube) -> bool {
            let ns = neighbours(c).collect::<Vec<Cube>>();
            ns.iter().all(|n| n.is_valid() && distance(c, *n) == 1)
                && ns.len() == 6
        }
        quickcheck(prop as fn(_) -> _);
    }

    #[test]
    fn prop_cube_diagonal_neighbours() {
        fn prop(c: Cube) -> bool {
            let ns = diagonal_neighbours(c).collect::<Vec<Cube>>();
            ns.iter().all(|n| n.is_valid() && distance(c, *n) == 2)
                && ns.len() == 6
        }
        quickcheck(prop as fn(_) -> _);
    }

    #[test]
    fn prop_cube_distance() {
        fn prop(c1: Cube, c2: Cube) -> bool {
            let v = c1 - c2;
            let (x,y,z) = (
                v.0.x.abs() as usize,
                v.0.y.abs() as usize,
                v.0.z.abs() as usize
            );
            distance(c1, c2) == max(x, max(y, z))
        }
        quickcheck(prop as fn(_,_) -> _);
    }

    #[test]
    fn prop_beeline_distance() {
        fn prop(c1: Cube, c2: Cube) -> bool {
            beeline(c1, c2).count() == distance(c1, c2) as usize + 1
        }
        quickcheck(prop as fn(_,_) -> _);
    }

    #[test]
    fn prop_cube_round_valid() {
        fn prop(xi: i16, dx: Frac1, yi: i16, dy: Frac1) -> bool {
            let  x = (xi as f32) + f32::from(dx);
            let  y = (yi as f32) + f32::from(dy);
            let  z = -x - y;
            Cube::round(x, y, z).is_valid()
        }
        quickcheck(prop as fn(_,_,_,_) -> bool);
    }

    #[test]
    fn prop_range() {
        fn prop(c: Cube, r: u16) -> bool {
            let v = range(c, r).collect::<Vec<Cube>>();
            v.iter().all(|n| distance(c, *n) <= r as usize)
                && v.contains(&c)
                && v.len() == num_in_range(r)
        }
        quickcheck(prop as fn(_,_) -> _);
    }

    #[test]
    fn prop_range_overlapping() {
        fn prop(c1: Cube) -> bool {
            let mut g = thread_rng();
            let r  = g.gen_range(0, 16);
            let dx = g.gen_range(-32, 32);
            let dy = g.gen_range(-32, 32);
            let c2 = c1 + CubeVec::new_xy(dx, dy);
            let v  = range_overlapping(c1, c2, r).collect::<Vec<_>>();
            v.iter().all(|n| range(c1, r).any(|x| x == *n) &&
                             range(c2, r).any(|x| x == *n))
        }
        quickcheck(prop as fn(_) -> _);
    }

    #[test]
    fn prop_walk_ring() {
        fn prop(c: Cube, r: u16, d: FlatTopDirection) -> bool {
            let cw = walk_ring(c, d, r, Rotation::CW).collect::<Vec<_>>();
            let (cw_head, cw_tail) = (cw.first(), cw.iter().skip(1));

            let ccw = walk_ring(c, d, r, Rotation::CCW).collect::<Vec<_>>();
            let (ccw_head, ccw_tail) = (ccw.first(), ccw.iter().skip(1));

            cw_head == ccw_head
                &&
                cw_tail.collect::<Vec<_>>()
                ==
                ccw_tail.rev().collect::<Vec<_>>()
        }
        quickcheck(prop as fn(_,_,_) -> _);
    }

    #[test]
    fn prop_walk_range() {
        fn prop(c: Cube, r: u16, d: FlatTopDirection, rot: Rotation) -> bool {
            walk_range(c, d, r, rot).collect::<HashSet<_>>()
                ==
            range(c, r).collect()
        }
        quickcheck(prop as fn(_,_,_,_) -> _);
    }

    #[test]
    fn prop_cube_to_pixel_origin() {
        fn prop(o: Orientation, l: SideLength) -> bool {
            let s = Schema::new(l, o);
            Cube::origin().to_pixel(&s) == Point2::origin()
        }
        quickcheck(prop as fn(_,_) -> _);
    }

    #[test]
    fn prop_cube_from_to_pixel_identity() {
        fn prop(c: Cube, s: SideLength, o: Orientation) -> bool {
            let s = Schema::new(s, o);
            Cube::from_pixel(c.to_pixel(&s), &s) == c
        }
        quickcheck(prop as fn(_,_,_) -> _);
    }

    #[test]
    fn prop_range_visible_all() {
        fn prop(c: Cube, r: u16) -> bool {
            range_visible(c, r % 32, |_| true).collect::<HashSet<_>>()
                ==
            range(c, r % 32).collect()
        }
        quickcheck(prop as fn(_,_) -> _);
    }

    #[test]
    fn prop_range_visible_blocked_dir() {
        fn prop(c: Cube, r: u16, d: FlatTopDirection) -> bool {
            let range = (r % 32) + 1;
            let blocked = c + d.vector();
            let visible = range_visible(c, range, |x| x != blocked).collect::<HashSet<_>>();
            // All coordinates in the direction of (and beyond)
            // the blocked neighbour are expected not to be visible.
            let blocked_end = c + d.vector() * range as i32;
            visible.contains(&blocked)
                &&
            beeline(c, blocked_end)
                .skip(1) // skip the origin
                .all(|x| x == blocked || !visible.contains(&x))
        }
        quickcheck(prop as fn(_,_,_) -> _);
    }
}

