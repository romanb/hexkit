//! Geometry of regular hexagons in a 2d cartesian coordinate system.

use nalgebra::core::Matrix2;
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

#[derive(PartialEq, Copy, Clone, PartialOrd, Debug)]
pub struct SideLength(pub f32);

impl From<SideLength> for f32 {
    fn from(s: SideLength) -> f32 {
        s.0
    }
}

/// A schematic for a regular hexagon.
#[derive(Clone, Debug)]
pub struct Schema {
    pub(crate) side_len: SideLength,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) center_row_offset: f32,
    pub(crate) center_col_offset: f32,
               to_pixel: Matrix2<f32>,
               from_pixel: Matrix2<f32>,
    pub(crate) orientation: Orientation,
               first_corner_angle: f32,
}

impl Schema {
    pub fn new(side_len: SideLength, orientation: Orientation) -> Schema {
        let size = side_len.0;
        assert!(size > 0., "size <= 0");
        match orientation {
            Orientation::FlatTop => {
                let height = f32::sqrt(3.0) * size;
                let to_pixel = side_len.0 * Matrix2::new(
                    1.5,                0.,
                    f32::sqrt(3.) / 2., f32::sqrt(3.));
                let from_pixel = to_pixel.try_inverse().unwrap();
                Schema {
                    side_len,
                    orientation: Orientation::FlatTop,
                    width: 2.0 * size,
                    height,
                    center_col_offset: 1.5 * size,
                    center_row_offset: height,
                    to_pixel,
                    from_pixel,
                    first_corner_angle: 0.,
                }
            }
            Orientation::PointyTop => {
                let width = f32::sqrt(3.0) * size;
                let to_pixel = size * Matrix2::new(
                    f32::sqrt(3.), f32::sqrt(3.) / 2.,
                    0.0,           1.5);
                let from_pixel = to_pixel.try_inverse().unwrap();
                Schema {
                    side_len,
                    orientation: Orientation::PointyTop,
                    width,
                    height: 2.0 * size,
                    center_col_offset: width,
                    center_row_offset: 1.5 * size,
                    to_pixel,
                    from_pixel,
                    first_corner_angle: ANGLE_RADIANS / 2.,
                }
            }
        }
    }
}

