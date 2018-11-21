//! Directions in the cube coordinate system.
//!
//! ...

pub use nalgebra::core::Vector3;
pub use std::ops::{ Add, Mul, Neg };

/// Vectors for the displacement to a neighbouring (adjacent) cube coordinate
/// along one of the sides of a hexagon.
pub const CUBE_DIR_VECTORS: [ [i32; 3]; 6] =
    [ [0,  1, -1], [ 1, 0, -1], [ 1, -1, 0]
    , [0, -1,  1], [-1, 0,  1], [-1,  1, 0]
    ];

/// Vectors for the displacement to the nearest cube coordinate
/// along one of the diagonal axes of a hexagon.
pub const CUBE_DIA_VECTORS: [[i32; 3]; 6] =
    [ [-1,  2, -1], [ 1,  1, -2], [ 2, -1, -1]
    , [ 1, -2,  1], [-1, -1,  2], [-2,  1,  1]
    ];

mod tests {
    use super::*;

    #[test]
    fn test_cube_vectors_valid() {
        for [x,y,z] in &CUBE_DIR_VECTORS {
            assert!(x + y + z == 0)
        }
        for [x,y,z] in &CUBE_DIA_VECTORS {
            assert!(x + y + z == 0)
        }
    }
}

/// Directions for hexagons with flat-top orientation in
/// the cube coordinate system.
pub mod flat {
    use super::*;

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

    impl Direction {
        pub fn vector(&self) -> Vector3<i32> {
            Vector3::from(CUBE_DIR_VECTORS[*self as usize])
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
        pub fn vector(&self) -> Vector3<i32> {
            Vector3::from(CUBE_DIA_VECTORS[*self as usize])
        }
    }

    #[cfg(test)]
    mod tests {
        use std::iter;
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

    impl Direction {
        pub fn vector(self) -> Vector3<i32> {
            Vector3::from(CUBE_DIR_VECTORS[self as usize])
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
        pub fn vector(self) -> Vector3<i32> {
            Vector3::from(CUBE_DIA_VECTORS[self as usize])
        }
    }

    #[cfg(test)]
    mod tests {
        use std::iter;
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

