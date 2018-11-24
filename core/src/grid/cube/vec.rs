//! Directions in the cube coordinate system.
//!
//! ...

pub use nalgebra::core::Vector3;
pub use std::ops::{ Add, Sub, Mul, Neg };
pub use geo::{ Z6, Rotation };

use either::Either;

/// Vectors for the displacement to a neighbouring (adjacent) cube coordinate
/// along one of the sides of a hexagon.
const CUBE_DIR_VECTORS: [ [i32; 3]; 6] =
    [ [0,  1, -1], [ 1, 0, -1], [ 1, -1, 0]
    , [0, -1,  1], [-1, 0,  1], [-1,  1, 0]
    ];

/// Vectors for the displacement to the nearest cube coordinate
/// along one of the diagonal axes of a hexagon.
const CUBE_DIA_VECTORS: [[i32; 3]; 6] =
    [ [-1,  2, -1], [ 1,  1, -2], [ 2, -1, -1]
    , [ 1, -2,  1], [-1, -1,  2], [-2,  1,  1]
    ];

// TODO: Rename to Direction and remove flat/pointy submodules,
// instead using FlatTopDirection, PointyTopDirection etc. for
// the enums.
pub trait DirIndex: Copy + Clone {
    fn index(self) -> Z6;
    fn vector(self) -> CubeVec;
}

/// A displacement of cube coordinates.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct CubeVec(pub(super) Vector3<i32>);

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

    pub fn directions() -> impl DoubleEndedIterator<Item=CubeVec> + Clone {
        CUBE_DIR_VECTORS.iter().map(|v| CubeVec(Vector3::from(*v)))
    }

    pub fn direction<D: DirIndex>(d: D) -> CubeVec {
        CubeVec(Vector3::from(CUBE_DIR_VECTORS[d.index() as usize]))
    }

    pub fn diagonals() -> impl DoubleEndedIterator<Item=CubeVec> {
        CUBE_DIA_VECTORS.iter().map(|v| CubeVec(Vector3::from(*v)))
    }

    pub fn walk_directions<D>(d: D, r: Rotation) -> impl Iterator<Item=CubeVec>
        where D: DirIndex
    {
        let dirs = Self::directions();
        match r {
            Rotation::CW  => Either::Left(
                dirs.cycle().skip((d.index() + Z6::Two) as usize).take(6)
            ),
            Rotation::CCW => Either::Right(
                dirs.rev().cycle().skip((Z6::One - d.index()) as usize).take(6)
            )
        }
    }

    /// Rotate the vector `n` times by 60 degrees in the given direction.
    pub fn rotate(&self, r: Rotation, n: Z6) -> CubeVec {
        match r {
            Rotation::CW  => self.rotate(Rotation::CCW, -n),
            Rotation::CCW => match n {
                Z6::Zero  => *self,
                Z6::One   => CubeVec::new_xy(-self.0.y, -self.0.z),
                Z6::Two   => CubeVec::new_xy( self.0.z,  self.0.x),
                Z6::Three => CubeVec::new_xy(-self.0.x, -self.0.y),
                Z6::Four  => CubeVec::new_xy( self.0.y,  self.0.z),
                Z6::Five  => CubeVec::new_xy(-self.0.z, -self.0.x)
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::*;

    #[test]
    fn test_cube_vectors_valid() {
        for [x,y,z] in &CUBE_DIR_VECTORS {
            assert!(x + y + z == 0)
        }
        for [x,y,z] in &CUBE_DIA_VECTORS {
            assert!(x + y + z == 0)
        }
    }

    #[test]
    fn prop_vec_rotate() {
        fn prop(v: CubeVec, z: Z6) -> bool {
            v.rotate(Rotation::CW, z) == v.rotate(Rotation::CCW, Z6::Zero - z)
        }
        quickcheck(prop as fn(_,_) -> _)
    }
}

/// Directions for hexagons with flat-top orientation in
/// the cube coordinate system.
pub mod flat {
    use super::*;
    use num_traits::cast::FromPrimitive;

    /// Directions for adjacent neighbours.
    #[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
    #[derive(FromPrimitive, Debug)]
    pub enum Direction {
        North     = 0,
        NorthEast = 1,
        SouthEast = 2,
        South     = 3,
        SouthWest = 4,
        NorthWest = 5
    }

    impl DirIndex for Direction {
        fn vector(self) -> CubeVec {
            CubeVec(Vector3::from(CUBE_DIR_VECTORS[self as usize]))
        }

        fn index(self) -> Z6 {
            Z6::from_u8(self as u8).unwrap()
        }
    }

    /// Directions for diagonal neighbours.
    #[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
    #[derive(FromPrimitive, Debug)]
    pub enum Diagonal {
        NorthWest = 0,
        NorthEast = 1,
        East      = 2,
        SouthEast = 3,
        SouthWest = 4,
        West      = 5
    }

    impl Diagonal {
        pub fn vector(self) -> CubeVec {
            CubeVec(Vector3::from(CUBE_DIA_VECTORS[self as usize]))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use quickcheck::*;
        use num_traits::cast::FromPrimitive;

        impl Arbitrary for Direction {
            fn arbitrary<G: Gen>(g: &mut G) -> Direction {
                Direction::from_u8(g.gen_range(0,6)).unwrap()
            }
        }

        impl Arbitrary for Diagonal {
            fn arbitrary<G: Gen>(g: &mut G) -> Diagonal {
                Diagonal::from_u8(g.gen_range(0,6)).unwrap()
            }
        }
    }
}

/// Directions for hexagons with pointy-top orientation in
/// the cube coordinate system.
pub mod pointy {
    use super::*;
    use num_traits::cast::FromPrimitive;

    /// Directions for adjacent neighbours.
    #[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
    #[derive(Debug, FromPrimitive)]
    pub enum Direction {
        NorthWest = 0,
        NorthEast = 1,
        East      = 2,
        SouthEast = 3,
        SouthWest = 4,
        West      = 5
    }

    impl DirIndex for Direction {
        fn vector(self) -> CubeVec {
            CubeVec(Vector3::from(CUBE_DIR_VECTORS[self as usize]))
        }

        fn index(self) -> Z6 {
            Z6::from_u8(self as u8).unwrap()
        }
    }

    /// Directions for diagonal neighbours.
    #[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
    #[derive(Debug, FromPrimitive)]
    pub enum Diagonal {
        NorthWest = 0,
        North     = 1,
        NorthEast = 2,
        SouthEast = 3,
        South     = 4,
        SouthWest = 5
    }

    impl Diagonal {
        pub fn vector(self) -> CubeVec {
            CubeVec(Vector3::from(CUBE_DIA_VECTORS[self as usize]))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use quickcheck::*;
        use num_traits::cast::FromPrimitive;

        impl Arbitrary for Direction {
            fn arbitrary<G: Gen>(g: &mut G) -> Direction {
                Direction::from_u8(g.gen_range(0,6)).unwrap()
            }
        }

        impl Arbitrary for Diagonal {
            fn arbitrary<G: Gen>(g: &mut G) -> Diagonal {
                Diagonal::from_u8(g.gen_range(0,6)).unwrap()
            }
        }
    }

}

