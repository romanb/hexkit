//! A cube coordinate system for hexagonal grids.
pub mod dir;

use nalgebra::core::Vector3;
use nalgebra::geometry::Point3;

use std::collections::HashSet;
use std::ops::{ Add, Sub, RangeInclusive };
use std::cmp::{ min, max };

use super::*;
use self::dir::*;

/// Cube coordinates, i.e. points in 3d space, satisfying `x + y + z = 0`.
///
/// Cube coordinates are points on a diagonal plane that "cuts through"
/// a cube grid (a cube made of many smaller cubes). The cubes intersecting
/// the plane project regular hexagons onto the plane, allowing to see the
/// plane as a hexagonal grid whereby the coordinates of each hexagon can be
/// identified with the coordinates of the cube it is projected from.
/// This yields a coordinate system that simplifies many algorithms and
/// thus serves as the canonical coordinate system for any grid
/// (see [`Coords`]).
///
/// Guide: [Cube coordinates]
///
/// [Cube coordinates]: https://www.redblobgames.com/grids/hexagons/#coordinates-cube
/// [`Coords`]: trait.Coords.html
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Cube {
    p: Point3<i32>,
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

    pub fn x(&self) -> i32 { self.p.coords.x }
    pub fn y(&self) -> i32 { self.p.coords.y }
    pub fn z(&self) -> i32 { self.p.coords.z }

    /// Iterate over the neighbouring (adjacent) cube coordinates.
    pub fn neighbours(&self) -> impl Iterator<Item=Cube> + '_ {
        CubeVec::directions().map(move |v| *self + v)
    }

    /// Iterate over the neighbouring cube coordinates along the
    /// diagonal axes.
    pub fn diagonal_neighbours(&self) -> impl Iterator<Item=Cube> + '_ {
        CubeVec::diagonals().map(move |v| *self + v)
    }

    /// The distance to another cube coordinate.
    pub fn distance(&self, other: Cube) -> u32 { // TODO: usize
        ( (self.x() - other.x()).abs() as u32 +
          (self.y() - other.y()).abs() as u32 +
          (self.z() - other.z()).abs() as u32 ) / 2
    }

    /// The shortest path to another cube coordinate, i.e. along
    /// a straight line, including the start coordinate.
    pub fn beeline(&self, other: Cube) -> impl Iterator<Item=Cube> + '_ {
        LineIterator {
            distance: self.distance(other),
            start: *self,
            end: other,
            current: 0
        }
    }

    /// The cube coordinates that are within the given range.
    pub fn range(&self, r: u16) -> impl Iterator<Item=Cube> + '_ {
        let mut v   = Vec::with_capacity(Self::num_in_range(r));
        let x_end   = r as i32;
        let x_start = -x_end;
        for x in RangeInclusive::new(x_start, x_end) {
            let y_start = max(x_start, -x - x_end);
            let y_end   = min(x_end,   -x + x_end);
            for y in RangeInclusive::new(y_start, y_end) {
                v.push(*self + CubeVec::new_xy(x, y));
            }
        }
        v.into_iter()
    }

    /// The number of cube coordinates that are within the given range.
    pub fn num_in_range(r: u16) -> usize {
        3 * (r as usize) * (r as usize + 1) + 1
    }

    pub fn range_overlapping(&self, other: Cube, r: u16)
            -> impl Iterator<Item=Cube> + '_ {
        let n = r as i32;
        let mut v = Vec::new();
        let x_min = max(self.x() - n, other.x() - n);
        let x_max = min(self.x() + n, other.x() + n);
        let y_min = max(self.y() - n, other.y() - n);
        let y_max = min(self.y() + n, other.y() + n);
        let z_min = max(self.z() - n, other.z() - n);
        let z_max = min(self.z() + n, other.z() + n);
        for x in RangeInclusive::new(x_min, x_max) {
            let y_start = max(y_min, -x - z_max);
            let y_end   = min(y_max, -x - z_min);
            for y in RangeInclusive::new(y_start, y_end) {
                v.push(Cube::new_xy(x, y));
            }
        }
        v.into_iter()
    }

    /// The cube coordinates that are within the given range and reachable.
    /// A cube coordinate is reachable if it is in the range...
    pub fn range_reachable<F>(&self, r: u16, f: F) -> impl Iterator<Item=Cube> + '_
            where F: Fn(Cube) -> bool {
        let mut reachable = HashSet::new();
        reachable.insert(*self);
        let mut fringe = Vec::new();
        fringe.push(*self);
        for _ in 1..(r as usize + 1) {
            let mut fringe_i = Vec::new();
            for c in fringe {
                for cn in c.neighbours() {
                    if !reachable.contains(&cn) && f(cn) {
                        reachable.insert(cn);
                        fringe_i.push(cn);
                    }
                }
            }
            fringe = fringe_i;
        }
        reachable.into_iter()
    }

    fn mk(x: i32, y: i32, z: i32) -> Cube {
        let c = Cube { p: Point3::new(x, y, z) };
        debug_assert!(c.is_valid());
        c
    }

    pub fn lerp(&self, other: Cube, t: Frac1) -> Cube {
        let x = lerp(self.x(), other.x(), t);
        let y = lerp(self.y(), other.y(), t);
        let z = lerp(self.z(), other.z(), t);
        Self::round(x, y, z)
    }

    /// Round to the nearest cube coordinate.
    fn round(x: f32, y: f32, z: f32) -> Cube {
        debug_assert!((x + y + z) as isize == 0);
        let (rx, ry, rz) = (x.round(), y.round(), z.round());
        let (dx, dy, dz) = ((x - rx).abs(), (y - ry).abs(), (z - rz).abs());
        if dx > dy && dx > dz {
            Self::mk(-(ry+rz) as i32, ry as i32, rz as i32)
        }
        else if dy > dz {
            Self::mk(rx as i32, -(rx+rz) as i32, rz as i32)
        }
        else {
            Self::mk(rx as i32, ry as i32, -(rx+ry) as i32)
        }
    }

    /// Validity check for the cube coordinates, i.e. that they
    /// represent a point in the plane defined by `x + y + z = 0`.
    fn is_valid(&self) -> bool {
        self.x() + self.y() + self.z() == 0
    }
}

