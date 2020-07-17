
pub mod cube;
pub mod axial;
pub mod offset;

pub use cube::*;
pub use axial::*;
pub use offset::*;

use crate::geo;

use std::collections::HashSet;
use std::cmp::{ min, max };
use std::fmt::{ Debug, Display };
use std::hash::Hash;
use std::iter;

/// TODO
pub trait Coords:
    From<Cube> + Into<Cube> + Eq + Copy + Debug + Display + Hash
{}

/// Iterate over the neighbouring (adjacent) coordinates.
pub fn neighbours<C>(c: C) -> impl Iterator<Item=C>
where
    C: Coords
{
    CubeVec::directions().map(move |v| C::from(c.into() + v))
}

/// Iterate over the neighbouring coordinates along the diagonal axes.
pub fn diagonal_neighbours<C>(c: C) -> impl Iterator<Item=C>
where
    C: Coords
{
    CubeVec::diagonals().map(move |v| C::from(c.into() + v))
}

/// The (beeline) distance between coordinates.
pub fn distance<C>(from: C, to: C) -> usize
where
    C: Coords
{
    let a: Cube = from.into();
    let b: Cube = to.into();
    ( (a.p.x - b.p.x).abs() as usize +
      (a.p.y - b.p.y).abs() as usize +
      (a.p.z - b.p.z).abs() as usize ) / 2
}

/// The shortest path to other coordinates along a straight line,
/// always including the start coordinates.
pub fn beeline<C>(from: C, to: C) -> impl ExactSizeIterator<Item=C>
where
    C: Coords
{
    LineIterator {
        distance: distance(from, to),
        start: from,
        end: to,
        current: 0
    }
}

/// TODO
pub struct LineIterator<C> {
    distance: usize,
    current: usize,
    start: C,
    end: C,
}

impl<C> Iterator for LineIterator<C>
where
    C: Coords
{
    type Item = C;

    fn next(&mut self) -> Option<C> {
        if self.distance > 0 && self.current <= self.distance {
            let frac = geo::Frac1::new(self.current as f32, self.distance as f32);
            let next = lerp(self.start, self.end, frac);
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

impl<C: Coords> ExactSizeIterator for LineIterator<C> {}

pub fn lerp<C>(from: C, to: C, t: geo::Frac1) -> C
where
    C: Coords
{
    let a: Cube = from.into();
    let b: Cube = to.into();
    let x = geo::lerp(a.x(), b.x(), t);
    let y = geo::lerp(a.y(), b.y(), t);
    let z = geo::lerp(a.z(), b.z(), t);
    C::from(Cube::round(x, y, z))
}

/// The number of coordinates that are within the given range.
pub fn num_in_range(r: u16) -> usize {
    num_in_ring(r) * (r as usize + 1) / 2 + 1
}

/// The number of coordinates that are in a ring of a given radius.
pub fn num_in_ring(r: u16) -> usize {
    6 * (r as usize)
}

/// The coordinates that are within the specified range of the given
/// coordinates.
pub fn range<C>(c: C, r: u16) -> impl Iterator<Item=C> + Clone
where
    C: Coords
{
    let x_end   = r as i32;
    let x_start = -x_end;
    let center = c.into();
    (x_start ..= x_end).flat_map(move |x| {
        let y_start = max(x_start, -x - x_end);
        let y_end   = min(x_end,   -x + x_end);
        (y_start ..= y_end).map(move |y| {
            C::from(center + CubeVec::new_xy(x, y))
        })
    })
}

/// TODO
pub fn range_overlapping<C>(c1: C, c2: C, r: u16) -> impl Iterator<Item=C>
where
    C: Coords
{
    let n = r as i32;
    let a: Cube = c1.into();
    let b: Cube = c2.into();
    let x_min = max(a.x() - n, b.x() - n);
    let x_max = min(a.x() + n, b.x() + n);
    let y_min = max(a.y() - n, b.y() - n);
    let y_max = min(a.y() + n, b.y() + n);
    let z_min = max(a.z() - n, b.z() - n);
    let z_max = min(a.z() + n, b.z() + n);
    (x_min ..= x_max).flat_map(move |x| {
        let y_start = max(y_min, -x - z_max);
        let y_end   = min(y_max, -x - z_min);
        (y_start ..= y_end).map(move |y| C::from(Cube::new_xy(x, y)))
    })
}

/// The cube coordinates that are within the given range and reachable.
pub fn range_reachable<C, F>(c: C, r: u16, f: F) -> HashSet<C>
where
    C: Coords,
    F: Fn(C) -> bool
{
    let mut reachable = HashSet::new();
    let mut fringe = Vec::new();
    reachable.insert(c);
    fringe.push(c);
    for _ in 1..(r as usize + 1) {
        let mut fringe_i = Vec::new();
        for c in fringe {
            for cn in neighbours(c) {
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

/// Returns an iterator over the visible coordinates in the specified range.
///
/// Visibility of a coordinate `c` is determined by checking
/// whether all coordinates between `self` and `c` (as determined
/// by [`beeline`]) satisfy the given predicate. The first blocked
/// coordinate on a beeline is always considered visible.
pub fn range_visible<C, F>(c: C, r: u16, f: F) -> impl Iterator<Item=C>
where
    C: Coords,
    F: Fn(C) -> bool
{
    range(c, r).filter(move |x| {
        let l = beeline(c, *x);
        let n = l.len(); // n > 0
        l.take(n - 1).all(|x| f(x))
    })
}

/// Iterate over the coordinates in the ring at a given distance
/// from `self`, starting at the first coordinate of the ring in
/// the given direction from `self` and walking along the ring
/// as per the given `Rotation`.
pub fn walk_ring<C, D>(c: C, dir: D, rad: u16, rot: geo::Rotation) -> impl Iterator<Item=C>
where
    C: Coords,
    D: Direction
{
    let mut dirs = CubeVec::walk_directions(dir, rot);
    let dir1 = dirs.next().unwrap();
    RingIterator {
        radius: rad,
        pos: C::from(c.into() + CubeVec::direction(dir) * rad as i32),
        dir: dir1,
        dir_count: 0,
        dirs,
    }
}

/// TODO
pub struct RingIterator<C, I: Iterator<Item=CubeVec>> {
    pos: C,
    dirs: I,
    dir: CubeVec,
    radius: u16,
    dir_count: u16,
}

impl<C, I> Iterator for RingIterator<C,I>
where
    C: Coords,
    I: ExactSizeIterator<Item=CubeVec>
{
    type Item = C;

    fn next(&mut self) -> Option<C> {
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
            self.pos = C::from(self.pos.into() + self.dir);
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

impl<C, I> ExactSizeIterator for RingIterator<C,I>
where
    C: Coords,
    I: ExactSizeIterator<Item=CubeVec>
{}

pub fn walk_range<C, D>(c: C, dir: D, rad: u16, rot: geo::Rotation) -> impl Iterator<Item=C>
where
    C: Coords,
    D: Direction
{
    let rings = (1 .. rad + 1).flat_map(move |i| walk_ring(c, dir, i, rot));
    iter::once(c).chain(rings)
}

