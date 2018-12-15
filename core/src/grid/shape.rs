
use super::*;

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

/// A rectangular grid with even rows indented.
///
/// The orientation of the rectangle w.r.t. the orientation of the hexagons:
///
///   * `FlatTop`: Rotated 30 degrees clockwise.
///   * `PointyTop`: Axis-aligned.
pub fn rect_zx_even(rows: i32, cols: i32) -> Shape<impl Iterator<Item=Cube> + Clone> {
    let data = (0 .. rows).flat_map(move |r| {
        let r_offset = (r + 1) / 2;
        (-r_offset .. cols - r_offset).map(move |q| {
            Cube::new_xz(q, r)
        })
    });
    Shape { data, total: (rows * cols) as usize }
}

/// A rectangular grid with odd rows indented.
///
/// The orientation of the rectangle w.r.t. the orientation of the hexagons:
///
///   * `FlatTop`: Rotated 30 degrees clockwise.
///   * `PointyTop`: Axis-aligned.
pub fn rect_zx_odd(rows: i32, cols: i32) -> Shape<impl Iterator<Item=Cube> + Clone> {
    let data = (0 .. rows).flat_map(move |r| {
        let r_offset = r / 2;
        (-r_offset .. cols - r_offset).map(move |q| {
            Cube::new_xz(q, r)
        })
    });
    Shape { data, total: (rows * cols) as usize }
}

/// A rectangular grid with even columns indented.
///
/// The orientation of the rectangle w.r.t. the orientation of the hexagons:
///
///   * `FlatTop`: Axis-aligned.
///   * `PointyTop`: Rotated 60 degrees clockwise.
pub fn rect_xz_even(rows: i32, cols: i32) -> Shape<impl Iterator<Item=Cube> + Clone> {
    let data = (0 .. cols).flat_map(move |q| {
        let q_offset = (q + 1) / 2;
        (-q_offset .. rows - q_offset).map(move |r| {
            Cube::new_xz(q, r)
        })
    });
    Shape { data, total: (rows * cols) as usize }
}

/// A rectangular grid with odd columns indented.
///
/// The orientation of the rectangle w.r.t. the orientation of the hexagons:
///
///   * `FlatTop`: Axis-aligned.
///   * `PointyTop`: Rotated 60 degrees clockwise.
pub fn rect_xz_odd(rows: i32, cols: i32) -> Shape<impl Iterator<Item=Cube> + Clone> {
    let data = (0 .. cols).flat_map(move |q| {
        let q_offset = q / 2;
        (-q_offset .. rows - q_offset).map(move |r| {
            Cube::new_xz(q,r)
        })
    });
    Shape { data, total: (rows * cols) as usize }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::*;
    use rand::Rng;

    impl Arbitrary for Shape<Vec<Cube>> {
        fn arbitrary<G: Gen>(g: &mut G) -> Shape<Vec<Cube>> {
            let rows = g.gen_range(0,64);
            let cols = g.gen_range(0,64);
            let data = match g.gen_range(0,4) {
                0 => rect_xz_even(rows, cols).data.collect::<Vec<_>>(),
                1 => rect_xz_even(rows, cols).data.collect::<Vec<_>>(),
                2 => rect_zx_odd(rows, cols).data.collect::<Vec<_>>(),
                _ => rect_zx_even(rows, cols).data.collect::<Vec<_>>(),
            };
            let total = data.len();
            Shape { data, total }
        }
    }
}