impl Schema {
    pub fn side_len(&self) -> f32 {
        self.side_len.0
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn center_col_offset(&self) -> f32 {
        self.center_col_offset
    }

    pub fn center_row_offset(&self) -> f32 {
        self.center_row_offset
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub fn hexagon(&self, center: Point2<f32>) -> Hexagon {
        Hexagon {
            center,
            corners: self.corners(center, self.first_corner_angle),
        }
    }

    /// Compute the minimal bounding box of a hexagon.
    pub fn bounds(&self, h: &Hexagon) -> Bounds {
        Bounds {
            position: Point2::new(h.center.coords.x - self.width  / 2.,
                                  h.center.coords.y - self.height / 2.),
            width: self.width,
            height: self.height
        }
    }

    /// Convert the coordinates of a hexagon on an overlaid coordinate
    /// system into the pixel coordinates of the hexagon's center, with
    /// ```ignore
    /// s.to_pixel(Point2::origin()) == Point2::origin()
    /// ```
    /// for every schema `s`.
    pub fn to_pixel<P: Into<Point2<f32>>>(&self, p: P) -> Point2<f32> {
        let c = self.to_pixel * p.into().coords;
        Point2::from_coordinates(c)
    }

    /// Convert pixel coordinates into hexagon coordinates, satisfying
    /// ```ignore
    /// s.from_pixel(s.to_pixel(p)) == p
    /// ```
    /// for any point `p` and schema `s`.
    pub fn from_pixel<P: From<Point2<f32>>>(&self, p: Point2<f32>) -> P {
        let c = self.from_pixel * p.coords;
        P::from(Point2::from_coordinates(c))
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
        let x = center.x + self.side_len() * angle_rad.cos();
        let y = center.y + self.side_len() * angle_rad.sin();
        Point2::new(x, y)
    }
}

#[derive(PartialEq, Clone, Debug)]
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

pub struct Line([Point2<f32>; 2]);

impl Line {
    pub fn bounds(&self) -> Bounds {
        Bounds {
            position: Point2::new(f32::min(self.0[0].x, self.0[1].x),
                                  f32::min(self.0[0].y, self.0[1].y)),
            width: (self.0[0].x - self.0[1].x).abs(),
            height: (self.0[0].y - self.0[1].y).abs(),
        }
    }
}

/// A (minimal) bounding box for geometric shapes.
#[derive(Copy, Clone, Debug)]
pub struct Bounds {
    /// The top-left corner of the bounding box.
    pub position: Point2<f32>,
    pub width: f32,
    pub height: f32
}

impl Bounds {
    /// Check whether the two bounds intersect.
    pub fn intersects(&self, b: &Bounds) -> bool {
        self.position.x               < b.position.x + b.width  &&
        self.position.x + self.width  > b.position.x            &&
        self.position.y               < b.position.y + b.height &&
        self.position.y + self.height > b.position.y
    }

    /// Test whether a point lies within the bounds.
    pub fn contains(&self, p: Point2<f32>) -> bool {
        self.position.x <= p.x && p.x <= self.position.x + self.width
            &&
        self.position.y <= p.y && p.y <= self.position.y + self.height
    }

    /// Test whether the bounds lie completely within other bounds.
    pub fn within(&self, other: &Bounds) -> bool {
        let min_x = other.position.x;
        let max_x = min_x + other.width;
        let min_y = other.position.y;
        let max_y = min_y + other.height;
        min_x <= self.position.x && self.position.x + self.width <= max_x
            &&
        min_y <= self.position.y && self.position.y + self.height <= max_y
    }

    pub fn floor(&self) -> Bounds {
        Bounds {
            position: Point2::new(self.position.x.ceil(), self.position.y.ceil()),
            width: self.width.floor(),
            height: self.height.floor()
        }
    }
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
    use rand::Rng;
    use rand::seq::SliceRandom;

    impl Arbitrary for SideLength {
        fn arbitrary<G: Gen>(g: &mut G) -> SideLength {
            SideLength(g.gen_range(1., 100.))
        }
    }

    impl Arbitrary for Orientation {
        fn arbitrary<G: Gen>(g: &mut G) -> Orientation {
            *[Orientation::FlatTop, Orientation::PointyTop].choose(g).unwrap()
        }
    }

    impl Arbitrary for Rotation {
        fn arbitrary<G: Gen>(g: &mut G) -> Rotation {
            *[Rotation::CW, Rotation::CCW].choose(g).unwrap()
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

    #[test]
    fn prop_from_to_pixel_identity() {
        fn round(p: Point2<f32>) -> Point2<i16> {
            Point2::new(p.x.round() as i16, p.y.round() as i16)
        }
        fn prop(x: i16, y: i16, s: SideLength, o: Orientation) -> bool {
            let s = Schema::new(s, o);
            let p = Point2::new(x as f32, y as f32);
            round(s.from_pixel(s.to_pixel(p))) == round(p)
        }
        quickcheck(prop as fn(_,_,_,_) -> _);
    }

    #[test]
    fn prop_to_pixel_distance() {
        // The distances of the x and y coordinates of any
        // two hexagon center's must be a multiple of (half of)
        // the x respectively y distance between the centers
        // of adjacent hexagons, as defined by the schema.
        fn prop(cs: Vec<(i16,i16)>, s: SideLength, o: Orientation) -> bool {
            let s = Schema::new(s, o);
            cs.iter().all(|c1| {
                cs.iter().all(|c2| {
                    let p1 = s.to_pixel(Point2::new(c1.0 as f32, c1.1 as f32));
                    let p2 = s.to_pixel(Point2::new(c2.0 as f32, c2.1 as f32));
                    let dx = (p1.x - p2.x).abs();
                    let dy = (p1.y - p2.y).abs();
                    let nx = dx / (s.center_col_offset / 2.);
                    let ny = dy / (s.center_row_offset / 2.);
                    let ex = (nx - nx.round()).abs();
                    let ey = (ny - ny.round()).abs();
                    ex < 0.02 && ey < 0.02
                })
            })
        }
        quickcheck(prop as fn(_,_,_) -> _);
    }
}

