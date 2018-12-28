//! Offset coordinates.

use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use super::*;

pub trait OffsetType: Debug + Hash + Eq + Copy + Clone + Send + 'static {}

/// Offset coordinates.
///
/// Guide: [Offset Coordinates]
///
/// [Offset Coordinates]: https://www.redblobgames.com/grids/hexagons/#coordinates-offset
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Offset<T: OffsetType> {
    pub col: i32,
    pub row: i32,
        _ty: PhantomData<T>,
}

impl<T: OffsetType> Coords for Offset<T>
where Offset<T>: From<Cube> + Into<Cube> {}

/// The type of offset coordinates that yield strictly
/// non-negative coordinates when used with a
/// flat-top orientation for the axis-aligned rectangular
/// grid constructed by `grid::shape::rectangle_xz_odd`.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct OddCol;
impl OffsetType for OddCol {}

/// The type of offset coordinates that yield strictly
/// non-negative coordinates when used with a
/// pointy-top orientation for the axis-aligned rectangular
/// grid constructed by `grid::shape::rectangle_zx_odd`.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct OddRow;
impl OffsetType for OddRow {}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// The type of offset coordinates that yield strictly
/// non-negative coordinates when used with a
/// flat-top orientation for the axis-aligned rectangular
/// grid constructed by `grid::shape::rectangle_xz_even`.
pub struct EvenCol;
impl OffsetType for EvenCol {}

/// The type of offset coordinates that yield strictly
/// non-negative coordinates when used with a
/// pointy-top orientation for the axis-aligned rectangular
/// grid constructed by `grid::shape::rectangle_zx_even`.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct EvenRow;
impl OffsetType for EvenRow {}

impl<T: OffsetType> Offset<T> {
    pub fn new(col: i32, row: i32) -> Offset<T> {
        Offset { col, row, _ty: PhantomData }
    }
}

impl From<Cube> for Offset<OddCol> {
    fn from(c: Cube) -> Self {
        let col = c.x();
        let row = c.z() + (col - (col & 1)) / 2;
        Offset { col, row, _ty: PhantomData }
    }
}

impl From<Offset<OddCol>> for Cube {
    fn from(o: Offset<OddCol>) -> Cube {
        let z = o.row - ((o.col - (o.col & 1)) / 2);
        Cube::new_xz(o.col, z)
    }
}

impl From<Cube> for Offset<EvenCol> {
    fn from(c: Cube) -> Self {
        let col = c.x();
        let row = c.z() + (col + (col & 1)) / 2;
        Offset { col, row, _ty: PhantomData }
    }
}

impl From<Offset<EvenCol>> for Cube {
    fn from(o: Offset<EvenCol>) -> Cube {
        let z = o.row - (o.col + (o.col & 1)) / 2;
        Cube::new_xz(o.col, z)
    }
}

impl From<Cube> for Offset<OddRow> {
    fn from(c: Cube) -> Self {
        let row = c.z();
        let col = c.x() + (row - (row & 1)) / 2;
        Offset { col, row, _ty: PhantomData }
    }
}

impl From<Offset<OddRow>> for Cube {
    fn from(o: Offset<OddRow>) -> Cube {
        let x = o.col - (o.row - (o.row & 1)) / 2;
        Cube::new_xz(x, o.row)
    }
}

impl From<Cube> for Offset<EvenRow> {
    fn from(c: Cube) -> Self {
        let row = c.z();
        let col = c.x() + (row + (row & 1)) / 2;
        Offset { col, row, _ty: PhantomData }
    }
}

impl From<Offset<EvenRow>> for Cube {
    fn from(o: Offset<EvenRow>) -> Cube {
        let x = o.col - (o.row + (o.row & 1)) / 2;
        Cube::new_xz(x, o.row)
    }
}

impl<T: OffsetType> fmt::Display for Offset<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.col, self.row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::*;

    #[test]
    fn prop_from_to_cube_identity() {
        fn prop(c: Cube) -> bool {
            Cube::from(Offset::<OddCol>::from(c))  == c &&
            Cube::from(Offset::<OddRow>::from(c))  == c &&
            Cube::from(Offset::<EvenCol>::from(c)) == c &&
            Cube::from(Offset::<EvenRow>::from(c)) == c
        }
        quickcheck(prop as fn(_)  -> _);
    }
}

