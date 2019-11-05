//! Iterators over cube coordinates for creating maps with common shapes.
//!
//! The `.` in the ASCII-art indicates the origin, i.e. `(0,0,0)`.

use super::coords::{ self, Cube };

#[derive(Clone)]
pub struct Shape<I: IntoIterator<Item=Cube>> {
    pub data: I,
    pub total: usize,
}

impl<I: IntoIterator<Item=Cube>> IntoIterator for Shape<I> {
    type Item = Cube;
    type IntoIter = I::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

//////////////////////////////////////////////////////////////////////////////
// Hexagon

/// A hexagon created by walking along rings around the origin in the center.
///
/// # Arguments
///
/// * `side_length` - The side length of the hexagon formed by the grid.
///
/// # Shapes
///
/// Flat-Top: A grid with the shape of a pointy-top hexagon.
/// ```raw
///   / \
///  /   \
/// |  .  |
/// |     |
///  \   /
///   \ /
/// ```
///
/// Pointy-Top: A grid with the shape of a flat-top hexagon.
/// ```raw
///   _____
///  /     \
/// /   .   \
/// \       /
///  \_____/
/// ```
pub fn hexagon(side_length: u16) -> Shape<impl Iterator<Item=Cube>> {
    assert!(side_length > 0, "side_length == 0");
    let data = coords::range(Cube::origin(), side_length - 1);
    Shape { data, total: coords::num_in_range(side_length - 1) }
}

//////////////////////////////////////////////////////////////////////////////
// Parallelograms

/// A parallelogram created by walking along `+x/+y` cube coordinates.
///
/// Flat-Top:
/// ```raw
///     _/|
///   _/  |
///  /    |
/// |     |
/// |   _/
/// | _/
/// ./
/// ```
///
/// Pointy-Top:
/// ```raw
///   / \
///  /   \
/// /     \
/// \     /
///  \   /
///   \./
/// ```
pub fn parallelogram_xy(rows: i32, cols: i32) -> Shape<impl Iterator<Item=Cube>> {
    let data = (0 .. cols).flat_map(move |x| {
        (0 .. rows).map(move |y| Cube::new_xy(x, y))
    });
    Shape { data, total: (rows * cols) as usize }
}

/// A parallelogram created by walking along `+x/+z` cube coordinates.
///
/// Flat-Top:
/// ```raw
/// .
/// |\_
/// |  \_
/// |    \
///  \_   |
///    \_ |
///      \|
/// ```
///
/// Pointy-Top:
/// ```raw
/// . ____
///  \    \
///   \    \
///    \____\
/// ```
pub fn parallelogram_xz(rows: i32, cols: i32) -> Shape<impl Iterator<Item=Cube>> {
    let data = (0 .. cols).flat_map(move |x| {
        (0 .. rows).map(move |z| Cube::new_xz(x, z))
    });
    Shape { data, total: (rows * cols) as usize }
}

/// A parallelogram created by walking along `+y/+z` cube coordinates.
///
/// Flat-Top:
/// ```raw
///   / \
///  /   \
/// /     \.
/// \     /
///  \   /
///   \ /
/// ```
///
/// Pointy-Top:
/// ```raw
///    _____.
///   /     /
///  /     /
/// /_____/
/// ```
pub fn parallelogram_yz(dy: i32, dz: i32) -> Shape<impl Iterator<Item=Cube>> {
    let data = (0 .. dy).flat_map(move |y| {
        (0 .. dz).map(move |z| Cube::new_yz(y, z))
    });
    Shape { data, total: (dy * dz) as usize }
}

//////////////////////////////////////////////////////////////////////////////
// Rectangles

/// A rectangle created by walking along `+x/-z..+z` cube coordinates.
///
/// Flat-Top (even columns staggered):
/// ```raw
///   0 1 2 3
///  . ___ __
///  |v   v  |
///  |       |
///  |       |
///  | ___ __|
///   v   v
/// ```
///
/// Pointy-Top: Flat-top rotated 30 degrees counterclockwise.
pub fn rectangle_xz_even(dx: i32, dz: i32) -> Shape<impl Iterator<Item=Cube>> {
    let data = (0 .. dx).flat_map(move |x| {
        let x_offset = (x + 1) / 2;
        (-x_offset .. dz - x_offset).map(move |z| {
            Cube::new_xz(x, z)
        })
    });
    Shape { data, total: (dx * dz) as usize }
}

/// A rectangle created by walking along `+x/-z..+z` cube coordinates.
///
/// Most commonly used with `Offset<OddCol>` coordinates.
///
/// Flat-Top (odd columns staggered):
/// ```raw
///   0 1 2 3
///  .__ ___
///  |  v   v|
///  |       |
///  |       |
///  |__ ___ |
///     v   v
/// ```
///
/// Pointy-Top: Flat-top rotated 30 degrees counterclockwise.
pub fn rectangle_xz_odd(dx: i32, dz: i32) -> Shape<impl Iterator<Item=Cube>> {
    let data = (0 .. dx).flat_map(move |x| {
        let x_offset = x / 2;
        (-x_offset .. dz - x_offset).map(move |z| {
            Cube::new_xz(x,z)
        })
    });
    Shape { data, total: (dx * dz) as usize }
}

/// A rectangle created by walking along `+z/-x..+x` cube coordinates.
///
/// Most commonly used with `Offset<EvenRow>` coordinates and a `PointyTop`
/// orientation.
///
/// Flat-Top: Pointy-top rotated 30 degrees clockwise.
///
/// Pointy-Top (even rows staggered):
/// ```raw
///     ._______
/// (0) >       >
/// (1) |       |
/// (2) >       >
/// (3) |_______|
/// ```
pub fn rectangle_zx_even(dz: i32, dx: i32) -> Shape<impl Iterator<Item=Cube>> {
    let data = (0 .. dz).flat_map(move |z| {
        let z_offset = (z + 1) / 2;
        (-z_offset .. dx - z_offset).map(move |x| {
            Cube::new_xz(x, z)
        })
    });
    Shape { data, total: (dz * dx) as usize }
}

/// A rectangle created by walking along `+z/-x..+x` cube coordinates.
///
/// Most commonly used with `Offset<OddRow>` coordinates and a `PointyTop`
/// orientation.
///
/// Flat-Top: Pointy-top rotated 30 degrees clockwise.
///
/// Pointy-Top (odd rows staggered):
/// ```raw
///   ._______
/// 0 |       |
/// 1 >       >
/// 2 |       |
/// 3 >_______>
/// ```
pub fn rectangle_zx_odd(dz: i32, dx: i32) -> Shape<impl Iterator<Item=Cube>> {
    let data = (0 .. dz).flat_map(move |z| {
        let z_offset = z / 2;
        (-z_offset .. dx - z_offset).map(move |x| {
            Cube::new_xz(x, z)
        })
    });
    Shape { data, total: (dz * dx) as usize }
}


/// A rectangle created by walking along `+x/-y..+y` cube coordinates.
///
/// Flat-Top: The same as `rectangle_xz_even`
/// with the origin at the bottom left corner.
///
/// Pointy-Top: Flat-top rotated 30 degrees counterclockwise.
pub fn rectangle_xy(dx: i32, dy: i32) -> Shape<impl Iterator<Item=Cube>> {
    let data = (0 .. dx).flat_map(move |x| {
        let x_offset = x / 2;
        (-x_offset .. dy - x_offset).map(move |y| {
            Cube::new_xy(x,y)
        })
    });
    Shape { data, total: (dx * dy) as usize }
}

/// A rectangle created by walking along `+z/-y..+y` cube coordinates.
///
/// Pointy-Top: The same as `rectangle_zx_even` with the origin at the
/// top-right corner.
///
/// Flat-Top: Pointy-top rotated 30 degrees clockwise.
pub fn rectangle_zy(dz: i32, dy: i32) -> Shape<impl Iterator<Item=Cube>> {
    let data = (0 .. dz).flat_map(move |z| {
        let z_offset = z / 2;
        (-z_offset .. dy - z_offset).map(move |y| {
            Cube::new_yz(y, z)
        })
    });
    Shape { data, total: (dy * dz) as usize }
}

/// A rectangle created by walking along `+y/-x..+x` cube coordinates.
///
/// Flat-Top: A pointy-top axis-aligned reactangle with the origin in
/// the bottom-left corner, rotated 30 degrees counterclockwise.
///
/// Pointy-Top: A flat-top axis-aligned grid with the origin at the
/// bottom-right corner, rotated 30 degrees clockwise.
pub fn rectangle_yx(dy: i32, dx: i32) -> Shape<impl Iterator<Item=Cube>> {
    let data = (0 .. dy).flat_map(move |y| {
        let y_offset = y / 2;
        (-y_offset .. dx - y_offset).map(move |x| {
            Cube::new_xy(x,y)
        })
    });
    Shape { data, total: (dx * dy) as usize }
}

/// A rectangle created by walking along `+y/-z..+z` cube coordinates.
///
/// Flat-Top: A pointy-top axis-aligned rectangle with the origin
/// in the bottom-right corner, rotated 30 degrees counterclockwise.
///
/// Pointy-Top: A flat-top axis-aligned rectangle with the origin
/// in the top-right corner, rotated 30 degrees clockwise.
pub fn rectangle_yz(dy: i32, dz: i32) -> Shape<impl Iterator<Item=Cube>> {
    let data = (0 .. dy).flat_map(move |y| {
        let y_offset = y / 2;
        (-y_offset .. dz - y_offset).map(move |z| {
            Cube::new_yz(y, z)
        })
    });
    Shape { data, total: (dy * dz) as usize }
}

//////////////////////////////////////////////////////////////////////////////
// Triangles

/// A triangle created by walking along `+x/-y` cube coordinates,
/// with the origin on the left.
///
/// # Arguments
///
///   * `dx` - The side length of the triangle, in hexagons.
///
/// # Shapes
///
/// Flat-Top:
/// ```raw
/// .
/// |\
/// | \
/// |  \
/// |  /
/// | /
/// |/
/// ```
///
/// Pointy-Top:
/// ```raw
/// ._______
/// \      /
///  \    /
///   \  /
///    \/
/// ```
pub fn triangle_xy(dx: i32) -> Shape<impl Iterator<Item=Cube>> {
    let data = (0 .. dx).flat_map(move |x| {
        (x .. dx).map(move |y| Cube::new_xy(x, -y))
    });
    Shape { data, total: (dx * (dx + 1) / 2) as usize }
}

/// A triangle created by walking along `-y/+x` cube coordinates,
/// with the origin on the left.
///
/// # Arguments
///
///   * `dy` - The side length of the triangle, in hexagons.
///
/// # Shapes
///
/// Flat-Top:
/// ```raw
///   /|
///  / |
/// .  |
///  \ |
///   \|
/// ```
///
/// Pointy-Top:
/// ```raw
///    /\
///   /  \
///  /    \
/// .______\
/// ```
pub fn triangle_yx(dy: i32) -> Shape<impl Iterator<Item=Cube>> {
    let data = (0 .. dy).flat_map(move |y| {
        (y .. dy).map(move |x| Cube::new_xy(x, -y))
    });
    Shape { data, total: (dy * (dy + 1) / 2) as usize }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::*;
    use rand::Rng;

    impl Arbitrary for Shape<Vec<Cube>> {
        fn arbitrary<G: Gen>(g: &mut G) -> Shape<Vec<Cube>> {
            let n1 = g.gen_range(0,64);
            let n2 = g.gen_range(0,64);
            let data: Vec<Cube> = match g.gen_range(0,10) {
                0 => rectangle_xz_even(n1, n2).data.collect(),
                1 => rectangle_xz_even(n1, n2).data.collect(),
                2 => rectangle_zx_odd(n1, n2).data.collect(),
                3 => rectangle_zx_even(n1, n2).data.collect(),
                4 => triangle_xy(n1).data.collect(),
                5 => triangle_yx(n1).data.collect(),
                6 => hexagon(n1 as u16 + 1).data.collect(),
                7 => parallelogram_xy(n1, n2).data.collect(),
                8 => parallelogram_xz(n1, n2).data.collect(),
                9 => parallelogram_yz(n1, n2).data.collect(),
                _ => Vec::new()
            };
            let total = data.len();
            Shape { data, total }
        }
    }
}

