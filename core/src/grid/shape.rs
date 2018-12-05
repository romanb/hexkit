
use super::*;

#[derive(Clone)]
pub struct ShapeIter<I: Iterator<Item=Cube> + Clone> {
    pub iter: I,
    pub total: usize,
}

impl<I: Iterator<Item=Cube> + Clone> Iterator for ShapeIter<I> {
    type Item = Cube;

    fn next(&mut self) -> Option<Cube> {
        self.iter.next()
    }
}

/// A rectangular grid with even rows indented.
///
/// The orientation of the rectangle w.r.t. the orientation of the hexagons:
///
///   * `FlatTop`: Rotated 30 degrees clockwise.
///   * `PointyTop`: Axis-aligned.
pub fn rect_zx_even(rows: i32, cols: i32) -> ShapeIter<impl Iterator<Item=Cube> + Clone> {
    let iter = (0 .. rows).flat_map(move |r| {
        let r_offset = (r + 1) / 2;
        (-r_offset .. cols - r_offset).map(move |q| {
            Cube::new_xz(q, r)
        })
    });
    ShapeIter { iter, total: (rows * cols) as usize }
}

/// A rectangular grid with odd rows indented.
///
/// The orientation of the rectangle w.r.t. the orientation of the hexagons:
///
///   * `FlatTop`: Rotated 30 degrees clockwise.
///   * `PointyTop`: Axis-aligned.
pub fn rect_zx_odd(rows: i32, cols: i32) -> ShapeIter<impl Iterator<Item=Cube> + Clone> {
    let iter = (0 .. rows).flat_map(move |r| {
        let r_offset = r / 2;
        (-r_offset .. cols - r_offset).map(move |q| {
            Cube::new_xz(q, r)
        })
    });
    ShapeIter { iter, total: (rows * cols) as usize }
}

/// A rectangular grid with even columns indented.
///
/// The orientation of the rectangle w.r.t. the orientation of the hexagons:
///
///   * `FlatTop`: Axis-aligned.
///   * `PointyTop`: Rotated 60 degrees clockwise.
pub fn rect_xz_even(rows: i32, cols: i32) -> ShapeIter<impl Iterator<Item=Cube> + Clone> {
    let iter = (0 .. cols).flat_map(move |q| {
        let q_offset = (q + 1) / 2;
        (-q_offset .. rows - q_offset).map(move |r| {
            Cube::new_xz(q, r)
        })
    });
    ShapeIter { iter, total: (rows * cols) as usize }
}

/// A rectangular grid with odd columns indented.
///
/// The orientation of the rectangle w.r.t. the orientation of the hexagons:
///
///   * `FlatTop`: Axis-aligned.
///   * `PointyTop`: Rotated 60 degrees clockwise.
pub fn rect_xz_odd(rows: i32, cols: i32) -> ShapeIter<impl Iterator<Item=Cube> + Clone> {
    let iter = (0 .. cols).flat_map(move |q| {
        let q_offset = q / 2;
        (-q_offset .. rows - q_offset).map(move |r| {
            Cube::new_xz(q,r)
        })
    });
    ShapeIter { iter, total: (rows * cols) as usize }
}

