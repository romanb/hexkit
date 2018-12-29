//! Cube coordinates.

pub mod vec;

use nalgebra::geometry::Point3;

use std::collections::HashSet;
use std::ops::{ Add, Sub };
use std::cmp::{ Ordering, min, max };
use std::iter;
use std::fmt;

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
/// The following illustrates the coordinate system with a flat-top orientation.
/// For a pointy-top orientation, it is to be rotated 30 degrees
/// counterclockwise, i.e. such that `+x/-y` and `+y/-x` are horizontally
/// aligned.
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
    pub fn distance(&self, other: Cube) -> usize { // TODO: usize
        ( (self.p.x - other.p.x).abs() as usize +
          (self.p.y - other.p.y).abs() as usize +
          (self.p.z - other.p.z).abs() as usize ) / 2
    }

    /// The shortest path to another cube coordinate, i.e. along
    /// a straight line, always including the start coordinate.
    pub fn beeline(&self, other: Cube) -> impl ExactSizeIterator<Item=Cube> + '_ {
        LineIterator {
            distance: self.distance(other),
            start: *self,
            end: other,
            current: 0
        }
    }

    /// The cube coordinates that are within the given range.
    pub fn range(&self, r: u16) -> impl Iterator<Item=Cube> + Clone {
        let x_end   = r as i32;
        let x_start = -x_end;
        let center = *self;
        (x_start ..= x_end).flat_map(move |x| {
            let y_start = max(x_start, -x - x_end);
            let y_end   = min(x_end,   -x + x_end);
            (y_start ..= y_end).map(move |y| {
                center + CubeVec::new_xy(x, y)
            })
        })
    }

    /// The number of cube coordinates that are within the given range.
    pub fn num_in_range(r: u16) -> usize {
        Self::num_in_ring(r) * (r as usize + 1) / 2 + 1
    }

    /// The number of cube coordinates that are in the ring of
    /// a given radius.
    pub fn num_in_ring(r: u16) -> usize {
        6 * (r as usize)
    }

    pub fn range_overlapping(&self, other: Cube, r: u16)
    -> impl Iterator<Item=Cube> + '_ {
        let n = r as i32;
        let x_min = max(self.x() - n, other.x() - n);
        let x_max = min(self.x() + n, other.x() + n);
        let y_min = max(self.y() - n, other.y() - n);
        let y_max = min(self.y() + n, other.y() + n);
        let z_min = max(self.z() - n, other.z() - n);
        let z_max = min(self.z() + n, other.z() + n);
        (x_min ..= x_max).flat_map(move |x| {
            let y_start = max(y_min, -x - z_max);
            let y_end   = min(y_max, -x - z_min);
            (y_start ..= y_end).map(move |y| Cube::new_xy(x, y))
        })
    }

    /// The cube coordinates that are within the given range and reachable.
    pub fn range_reachable<F>(&self, r: u16, f: F) -> HashSet<Cube>
    where F: Fn(Cube) -> bool {
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

    /// Iterator over the visible coordinates in the specified range,
    /// where visibility of a coordinate `c` is determined by checking
    /// whether all coordinates between `self` and `c` (as determined
    /// by `beeline`) satisfy the given predicate. The first blocked
    /// coordinate on a beeline is always considered visible.
    pub fn range_visible<F>(self, r: u16, f: F)
    -> impl Iterator<Item=Cube>
    where F: Fn(Cube) -> bool {
        self.range(r).filter(move |c| {
            let l = self.beeline(*c);
            let n = l.len(); // n > 0
            l.take(n - 1).all(|x| f(x))
        })
    }

    /// Iterate over the coordinates in the ring at a given distance
    /// from `self`, starting at the first coordinate of the ring in
    /// the given direction from `self` and walking along the ring
    /// as per the given `Rotation`.
    pub fn walk_ring<'a,D>(&'a self, dir: D, rad: u16, rot: Rotation)
    -> impl Iterator<Item=Cube> + 'a
    where D: Direction + 'a {
        let mut dirs = CubeVec::walk_directions(dir, rot);
        let dir1 = dirs.next().unwrap();
        RingIterator {
            radius: rad,
            pos: *self + CubeVec::direction(dir) * rad as i32,
            dir: dir1,
            dir_count: 0,
            dirs,
        }
    }

    pub fn walk_range<'a, D>(&'a self, dir: D, rad: u16, rot: Rotation)
    -> impl Iterator<Item=Cube> + 'a
    where D: Direction + 'a {
        let rings = (1..rad+1).flat_map(move |i| self.walk_ring(dir, i, rot));
        iter::once(*self).chain(rings)
    }

    fn mk(x: i32, y: i32, z: i32) -> Cube {
        debug_assert!(x + y + z == 0);
        Cube { p: Point3::new(x, y, z) }
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
    pub fn to_pixel(self, schema: &Schema) -> Point2<f32> {
        schema.to_pixel(self)
    }

    /// Compute the (nearest) cube coordinates for a point in the
    /// context of the given geometric schema, satisfying
    /// ```ignore
    /// Cube::from_pixel(c.to_point(&s), &s) == c
    /// ```
    /// for any cube coordinates `c` and schema `s`.
    pub fn from_pixel(p: Point2<f32>, schema: &Schema) -> Cube {
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

pub struct LineIterator {
    distance: usize,
    current: usize,
    start: Cube,
    end: Cube,
}

impl Iterator for LineIterator {
    type Item = Cube;

    fn next(&mut self) -> Option<Cube> {
        if self.distance > 0 && self.current <= self.distance {
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

pub struct RingIterator<I: Iterator<Item=CubeVec>> {
    pos: Cube,
    dirs: I,
    dir: CubeVec,
    radius: u16,
    dir_count: u16,
}

impl<I: ExactSizeIterator<Item=CubeVec>> Iterator for RingIterator<I> {
    type Item = Cube;

    fn next(&mut self) -> Option<Cube> {
        if self.radius == 0 {
            return None
        }
        if self.dir_count >= self.radius {
            self.dirs.next().and_then(|dir| {
                self.dir = dir;
                self.dir_count = 0;
                self.next()
            })
        } else {
            let pos = self.pos;
            self.dir_count += 1;
            self.pos = self.pos + self.dir;
            Some(pos)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = {
            self.dirs.len() as u16 * self.radius + self.radius - self.dir_count
        } as usize;
        (remaining, Some(remaining))
    }
}

impl<I: ExactSizeIterator<Item=CubeVec>> ExactSizeIterator
for RingIterator<I> {}

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
    use crate::grid::*;
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
            let (x,y,z) = (
                v.0.x.abs() as usize,
                v.0.y.abs() as usize,
                v.0.z.abs() as usize
            );
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
            v.iter().all(|n| c.distance(*n) <= r as usize)
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
        fn prop(c: Cube, r: u16, d: FlatTopDirection) -> bool {
            let cw = c.walk_ring(d, r, Rotation::CW).collect::<Vec<_>>();
            let (cw_head, cw_tail) = (cw.first(), cw.iter().skip(1));

            let ccw = c.walk_ring(d, r, Rotation::CCW).collect::<Vec<_>>();
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
            c.walk_range(d, r, rot).collect::<HashSet<_>>()
                ==
            c.range(r).collect()
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
            c.range_visible(r % 32, |_| true).collect::<HashSet<_>>()
                ==
            c.range(r % 32).collect()
        }
        quickcheck(prop as fn(_,_) -> _);
    }

    #[test]
    fn prop_range_visible_blocked_dir() {
        fn prop(c: Cube, r: u16, d: FlatTopDirection) -> bool {
            let range = (r % 32) + 1;
            let blocked = c + d.vector();
            let visible = c.range_visible(range, |x| x != blocked).collect::<HashSet<_>>();
            // All coordinates in the direction of (and beyond)
            // the blocked neighbour are expected not to be visible.
            let blocked_end = c + d.vector() * range as i32;
            visible.contains(&blocked)
                &&
            c.beeline(blocked_end)
                .skip(1) // skip the origin
                .all(|x| x == blocked || !visible.contains(&x))
        }
        quickcheck(prop as fn(_,_,_) -> _);
    }
}

