
use std::fmt;
use super::*;

/// Axial coordinates.
///
/// Guide: [Axial Coordinates]
///
/// [Axial Coordinates]: https://www.redblobgames.com/grids/hexagons/#coordinates-axial
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Axial {
    pub col: i32,
    pub row: i32
}

impl Coords for Axial {}

impl fmt::Display for Axial {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.col, self.row)
    }
}

impl From<Cube> for Axial {
    fn from(c: Cube) -> Axial {
        Axial { col: c.x(), row: c.z() }
    }
}

impl From<Axial> for Cube {
    fn from(a: Axial) -> Cube {
        Cube::new_xz(a.col, a.row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::*;

    #[test]
    fn prop_from_to_cube_identity() {
        fn prop(g: Grid<Axial>) -> bool {
            g.iter().all(|(&a,_)| {
                let c: Cube = a.into();
                Axial::from(c) == a
            })
        }
        quickcheck(prop as fn(_)  -> _);
    }
}

