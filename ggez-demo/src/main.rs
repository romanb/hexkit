
use std::thread;
use std::time;

use ggez::*;
use ggez::graphics::*;
use ggez::event::*;

use hexworld_ggez::*;

use hexworld::grid::*;
use hexworld::grid::shape;
use hexworld::grid::axial::*;
use hexworld::grid::offset::*;
use hexworld::grid::cube::vec::*;

struct State {
    view: GridView<Offset<OddCol>>,
    drawer: Drawer,
    image: Image,
    hover: Option<Offset<OddCol>>,
    updated: bool,
    font: Font,
    // obstacles: Vec<Cube>,
    // lines: Vec<(Cube, Cube)>,
    // ranges: Vec<(Cube, u16)>,
    // reachable_ranges: Vec<(Cube, u16)>,
    // rings: Vec<(Cube, FlatTopDirection, u16, Rotation)>
}

impl State {
}

// struct TileState {
//     obstacle: bool,
// }

struct Drawer {
    // hover: Option<Offset>,
}

impl TileDrawer<Offset<OddCol>> for Drawer {
    fn draw_tile(
        &mut self,
        ctx: &mut Context,
        coords: Offset<OddCol>,
        hex: &Hexagon,
        mb: &mut MeshBuilder
    ) -> GameResult<()> {
        mb.polygon(DrawMode::Line(1.), hex.corners());
        let label = TextCached::new(coords.to_string())?;
        let pos = hex.position(label.width(ctx) as f32, label.height(ctx) as f32);
        label.queue(ctx, pos, None);
        Ok(())
    }

    // fn finalise(&mut self, ctx: &mut Context, grid_pos: Point2<f32>)
}