pub struct LineIterator {
    distance: u32,
    current: u32,
    start: Cube,
    end: Cube,
}

impl Iterator for LineIterator {
    type Item = Cube;

    fn next(&mut self) -> Option<Cube> {
        if self.current <= self.distance {
            let frac = Frac1::new(self.current as f32, self.distance as f32);
            let next = self.start.lerp(self.end, frac);
            self.current += 1;
            Some(next)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.distance + 1 - self.current) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for LineIterator {}

/// Linear interpolation.
fn lerp(ai: i32, bi: i32, fr: Frac1) -> f32 {
    let (a, b, t) = (ai as f32, bi as f32, f32::from(fr));
    a + (b - a) * t
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct CubeVec(Vector3<i32>);

impl CubeVec {
    pub fn new_xz(x: i32, z: i32) -> CubeVec {
        CubeVec(Vector3::new(x, -x - z, z))
    }

    pub fn new_xy(x: i32, y: i32) -> CubeVec {
        CubeVec(Vector3::new(x, y, -x - y))
    }

    pub fn new_yz(y: i32, z: i32) -> CubeVec {
        CubeVec(Vector3::new(-y - z, y, z))
    }

    pub fn directions() -> impl Iterator<Item=CubeVec> {
        CUBE_DIR_VECTORS.iter().map(|v| CubeVec(Vector3::from(*v)))
    }

    pub fn diagonals() -> impl Iterator<Item=CubeVec> {
        CUBE_DIA_VECTORS.iter().map(|v| CubeVec(Vector3::from(*v)))
    }
}

impl Add<CubeVec> for CubeVec {
    type Output = CubeVec;

    fn add(self, other: CubeVec) -> Self::Output {
        CubeVec(self.0 + other.0)
    }
}

impl Sub<CubeVec> for CubeVec {
    type Output = CubeVec;

    fn sub(self, other: CubeVec) -> CubeVec {
        CubeVec(self.0 - other.0)
    }
}

impl Neg for CubeVec {
    type Output = CubeVec;

    fn neg(self) -> CubeVec {
        CubeVec(-self.0)
    }
}

impl Mul<i32> for CubeVec {
    type Output = CubeVec;

    fn mul(self, s: i32) -> CubeVec {
        CubeVec(self.0 * s)
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
    use grid::*;
    use quickcheck::*;
    use rand::{ Rng, thread_rng };
    use std::cmp::max;
    use std::i32;
    use super::dir::*;

    impl Arbitrary for Cube {
        fn arbitrary<G: Gen>(g: &mut G) -> Cube {
            let (x, z) = (g.gen::<i16>(), g.gen::<i16>());
            Cube::new_xz(x as i32, z as i32)
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
        quickcheck(prop as fn(Cube) -> bool);
    }

    #[test]
    fn prop_cube_neighbours() {
        fn prop(c: Cube) -> bool {
            let ns = c.neighbours().collect::<Vec<Cube>>();
            ns.iter().all(|n| n.is_valid() && c.distance(*n) == 1)
                && ns.len() == 6
        }
        quickcheck(prop as fn(Cube) -> bool);
    }

    #[test]
    fn prop_cube_diagonal_neighbours() {
        fn prop(c: Cube) -> bool {
            let ns = c.diagonal_neighbours().collect::<Vec<Cube>>();
            ns.iter().all(|n| n.is_valid() && c.distance(*n) == 2)
                && ns.len() == 6
        }
        quickcheck(prop as fn(Cube) -> bool);
    }

    #[test]
    fn prop_cube_distance() {
        fn prop(c1: Cube, c2: Cube) -> bool {
            let v = c1 - c2;
            let (x,y,z) = (v.0.x.abs() as u32, v.0.y.abs() as u32, v.0.z.abs() as u32);
            c1.distance(c2) == max(x, max(y, z))
        }
        quickcheck(prop as fn(Cube, Cube) -> bool);
    }

    #[test]
    fn prop_beeline_distance() {
        fn prop(c1: Cube, c2: Cube) -> bool {
            c1.beeline(c2).count() == c1.distance(c2) as usize + 1
        }
        quickcheck(prop as fn(Cube, Cube) -> bool);
    }

    #[test]
    fn prop_cube_round_valid() {
        fn prop(xi: i16, dx: Frac1, yi: i16, dy: Frac1) -> bool {
            let  x = (xi as f32) + f32::from(dx);
            let  y = (yi as f32) + f32::from(dy);
            let  z = -x - y;
            Cube::round(x, y, z).is_valid()
        }
        quickcheck(prop as fn(i16, Frac1, i16, Frac1) -> bool);
    }

    #[test]
    fn prop_range() {
        fn prop(c: Cube, r: u16) -> bool {
            let v = c.range(r).collect::<Vec<Cube>>();
            v.iter().all(|n| c.distance(*n) <= r as u32)
                && v.contains(&c)
                && v.len() == Cube::num_in_range(r)
        }
        quickcheck(prop as fn(Cube, u16) -> bool);
    }

    #[test]
    fn prop_range_overlapping() {
        fn prop(c1: Cube) -> bool {
            let mut g = thread_rng();
            let r  = g.gen_range(0, 16);
            let dx = g.gen_range(-32, 32);
            let dy = g.gen_range(-32, 32);
            let c2 = c1 + CubeVec::new_xy(dx, dy);
            let v  = c1.range_overlapping(c2, r).collect::<Vec<_>>();
            v.iter().all(|n| c1.range(r).any(|x| x == *n) &&
                             c2.range(r).any(|x| x == *n))
        }
        quickcheck(prop as fn(Cube) -> bool);
    }
}

