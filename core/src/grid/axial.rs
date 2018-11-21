
use std::collections::HashMap;
use std::i16;
use super::*;

/// Axial coordinates.
///
/// Guide: [Axial Coordinates]
///
/// [Axial Coordinates]: https://www.redblobgames.com/grids/hexagons/#coordinates-axial
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Axial {
    pub col: i16,
    pub row: i16
}

impl Axial {
    const MIN: i32 = i16::MIN as i32;
    const MAX: i32 = i16::MAX as i32;

    pub fn promote(&self) -> Cube {
        Cube::new_xz(self.col as i32, self.row as i32)
    }

    pub fn demote(cube: &Cube) -> Axial {
        Axial {
            col: cube.x() as i16,
            row: cube.z() as i16
        }
    }
}

impl Coords for Axial {
    type Grid = AxialGrid;

    fn to_cube(self, _grid: &AxialGrid) -> Cube {
        Cube::new_xz(self.col as i32, self.row as i32)
    }

    fn from_cube(cube: Cube, _grid: &AxialGrid) -> Option<Axial> {
        let (x,z) = (cube.x(), cube.z());
        if Self::MIN <= x && x <= Self::MAX && Self::MIN <= z && z <= Self::MAX {
            Some(Axial { col: x as i16, row: z as i16 })
        } else {
            None
        }
    }
}

/// A grid that expands from the origin at the center
/// in concentric circles, using [`Axial`] coordinates.
///
/// [`Axial`]: struct.Axial.html
pub struct AxialGrid {
    schema: Schema,
    tiles: HashMap<Axial, Hexagon>,
}

impl AxialGrid {
    pub fn new(schema: Schema) -> AxialGrid {
        AxialGrid {
            schema,
            tiles: HashMap::new()
        }
    }
}

impl Grid<Axial> for AxialGrid {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn tiles(&self) -> &HashMap<Axial, Hexagon> {
        &self.tiles
    }
}

