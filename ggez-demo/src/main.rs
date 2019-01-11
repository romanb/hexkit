
use std::thread;
use std::time;

use ggez::*;
use ggez::graphics::*;
use ggez::event::*;

use hexworld_ggez::mesh;

use hexworld::grid::*;
use hexworld::grid::shape;
// use hexworld::grid::axial::*;
use hexworld::grid::offset::*;
use hexworld::grid::cube::vec::*;
use hexworld::ui::gridview;
use hexworld::search;

use nalgebra::Point2;

use std::collections::VecDeque;
use std::collections::HashMap;

struct State {
    view: gridview::State<Offset<OddCol>>,
    image: Image,
    hover: Option<Offset<OddCol>>,
    updated: bool,
    font: Font,
    path: Option<VecDeque<search::Node<Offset<OddCol>>>>,
    // ranges: Vec<(Cube, u16)>,
    // reachable_ranges: Vec<(Cube, u16)>,
    // rings: Vec<(Cube, FlatTopDirection, u16, Rotation)>
    costs: HashMap<Offset<OddCol>, Option<usize>>,
}

impl State {
}

impl search::Context<Offset<OddCol>> for State {
    // fn max_cost(&mut self) -> usize {
    //     10
    // }
    fn cost(&mut self, _from: Offset<OddCol>, to: Offset<OddCol>) -> Option<usize> {
        self.view.grid().get(to).and_then(|_|
            *self.costs.get(&to).unwrap_or(&Some(1)))
    }
}

struct Update {
    hover: Option<Offset<OddCol>>,
    resize: Option<(u32,u32)>,
}

