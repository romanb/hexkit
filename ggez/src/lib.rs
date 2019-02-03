
pub mod menu;

use hexworld::grid::Coords;
use hexworld::ui::gridview;

use ggez::*;
use ggez::graphics::*;

pub mod mesh {
    use super::*;
    use std::borrow::Borrow;

    pub fn hexagons<C: Coords, T: Borrow<C>>(
        view: &gridview::State<C>,
        mesh: &mut MeshBuilder,
        it: impl Iterator<Item=T>,
        mode: DrawMode,
        color: Color,
    ) -> GameResult<()> {
        for t in it {
            if let Some(hex) = view.grid().get(*t.borrow()) {
                let hex_bounds = view.grid().schema().bounds(hex);
                if view.viewport().intersects(&hex_bounds) {
                    mesh.polygon(mode, hex.corners(), color)?;
                }
            }
        }
        Ok(())
    }

}

pub mod image {
    use super::*;
    use hexworld::geo::{ Hexagon, Schema, VAlign };
    use nalgebra::Point2;

    pub fn draw_into(
        ctx: &mut Context,
        img: &Image,
        hex: &Hexagon,
        schema: &Schema,
        origin: Point2<f32>
    ) -> GameResult<()> {
        let (img_w, img_h) = (img.width() as f32, img.height() as f32);
        let img_pos = schema.valign(&hex, img_w, img_h, VAlign::Middle);
        let img_dest = origin + img_pos.coords;
        img.draw(ctx, DrawParam::default().dest(img_dest))
    }
}

pub mod text {
    use super::*;
    use hexworld::geo::{ Hexagon, Schema, VAlign };

    /// Queue a hexagon label for rendering.
    pub fn queue_label(
        ctx: &mut Context,
        schema: &Schema,
        hex: &Hexagon,
        label: String,
        valign: VAlign,
        color: Color,
        scale: Scale
    ) {
        let txt = Text::new(TextFragment::new(label).scale(scale));
        let (w, h) = (txt.width(ctx) as f32, txt.height(ctx) as f32);
        let pos = schema.valign(hex, w, h, valign);
        graphics::queue_text(ctx, &txt, pos, Some(color));
    }

}

pub mod animation {
    use super::*;
    use std::borrow::Borrow;
    use nalgebra::Point2;
    use hexworld::grid::Grid;

    pub struct PathIter {
        edges: Vec<(Point2<f32>, Point2<f32>)>,
        edge_i: usize,
        step_dx: f32,
        step_dy: f32,
        step_i: usize,
        steps_per_hex: usize,
    }

    impl PathIter {
        fn new(edges: Vec<(Point2<f32>, Point2<f32>)>, steps_per_hex: usize) -> PathIter {
            let mut iter = PathIter {
                edges,
                steps_per_hex,
                edge_i: 0,
                step_i: 0,
                step_dx: 0.0,
                step_dy: 0.0,
            };
            iter.calc_dxy();
            iter
        }

        fn calc_dxy(&mut self) {
            let (center_a, center_b) = self.edges[self.edge_i];
            let dx = center_b.x - center_a.x;
            let dy = center_b.y - center_a.y;
            self.step_dx = dx / self.steps_per_hex as f32;
            self.step_dy = dy / self.steps_per_hex as f32;
        }
    }

    impl Iterator for PathIter {
        type Item = Point2<f32>;

        fn next(&mut self) -> Option<Self::Item> {
            let next_edge_i = self.edge_i + 1;
            let max_steps = self.steps_per_hex;
            if self.step_i == max_steps {
                if next_edge_i >= self.edges.len() {
                    None
                } else {
                    self.edge_i = next_edge_i;
                    self.step_i = 0;
                    self.calc_dxy();
                    self.next()
                }
            }
            else {
                let center_a = self.edges[self.edge_i].0;
                let i = self.step_i as f32;
                let next = Point2::new(center_a.x + i * self.step_dx,
                                       center_a.y + i * self.step_dy);
                self.step_i += 1;
                Some(next)
            }
        }
    }

    // search::Path::to_pixel ?
    pub fn path<C, T>(ups: u16, secs: f32, grid: &Grid<C>, path: &[T]) -> PathIter
    where C: Coords,
          T: Borrow<C>
    {
        let steps_per_hex = (ups as f32 * secs).round() as usize;
        let edges = path.windows(2).map(|win| {
            let (c1, c2) = (*win[0].borrow(), *win[1].borrow());
            (grid.to_pixel(c1), grid.to_pixel(c2))
        }).collect::<Vec<_>>();
        PathIter::new(edges, steps_per_hex)
    }

}

