//! Geometry of regular hexagons in a 2d cartesian coordinate system.

use nalgebra::geometry::Point2;
use num_traits::cast::{ FromPrimitive, ToPrimitive };
use std::ops::{ Neg, Add, Sub };

/// The angle (in degrees) of the equilateral triangles that
/// a regular hexagon is composed of, i.e. 60 degrees.
pub const ANGLE_DEGREES: f32 = 60.0;

/// The angle (in radians) of the equilateral triangles that
/// a hexagon is composed of, i.e. 60 degrees in radians.
pub const ANGLE_RADIANS: f32 = 1.0471975512;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Orientation {
    FlatTop,
    PointyTop
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Rotation {
    /// Clockwise roation.
    CW,
    /// Counterclockwise rotation.
    CCW
}

/// A schematic for a regular hexagon.
#[derive(Clone, Debug)]
pub struct Schema {
    pub(crate) size: f32, // side_length
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) center_xoffset: f32,
    pub(crate) center_yoffset: f32,
    pub(crate) orientation: Orientation,
}

impl Schema {
    pub fn new(size: f32, orientation: Orientation) -> Schema {
        match orientation {
            Orientation::FlatTop => {
                let height = f32::sqrt(3.0) * size;
                Schema {
                    size,
                    width: 2.0 * size,
                    height,
                    center_xoffset: 1.5 * size,
                    center_yoffset: height,
                    orientation,
                }
            }
            Orientation::PointyTop => {
                let width = f32::sqrt(3.0) * size;
                Schema {
                    size,
                    height: 2.0 * size,
                    width,
                    center_xoffset: width,
                    center_yoffset: 1.5 * size,
                    orientation,
                }
            }
        }
    }

    // side_length
    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn center_xoffset(&self) -> f32 {
        self.center_xoffset
    }

    pub fn center_yoffset(&self) -> f32 {
        self.center_yoffset
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub fn hexagon(&self, center: Point2<f32>) -> Hexagon {
        match self.orientation {
            Orientation::FlatTop => Hexagon {
                center,
                corners: self.corners(center, 0.)
            },
            Orientation::PointyTop => Hexagon {
                center,
                corners: self.corners(center, ANGLE_RADIANS / 2.)
            }
        }
    }

    /// Compute the rectangular bounds of a hexagon on a grid.
    pub fn bounds(&self, h: &Hexagon) -> HexBounds {
        HexBounds {
            x: h.center.coords.x - self.width / 2.,
            y: h.center.coords.y - self.height / 2.,
            width: self.width,
            height: self.height
        }
    }

    fn corners(&self, center: Point2<f32>, off: f32) -> [Point2<f32>; 6] {
        [ self.corner(center, 0, off)
        , self.corner(center, 1, off)
        , self.corner(center, 2, off)
        , self.corner(center, 3, off)
        , self.corner(center, 4, off)
        , self.corner(center, 5, off)
        ]
    }

    fn corner(&self, center: Point2<f32>, i: u8, off: f32) -> Point2<f32> {
        let angle_rad = ANGLE_RADIANS * i as f32 - off;
        let x = center.x + self.size * angle_rad.cos();
        let y = center.y + self.size * angle_rad.sin();
        Point2::new(x, y)
    }
}

#[derive(Clone, Debug)]
pub struct Hexagon {
    pub(crate) center: Point2<f32>,
    pub(crate) corners: [Point2<f32>; 6],
}

impl Hexagon {
    pub fn center(&self) -> Point2<f32> {
        self.center
    }

    pub fn center_x(&self) -> f32 {
        self.center.coords.x
    }

    pub fn center_y(&self) -> f32 {
        self.center.coords.y
    }

    pub fn corners(&self) -> &[Point2<f32>; 6] {
        &self.corners
    }

    pub fn gauge(&self, w: f32, h: f32) -> Point2<f32> {
        let x = self.center.coords.x - w / 2.;
        let y = self.center.coords.y - h / 2.;
        Point2::new(x, y)
    }
}

/// The rectangular bounds of a hexagon.
pub struct HexBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32
}

/// The additive group of integers modulo 6, i.e. Z/6Z,
/// which is isomorphic to the group of rotational symmetries
/// of a regular hexagon.
#[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum Z6 {
    Zero  = 0,
    One   = 1,
    Two   = 2,
    Three = 3,
    Four  = 4,
    Five  = 5,
}

impl Neg for Z6 {
    type Output = Z6;
    fn neg(self) -> Z6 {
        match self {
            Z6::Zero  => Z6::Zero,
            Z6::One   => Z6::Five,
            Z6::Two   => Z6::Four,
            Z6::Three => Z6::Three,
            Z6::Four  => Z6::Two,
            Z6::Five  => Z6::One,
        }
    }
}

impl Add<Z6> for Z6 {
    type Output = Z6;
    fn add(self, z: Z6) -> Z6 {
        let z1 = self.to_u8().unwrap();
        let z2 = z.to_u8().unwrap();
        Z6::from_u8((z1 + z2) % 6).unwrap()
    }
}

impl Sub<Z6> for Z6 {
    type Output = Z6;
    fn sub(self, z: Z6) -> Z6 {
        self + (-z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::*;

    impl Arbitrary for Orientation {
        fn arbitrary<G: Gen>(g: &mut G) -> Orientation {
            if g.gen() {
                Orientation::FlatTop
            } else {
                Orientation::PointyTop
            }
        }
    }

    impl Arbitrary for Rotation {
        fn arbitrary<G: Gen>(g: &mut G) -> Rotation {
            if g.gen() {
                Rotation::CW
            } else {
                Rotation::CCW
            }
        }
    }

    impl Arbitrary for Z6 {
        fn arbitrary<G: Gen>(g: &mut G) -> Z6 {
            Z6::from_u8(g.gen_range(0,6)).unwrap()
        }
    }

    #[test]
    fn test_z6_add() {
        for i in 0..6 {
            let z1 = Z6::from_u8(i).unwrap();
            for j in i..6 {
                let z2 = Z6::from_u8(j).unwrap();
                assert!((z1 + z2).to_u8() == Some((i + j) % 6))
            }
        }
    }

    #[test]
    fn prop_z6_associativity() {
        fn prop(z1: Z6, z2: Z6, z3: Z6) -> bool {
            (z1 + z2) + z3 == z1 + (z2 + z3)
        }
        quickcheck(prop as fn(_,_,_) -> _);
    }

    #[test]
    fn prop_z6_inverses() {
        fn prop(z: Z6) -> bool {
            z - z == Z6::Zero
        }
        quickcheck(prop as fn(_) -> _);
    }

    #[test]
    fn prop_z6_commutativity() {
        fn prop(z1: Z6, z2: Z6) -> bool {
            z1 + z2 == z2 + z1
        }
        quickcheck(prop as fn(_,_) -> _);
    }
}