const RED: Color = Color { r: 1., g: 0., b: 0., a: 0.7 };
const BLUE: Color = Color { r: 0., g: 0., b: 1., a: 0.7 };
const GREEN: Color = Color { r: 0., g: 1., b: 0., a: 0.7 };
const GREY: Color = Color { r: 0.5, g: 0.5, b: 0.5, a: 0.7 };

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while ggez::timer::check_update_time(ctx, 60) {
            let view_updated = self.view.update();
            self.updated = self.updated || view_updated;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.updated {
            thread::sleep(time::Duration::from_millis(10));
            return Ok(())
        }

        graphics::clear(ctx, WHITE);

        let mesh = &mut MeshBuilder::new();

        let dest = DrawParam::default().dest(self.view.grid_position());
        for (coords, hex) in self.view.iter_viewport() {
            mesh.polygon(DrawMode::Line(1.), hex.corners(), BLACK)?;
            let label = Text::new(coords.to_string());
            let pos = hex.position(label.width(ctx) as f32, label.height(ctx) as f32);
            graphics::queue_text(ctx, &label, pos, Some(BLACK));
        }

        // Ranges
        let r1_center: Cube = Offset::<OddCol>::new(20,20).into();
        let r2_center: Cube = Offset::<OddCol>::new(17,15).into();
        mesh::hexagons(&self.view, mesh, r1_center.range(3), DrawMode::Fill, BLUE)?;
        mesh::hexagons(&self.view, mesh, r2_center.range(3), DrawMode::Fill, BLUE)?;

        // Overlapping ranges
        let r12_overlap = r1_center.range_overlapping(r2_center, 3);
        mesh::hexagons(&self.view, mesh, r12_overlap, DrawMode::Line(3.), GREEN)?;

        // Obstacles
        let obstacle1: Cube = Offset::<OddCol>::new(7,7).into();
        let obstacle2: Cube = Offset::<OddCol>::new(9,9).into();
        mesh::hexagons(&self.view, mesh, [obstacle1, obstacle2].iter().cloned(), DrawMode::Fill, RED)?;

        // Reachable & visible ranges
        let obs_start: Cube = Offset::<OddCol>::new(8,9).into();
        let visible = obs_start.range_visible(3, |x| x != obstacle1 && x != obstacle2);
        mesh::hexagons(&self.view, mesh, visible.into_iter(), DrawMode::Fill, GREEN)?;

        // Rings
        let ring_center: Cube = Offset::<OddCol>::new(10,4).into();
        let ring = ring_center.walk_ring(FlatTopDirection::NorthEast, 4, Rotation::CW).collect::<Vec<_>>();
        mesh::hexagons(&self.view, mesh, ring.into_iter(), DrawMode::Fill, GREY)?;

        // Draw searh path
        self.path.as_ref().map_or(Ok(()), |p| {
            let path = p.iter().map(|n| n.coords);
            mesh::hexagons(&self.view, mesh, path, DrawMode::Line(5.), RED)
        })?;

        // Draw grid
        let grid = mesh.build(ctx)?;
        graphics::draw(ctx, &grid, dest)?;
        graphics::draw_queued_text(ctx, dest)?;

        // Draw "HUD"
        let mesh = &mut MeshBuilder::new();
        let win_size = graphics::drawable_size(ctx);
        let (win_width, win_height) = (win_size.0 as f32, win_size.1 as f32);
        mesh.rectangle(DrawMode::Fill, Rect::new(0.,0.,100.,win_height), BLACK);
        mesh.rectangle(DrawMode::Fill, Rect::new(0.,0.,win_width,100.), BLACK);
        mesh.rectangle(DrawMode::Fill, Rect::new(0.,win_height - 100.,win_width ,100.), BLACK);
        mesh.rectangle(DrawMode::Fill, Rect::new(win_width - 100.,0.,100.,win_height), BLACK);
        let hud = mesh.build(ctx)?;
        graphics::draw(ctx, &hud, DrawParam::default())?;
        self.image.draw(ctx, DrawParam::default())?;

        graphics::present(ctx)?;
        self.updated = false;
        timer::yield_now();

        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _btn: MouseButton, x: f32, y: f32) {
        if let Some(coords) = self.view.from_pixel(Point2::new(x,y)).map(|(c,_h)| c) {
            self.path = search::astar::path(Offset::new(0,0), coords, self);
            self.updated = true;
        }
    }

    fn key_down_event(&mut self, _ctx: &mut Context, code: KeyCode, _mod: KeyMods, repeat: bool) {
        use self::KeyCode::*;
        let delta = (10 * if repeat { 2 } else { 1 }) as f32;
        match code {
            Right => self.view.scroll_x(delta),
            Left  => self.view.scroll_x(-delta),
            Down  => self.view.scroll_y(delta),
            Up    => self.view.scroll_y(-delta),
            _     => {}
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, _xrel: f32, _yrel: f32) {
        // self.update.hover = self.view.from_pixel(x, y);
        self.hover = self.view.from_pixel(Point2::new(x,y)).map(|(c,_h)| c);
        println!("{:?}", self.hover);
        let bounds = match graphics::drawable_size(ctx) {
            (w,h) => Bounds {
                position: Point2::origin(),
                width: w as f32,
                height: h as f32
            }
        };
        self.view.scroll_border(x, y, &bounds, 25., 1.0)
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        let screen = Rect::new(0., 0., width, height);
        set_screen_coordinates(ctx, screen).unwrap();
        self.view.resize(width as u32 - 200, height as u32 - 200);
    }
}

fn main() -> Result<(), GameError> {
    let mut cfg = conf::Conf::new();
    // cfg.window_mode.width = 1200;
    cfg.window_mode.resizable = true;
    cfg.window_setup.title = "Hexworld".to_string();

    let width = cfg.window_mode.width;
    let height = cfg.window_mode.height;

    // ggez::mouse::set_grabbed(ctx, true);

    let schema = Schema::new(SideLength(50.), Orientation::FlatTop);
    let grid = Grid::new(schema, shape::rectangle_xz_odd(30,30));
    // let grid = Grid::new(schema, shape::rectangle_xz_even(30,30));
    // let grid = Grid::new(schema, shape::hexagon(5));
    let bounds = Bounds {
        position: Point2::new(100., 100.),
        width: width - 200.,
        height: height - 200.,
    };
    // let mut blocked = std::collections::HashSet::new();
    // blocked.insert(Offset::new(4,2));
    let view = gridview::State::new(grid, bounds);

    let (ctx, evl) = &mut ContextBuilder::new("ggez-demo", "nobody").conf(cfg).build()?;
    filesystem::mount(ctx, std::path::Path::new("/home/roman/dev/hexworld-rs/ggez-demo/assets"), true);
    filesystem::print_all(ctx);

    let image = Image::new(ctx, "/shadedDark04.png")?;
    let font = Font::default();
    // let drawer = Drawer {};
    let state = &mut State {
        view,
        image,
        // drawer,
        font,
        updated: true,
        hover: None,
        path: None,
        costs: HashMap::new(),
    };

    event::run(ctx, evl, state)
}

