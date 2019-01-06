
use hexworld::grid::{ Cube, Coords };
use hexworld::ui::gridview;

use ggez::*;
use ggez::graphics::*;

pub mod mesh {
    use super::*;

    pub fn hexagons<C: Coords>(
        view: &gridview::State<C>,
        mesh: &mut MeshBuilder,
        it: impl Iterator<Item=Cube>,
        mode: DrawMode,
        color: Color,
    ) -> GameResult<()> {
        for cc in it {
            if let Some(c) = Some(C::from(cc)) {
                if let Some(hex) = view.grid().get(c) {
                    let hex_bounds = view.grid().schema().bounds(hex);
                    if view.viewport().intersects(&hex_bounds) {
                        mesh.polygon(mode, hex.corners(), color)?;
                    }
                }
            }
        }
        Ok(())
    }

}

