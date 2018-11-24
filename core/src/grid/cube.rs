//! A cube coordinate system for hexagonal grids.

pub mod vec;

use nalgebra::geometry::Point3;

use std::collections::HashSet;
use std::ops::{ Add, Sub, RangeInclusive };
use std::cmp::{ Ordering, min, max };
use std::iter;

use super::*;
use self::vec::*;

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
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, PartialOrd)]
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

    pub fn x(&self) -> i32 { self.p.x }
    pub fn y(&self) -> i32 { self.p.y }
    pub fn z(&self) -> i32 { self.p.z }

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
        ( (self.p.x - other.p.x).abs() as u32 +
          (self.p.y - other.p.y).abs() as u32 +
          (self.p.z - other.p.z).abs() as u32 ) / 2
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
        // TODO: Dedicated RangeIter
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
    #[inline]
    pub fn num_in_range(r: u16) -> usize {
        Self::num_in_ring(r) * (r as usize + 1) / 2 + 1
        // 3 * (r as usize) * (r as usize + 1) + 1
    }

    /// The number of cube coordinates that are in the ring of
    /// a given radius.
    #[inline]
    pub fn num_in_ring(r: u16) -> usize {
        6 * (r as usize)
    }

    pub fn range_overlapping(&self, other: Cube, r: u16)
        -> impl Iterator<Item=Cube> + '_
    {
        // TODO: Use dedicated RangeIter
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
    pub fn range_reachable<F>(&self, r: u16, f: F) -> HashSet<Cube>
        where F: Fn(Cube) -> bool
    {
        let mut reachable = HashSet::new();
        let mut fringe = Vec::new();
        reachable.insert(*self);
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
        reachable
    }

    pub fn walk_ring<D>(&self, dir: D, rad: u16, rot: Rotation)
        -> impl Iterator<Item=Cube> + '_
        where D: DirIndex
    {
        let mut v = Vec::with_capacity(rad as usize * 6);
        let mut c = *self + CubeVec::direction(dir) * rad as i32;
        for d in CubeVec::walk_directions(dir, rot) {
            for _ in 0..rad {
                v.push(c);
                c = c + d;
            }
        }
        v.into_iter()
    }

    pub fn walk_range<'a, D>(&'a self, dir: D, rad: u16, rot: Rotation)
        -> impl Iterator<Item=Cube> + 'a
        where D: DirIndex + 'a
    {
        let rings = (1..rad+1).flat_map(move |i| self.walk_ring(dir, i, rot));
        iter::once(*self).chain(rings)
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
        // Cube::new_xy(self.p.x + v.x(), self.p.y + v.y())
    }
}

impl Sub<Cube> for Cube {
    type Output = CubeVec;

    fn sub(self, other: Cube) -> CubeVec {
        CubeVec(self.p - other.p)
        // CubeVec::new_xy(self.p.x - other.p.x, self.p.y - other.p.y)
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
    use std::collections::HashSet;
    use std::i32;
    use super::vec::*;

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
            let ns = c.neighbours().collect::<Vec<Cube>>();
            ns.iter().all(|n| n.is_valid() && c.distance(*n) == 1)
                && ns.len() == 6
        }
        quickcheck(prop as fn(_) -> _);
    }

    #[test]
    fn prop_cube_diagonal_neighbours() {
        fn prop(c: Cube) -> bool {
            let ns = c.diagonal_neighbours().collect::<Vec<Cube>>();
            ns.iter().all(|n| n.is_valid() && c.distance(*n) == 2)
                && ns.len() == 6
        }
        quickcheck(prop as fn(_) -> _);
    }

    #[test]
    fn prop_cube_distance() {
        fn prop(c1: Cube, c2: Cube) -> bool {
            let v = c1 - c2;
            let (x,y,z) = (v.0.x.abs() as u32, v.0.y.abs() as u32, v.0.z.abs() as u32);
            c1.distance(c2) == max(x, max(y, z))
        }
        quickcheck(prop as fn(_,_) -> _);
    }

    #[test]
    fn prop_beeline_distance() {
        fn prop(c1: Cube, c2: Cube) -> bool {
            c1.beeline(c2).count() == c1.distance(c2) as usize + 1
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
            let v = c.range(r).collect::<Vec<Cube>>();
            v.iter().all(|n| c.distance(*n) <= r as u32)
                && v.contains(&c)
                && v.len() == Cube::num_in_range(r)
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
            let v  = c1.range_overlapping(c2, r).collect::<Vec<_>>();
            v.iter().all(|n| c1.range(r).any(|x| x == *n) &&
                             c2.range(r).any(|x| x == *n))
        }
        quickcheck(prop as fn(_) -> _);
    }

    #[test]
    fn prop_walk_ring() {
        fn prop(c: Cube, r: u16, d: flat::Direction) -> bool {
            let ring_cw = c.walk_ring(d, r, Rotation::CW).collect::<Vec<_>>();
            let (ring_cw_head, ring_cw_tail) = (ring_cw.first(), ring_cw.iter().skip(1));

            let ring_ccw = c.walk_ring(d, r, Rotation::CCW).collect::<Vec<_>>();
            let (ring_ccw_head, ring_ccw_tail) = (ring_ccw.first(), ring_ccw.iter().skip(1));

            ring_cw_head == ring_ccw_head
                &&
                ring_cw_tail.collect::<Vec<_>>()
                ==
                ring_ccw_tail.rev().collect::<Vec<_>>()
        }
        quickcheck(prop as fn(_,_,_) -> _);
    }

    #[test]
    fn prop_walk_range() {
        fn prop(c: Cube, r: u16) -> bool {
            c.walk_range(flat::Direction::North, r, Rotation::CW)
                .collect::<HashSet<_>>()
                ==
                c.range(r).collect::<HashSet<_>>()
        }
        quickcheck(prop as fn(_,_) -> _);
    }
}