const RED: Color = Color { r: 1., g: 0., b: 0., a: 0.7 };
const BLUE: Color = Color { r: 0., g: 0., b: 1., a: 0.7 };
const GREEN: Color = Color { r: 0., g: 1., b: 0., a: 0.7 };
const GREY: Color = Color { r: 0.5, g: 0.5, b: 0.5, a: 0.7 };

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while ggez::timer::check_update_time(ctx, 60) {
            let view_updated = self.view.update(ctx)?;
            self.updated = self.updated || view_updated;
        }
        Ok(())
    }

    // nb. A new MeshBuilder is used for every section that uses
    // different colors for the polygons, because the DrawParams
    // can currently not be changed for individual items in a mesh.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.updated {
            thread::sleep(time::Duration::from_millis(10));
            return Ok(())
        }

        graphics::clear(ctx);

        set_color(ctx, WHITE)?;
        self.view.draw(ctx, &mut self.drawer)?;
        TextCached::draw_queued(ctx, DrawParam {
            dest: self.view.grid_position(), .. DrawParam::default()
        })?;

        let mut mesh: MeshBuilder;

        // Lines
        // set_color(ctx, RED)?;
        // mesh = MeshBuilder::new();
        // let start: Offset<OddCol> = Offset::new(0,0);
        // let end = Offset::new(10,4);
        // let hex_start = self.view.grid().get(start).unwrap();
        // let hex_end = self.view.grid().get(end).unwrap();
        // mesh.line(&[hex_start.center(), hex_end.center()], 2.);
        // let start_cube: Cube = start.into();
        // let end_cube: Cube = end.into();
        // let line_hexes = start_cube.beeline(end_cube);
        // self.view.draw_hexagons(ctx, &mut mesh, line_hexes, DrawMode::Line(2.))?;

        // // Ranges
        set_color(ctx, BLUE)?;
        mesh = MeshBuilder::new();
        let r1_center: Cube = Offset::<OddCol>::new(20,20).into();
        let r2_center: Cube = Offset::<OddCol>::new(17,15).into();
        self.view.draw_hexagons(ctx, &mut mesh, r1_center.range(3), DrawMode::Fill)?;
        self.view.draw_hexagons(ctx, &mut mesh, r2_center.range(3), DrawMode::Fill)?;

        // Overlapping ranges
        set_color(ctx, GREEN)?;
        mesh = MeshBuilder::new();
        let r12_overlap = r1_center.range_overlapping(r2_center, 3);
        self.view.draw_hexagons(ctx, &mut mesh, r12_overlap, DrawMode::Line(3.))?;

        // // Reachable ranges
        // set_color(ctx, BLACK)?;
        // mesh = MeshBuilder::new();
        // let obstacle1: Cube = Offset::<OddCol>::new(7,7).into();
        // let obstacle2: Cube = Offset::<OddCol>::new(9,9).into();
        // self.view.draw_hexagons(ctx, &mut mesh, [obstacle1, obstacle2].iter().cloned(), DrawMode::Fill)?;

        // set_color(ctx, GREEN)?;
        // mesh = MeshBuilder::new();
        // let obs_start: Cube = Offset::<OddCol>::new(8,9).into();
        // let reachable = obs_start.range_reachable(3, |x| x != obstacle1 && x != obstacle2);
        // self.view.draw_hexagons(ctx, &mut mesh, reachable.into_iter(), DrawMode::Fill)?;

        // // Rings
        // set_color(ctx, GREY)?;
        // mesh = MeshBuilder::new();
        // let ring_center: Cube = Offset::<OddCol>::new(10,4).into();
        // let ring = ring_center.walk_ring(FlatTopDirection::NorthEast, 4, Rotation::CW).collect::<Vec<_>>();
        // self.view.draw_hexagons(ctx, &mut mesh, ring.into_iter(), DrawMode::Fill)?;

        // "HUD"
        set_color(ctx, BLACK)?;
        let win_size = get_size(ctx);
        let (win_width, win_height) = (win_size.0 as f32, win_size.1 as f32);
        graphics::rectangle(ctx, DrawMode::Fill, Rect::new(0.,0.,100.,win_height))?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect::new(0.,0.,win_width,100.))?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect::new(0.,win_height - 100.,win_width ,100.))?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect::new(win_width - 100.,0.,100.,win_height))?;

        set_color(ctx, WHITE)?;
        self.image.draw(ctx, Point2::origin(), 0.0)?;

        graphics::present(ctx);
        self.updated = false;
        timer::yield_now();

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, code: Keycode, _mod: Mod, repeat: bool) {
        use self::Keycode::*;
        let delta = (10 * if repeat { 2 } else { 1 }) as f32;
        match code {
            Right => self.view.scroll_x(delta),
            Left  => self.view.scroll_x(-delta),
            Down  => self.view.scroll_y(delta),
            Up    => self.view.scroll_y(-delta),
            _     => {}
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, _state: MouseState, x: i32, y: i32, _xrel: i32, _yrel: i32) {
        // self.update.hover = self.view.from_pixel(x, y);
        self.hover = self.view.from_pixel(x, y).map(|(c,_h)| c);
        println!("{:?}", self.hover);
        let bounds = match get_size(ctx) {
            (w,h) => Bounds {
                position: Point2::origin(),
                width: w as f32,
                height: h as f32
            }
        };
        self.view.scroll_border(x as f32, y as f32, &bounds, 25., 1.0)
    }

    fn resize_event(&mut self, ctx: &mut Context, width: u32, height: u32) {
        let screen = Rect::new(0., 0., width as f32, height as f32);
        set_screen_coordinates(ctx, screen).unwrap();
        self.view.resize(width - 200, height - 200);
    }
}

fn main() -> Result<(), GameError> {
    let mut cfg = conf::Conf::new();
    // cfg.window_mode.width = 1200;
    cfg.window_setup.resizable = true;
    cfg.window_setup.title = "Hexworld".to_string();

    let width = cfg.window_mode.width;
    let height = cfg.window_mode.height;
    let ctx = &mut Context::load_from_conf("ggez-demo", "nobody", cfg)?;

    // ggez::mouse::set_grabbed(ctx, true);

    let schema = Schema::new(SideLength(50.), Orientation::FlatTop);
    // let grid = Grid::new(schema, shape::rect_xz_odd(30,30));
    // let grid = Grid::new(schema, shape::rectangle_xz_even(30,30));
    let grid = Grid::new(schema, shape::hexagon(5));
    let bounds = Bounds {
        position: Point2::new(100., 100.),
        width: (width - 200) as f32,
        height: (height - 200) as f32,
    };
    let view = GridView::new(grid, bounds);

    ctx.filesystem.mount(std::path::Path::new("/home/roman/dev/hexworld-rs/ggez-demo/assets"), true);
    ctx.filesystem.print_all();

    let image = Image::new(ctx, "/shadedDark04.png")?;
    let font = Font::default_font()?;
    let drawer = Drawer {};
    let state = &mut State {
        view,
        image,
        drawer,
        font,
        updated: true,
        hover: None,
    };

    event::run(ctx, state)
}

