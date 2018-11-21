
use nalgebra::geometry::Point2;
use std::collections::HashMap;
use std::u16;
use super::*;

/// Staggering for even or odd columns or rows, i.e. whether even or odd
/// columns or rows are offset from the top or left border of the grid,
/// respectively.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Stagger {
    Even, Odd
}

/// A rectangular grid that expands from the origin at the top-left
/// corner rightwards and downwards, using offset coordinates.
#[derive(Clone, Debug)]
pub struct OffsetGrid {
    cols: u16,
    rows: u16,
    schema: Schema,
    stagger: Stagger,
    hexagons: HashMap<Offset, Hexagon>
}

impl Grid<Offset> for OffsetGrid {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn tiles(&self) -> &HashMap<Offset, Hexagon> {
        &self.hexagons
    }
}

impl OffsetGrid {

    /// A rectangular grid that expands from the origin at the top-left corner
    /// in rows and columns, using offset coordinates.
    pub fn new(cols: u16, rows: u16, schema: Schema, stagger: Stagger) -> OffsetGrid {
        let hexagons = match schema.orientation {
            Orientation::FlatTop   => Self::mk_flat(cols, rows, &schema, stagger),
            Orientation::PointyTop => Self::mk_pointy(cols, rows, &schema, stagger)
        };
        OffsetGrid { cols, rows, schema, stagger, hexagons }
    }

    fn mk_flat(cols: u16, rows: u16, schema: &Schema, stagger: Stagger)
            -> HashMap<Offset, Hexagon> {
        let num = cols as usize * rows as usize;
        let mut hexagons = HashMap::with_capacity(num);
        for col in 0..cols {
            let ystag = match stagger {
                Stagger::Odd  => if col & 1 == 1 { schema.height / 2. } else { 0. }
                Stagger::Even => if col & 1 == 0 { schema.height / 2. } else { 0. }
            };
            for row in 0..rows {
                let center = Point2::new(
                    schema.size + col as f32 * schema.center_xoffset,
                    schema.height / 2. + row as f32 * schema.center_yoffset + ystag);
                let hex = schema.hexagon(center);
                let off = Offset { col, row };
                hexagons.insert(off, hex);
            }
        }
        hexagons
    }

    fn mk_pointy(cols: u16, rows: u16, schema: &Schema, stagger: Stagger) -> HashMap<Offset, Hexagon> {
        let num = cols as usize * rows as usize;
        let mut hexagons = HashMap::with_capacity(num);
        for row in 0..rows {
            let xstag = match stagger {
                Stagger::Odd  => if row & 1 == 1 { schema.width / 2. } else { 0. }
                Stagger::Even => if row & 1 == 0 { schema.width / 2. } else { 0. }
            };
            for col in 0..cols {
                let center = Point2::new(
                    schema.width / 2. + col as f32 * schema.center_xoffset + xstag,
                    schema.size + row as f32 * schema.center_yoffset);
                let hex = schema.hexagon(center);
                let off = Offset { col, row };
                hexagons.insert(off, hex);
            }
        }
        hexagons
    }
}

/// Offset coordinates.
///
/// Guide: [Offset Coordinates]
///
/// [Offset Coordinates]: https://www.redblobgames.com/grids/hexagons/#coordinates-offset
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Offset {
    pub col: u16, // TODO: i16, i.e. support negative offsets?
    pub row: u16
}

impl Offset {
    const MIN: i32 = 0;
    const MAX: i32 = u16::MAX as i32;

    pub fn new(col: u16, row: u16) -> Offset {
        Offset { col, row }
    }
}

impl Coords for Offset {
    type Grid = OffsetGrid;

    fn to_cube(self, grid: &OffsetGrid) -> Cube {
        let (col, row) = (self.col as i32, self.row as i32);
        match grid.schema.orientation {
            Orientation::FlatTop => match grid.stagger {
                Stagger::Odd => {
                    let z = row - ((col - (col & 1)) / 2);
                    Cube::new_xz(col, z)
                }
                Stagger::Even => {
                    let z = row - (col + (col & 1)) / 2;
                    Cube::new_xz(col, z)
                }
            }
            Orientation::PointyTop => match grid.stagger {
                Stagger::Odd => {
                    let x = col - (row - (row & 1)) / 2;
                    Cube::new_xz(x, row)
                }
                Stagger::Even => {
                    let x = col - (row + (row & 1)) / 2;
                    Cube::new_xz(x, row)
                }
            }
        }
    }

    fn from_cube(cube: Cube, grid: &OffsetGrid) -> Option<Offset> {
        // 0, 1, -1
        let (x, z) = (cube.x(), cube.z());
        let (col, row) = match grid.schema.orientation {
            Orientation::FlatTop => match grid.stagger {
                Stagger::Odd => {
                    let col = x;
                    let row = z + (x - (x & 1)) / 2;
                    (col, row)
                }
                Stagger::Even => {
                    let col = x;
                    let row = z + (x + (x & 1)) / 2;
                    (col, row)
                }
            }
            Orientation::PointyTop => match grid.stagger {
                Stagger::Odd => {
                    let col = x + (z - (z & 1)) / 2;
                    let row = z;
                    (col, row)
                }
                Stagger::Even => {
                    let col = x + (z + (z & 1)) / 2;
                    let row = z;
                    (col, row)
                }
            }
        };
        if Self::MIN <= col && col <= Self::MAX &&
                Self::MIN <= row && row <= Self::MAX {
            let o = Offset {
                col: col as u16,
                row: row as u16
            };
            if o.col <= grid.cols - 1 && o.row <= grid.rows {
                Some(o)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use geo::*;
    use super::*;
    use quickcheck::*;

    impl Arbitrary for Stagger {
        fn arbitrary<G: Gen>(g: &mut G) -> Stagger {
            if g.gen() {
                Stagger::Even
            } else {
                Stagger::Odd
            }
        }
    }

    impl Arbitrary for OffsetGrid {
        fn arbitrary<G: Gen>(g: &mut G) -> OffsetGrid {
            let cols = g.gen_range(0, 100);
            let rows = g.gen_range(0, 100);
            let orientation = Orientation::arbitrary(g);
            let stagger = Stagger::arbitrary(g);
            let schema = Schema::new(50., orientation);
            OffsetGrid::new(cols, rows, schema, stagger)
        }
    }

    #[test]
    fn prop_from_to_cube_identity() {
        fn prop(g: OffsetGrid) -> bool {
            g.tiles().keys().into_iter().all(|o| {
                Offset::from_cube(o.to_cube(&g), &g) == Some(*o)
            })
        }
        quickcheck(prop as fn(OffsetGrid) -> bool);
    }
}

