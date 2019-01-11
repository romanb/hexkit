
use hexworld::grid::Coords;
use hexworld::ui::gridview;

use ggez::*;
use ggez::graphics::*;

pub mod mesh {
    use super::*;

    pub fn hexagons<C: Coords, CC: Coords>(
        view: &gridview::State<C>,
        mesh: &mut MeshBuilder,
        it: impl Iterator<Item=CC>,
        mode: DrawMode,
        color: Color,
    ) -> GameResult<()> {
        for cc in it {
            if let Some(hex) = view.grid().get(C::from(cc.into())) {
                let hex_bounds = view.grid().schema().bounds(hex);
                if view.viewport().intersects(&hex_bounds) {
                    mesh.polygon(mode, hex.corners(), color)?;
                }
            }
        }
        Ok(())
    }

}

