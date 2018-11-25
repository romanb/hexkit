
extern crate hexworld;
extern crate ggez;

use ggez::*;
use ggez::graphics::*;
use ggez::event::*;

use hexworld::grid::*;
use hexworld::grid::offset::*;
use hexworld::grid::cube::vec::*;

struct State {
    grid: OffsetGrid,
    screen_movex: i32,
    screen_movey: i32,
    screen_move_repeat: bool,
    screen_resize_width: f32,
    screen_resize_height: f32,
    // font: Font,
}

impl State {
    fn render_hex(&self, mesh: &mut MeshBuilder, coord: Cube, mode: DrawMode) {
        if let Some(offset) = Offset::from_cube(coord, &self.grid) {
            if let Some(hex) = self.grid.tiles().get(&offset) {
                mesh.polygon(mode, hex.corners());
            }
        }
    }

    fn render_hexes<I: Iterator<Item=Cube>>(&self, mesh: &mut MeshBuilder, it: I, mode: DrawMode) {
        for coord in it {
            self.render_hex(mesh, coord, mode)
        }
    }
}

const RED: Color = Color { r: 1., g: 0., b: 0., a: 0.7 };
const BLUE: Color = Color { r: 0., g: 0., b: 1., a: 0.7 };
const GREEN: Color = Color { r: 0., g: 1., b: 0., a: 0.7 };
const GREY: Color = Color { r: 0.5, g: 0.5, b: 0.5, a: 0.7 };

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    // nb. A new MeshBuilder is used for every section that uses
    // different colors for the polygons, because the DrawParams
    // can currently not be changed for individual items in a mesh.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        // Update viewport
        let mut screen = get_screen_coordinates(ctx);
        if self.screen_movex != 0 || self.screen_movey != 0 {
            screen.x = f32::max(0., screen.x + self.screen_movex as f32);
            screen.y = f32::max(0., screen.y + self.screen_movey as f32);
            if !self.screen_move_repeat {
                self.screen_movex = 0;
                self.screen_movey = 0;
            }
        }
        if self.screen_resize_width != 0. || self.screen_resize_height != 0. {
            screen.w = self.screen_resize_width;
            screen.h = self.screen_resize_height;
            self.screen_resize_width = 0.;
            self.screen_resize_height = 0.;
        }
        set_screen_coordinates(ctx, screen)?;
        let viewport = Viewport {
            x: screen.x,
            y: screen.y,
            width: screen.w,
            height: screen.h,
        };

        // Render the grid with coordinates
        set_color(ctx, WHITE)?;
        let mut mesh = MeshBuilder::new();
        for (coords, hex) in self.grid.visible_tiles(&viewport) {
            mesh.polygon(DrawMode::Line(1.), hex.corners());
            let label = TextCached::new(format!("({},{})", coords.col, coords.row))?;
            let pos = hex.gauge(label.width(ctx) as f32, label.height(ctx) as f32);
            label.queue(ctx, pos, None);
        }
        // Render queued text fragments
        let mut param = DrawParam::default();
        param.dest = Point2::new(-screen.x, -screen.y);
        TextCached::draw_queued(ctx, param)?;
        // Render grid as a mesh
        let grid = mesh.build(ctx)?;
        graphics::draw(ctx, &grid, Point2::new(0.,0.), 0.0)?;

        // Lines
        set_color(ctx, RED)?;
        mesh = MeshBuilder::new();
        let start = Offset::new(0,0);
        let end = Offset::new(10,4);
        let hex_start = self.grid.tiles().get(&start).unwrap();
        let hex_end = self.grid.tiles().get(&end).unwrap();
        graphics::line(ctx, &[hex_start.center(), hex_end.center()], 2.)?;
        let start_cube = start.to_cube(&self.grid);
        let end_cube = end.to_cube(&self.grid);
        let line_hexes = start_cube.beeline(end_cube);
        self.render_hexes(&mut mesh, line_hexes, DrawMode::Line(2.));
        let lines = mesh.build(ctx)?;
        graphics::draw(ctx, &lines, Point2::new(0.,0.), 0.0)?;

        // Ranges
        set_color(ctx, BLUE)?;
        mesh = MeshBuilder::new();
        let r1_center = Offset::new(20,20).to_cube(&self.grid);
        let r2_center = Offset::new(17,15).to_cube(&self.grid);
        self.render_hexes(&mut mesh, r1_center.range(3), DrawMode::Fill);
        self.render_hexes(&mut mesh, r2_center.range(3), DrawMode::Fill);
        let ranges = mesh.build(ctx)?;
        graphics::draw(ctx, &ranges, Point2::new(0.,0.), 0.0)?;

        // Overlapping ranges
        set_color(ctx, GREEN)?;
        mesh = MeshBuilder::new();
        let r12_overlap = r1_center.range_overlapping(r2_center, 3);
        self.render_hexes(&mut mesh, r12_overlap, DrawMode::Line(3.));
        let ranges_overlapping = mesh.build(ctx)?;
        graphics::draw(ctx, &ranges_overlapping, Point2::new(0.,0.), 0.0)?;

        // Reachable ranges
        set_color(ctx, BLACK)?;
        mesh = MeshBuilder::new();
        let obstacle1 = Offset::new(7,7);
        let obstacle2 = Offset::new(9,9);
        let obs1_hex = self.grid.tiles().get(&obstacle1).unwrap();
        let obs2_hex = self.grid.tiles().get(&obstacle2).unwrap();
        mesh.polygon(DrawMode::Fill, obs1_hex.corners());
        mesh.polygon(DrawMode::Fill, obs2_hex.corners());
        let obstacles = mesh.build(ctx)?;
        graphics::draw(ctx, &obstacles, Point2::new(0.,0.), 0.0)?;

        set_color(ctx, GREEN)?;
        mesh = MeshBuilder::new();
        let obs1_cube = obstacle1.to_cube(&self.grid);
        let obs2_cube = obstacle2.to_cube(&self.grid);
        let obs_start = Offset::new(8,9).to_cube(&self.grid);
        let reachable = obs_start.range_reachable(3, |x| x != obs1_cube && x != obs2_cube);
        self.render_hexes(&mut mesh, reachable.into_iter(), DrawMode::Fill);
        let reachable_ranges = mesh.build(ctx)?;
        graphics::draw(ctx, &reachable_ranges, Point2::new(0.,0.), 0.0)?;

        // Rings
        set_color(ctx, GREY)?;
        mesh = MeshBuilder::new();
        let ring_center = Offset::new(10,4).to_cube(&self.grid);
        let ring = ring_center.walk_ring(FlatTopDirection::NorthEast, 4, Rotation::CW).collect::<Vec<_>>();
        self.render_hexes(&mut mesh, ring.into_iter(), DrawMode::Fill);
        let ring_mesh = mesh.build(ctx)?;
        graphics::draw(ctx, &ring_mesh, Point2::new(0.,0.), 0.0)?;

        present(ctx);

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, code: Keycode, _mod: Mod, _repeat: bool) {
        use Keycode::*;
        match code {
            Right => self.screen_movex += scroll_step(_repeat),
            Left  => self.screen_movex -= scroll_step(_repeat),
            Up    => self.screen_movey -= scroll_step(_repeat),
            Down  => self.screen_movey += scroll_step(_repeat),
            _     => {}
        }
    }

    // fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: i32, _y: i32) {
    //     println!("MOUSE DOWN");
    // }

    // fn mouse_button_up_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: i32, _y: i32) {
    //     println!("MOUSE UP");
    // }

    fn mouse_motion_event(&mut self, ctx: &mut Context, _state: MouseState, x: i32, y: i32, _xrel: i32, _yrel: i32) {
        let (w,h) = match get_size(ctx) {
            (w,h) => (w as i32, h as i32)
        };
        let scroll_border = 30;
        if x <= scroll_border {
            self.screen_movex = x - scroll_border - 1;
            self.screen_move_repeat = true;
        }
        else if y <= scroll_border {
            self.screen_movey = y - scroll_border - 1;
            self.screen_move_repeat = true;
        }
        else if x >= w - scroll_border {
            self.screen_movex = scroll_border + 1 - w + x;
            self.screen_move_repeat = true;
        }
        else if y >= h - scroll_border {
            self.screen_movey = scroll_border + 1 - h + y;
            self.screen_move_repeat = true;
        }
        else {
            self.screen_move_repeat = false;
        }
    }

    fn resize_event(&mut self, ctx: &mut Context, width: u32, height: u32) {
        self.screen_resize_width = width as f32;
        self.screen_resize_height = height as f32;
    }
}

fn scroll_step(repeat: bool) -> i32 {
    10 * if repeat { 2 } else { 1 }
}

fn main() {
    let mut cfg = conf::Conf::new();
    cfg.window_mode.vsync = true;
    cfg.window_mode.width = 1360;
    cfg.window_mode.height = 768;
    // cfg.window_mode.fullscreen_type = conf::FullscreenType::True;
    cfg.window_setup.resizable = true;
    let ctx = &mut Context::load_from_conf("ggez-demo", "nobody", cfg).unwrap();
    // let font = Font::default_font().unwrap();

    ggez::mouse::set_grabbed(ctx, true);

    let schema = Schema::new(50., Orientation::FlatTop);
    let grid = OffsetGrid::new(100, 100, schema, Stagger::Odd);

    println!("{:?}", Offset::from_cube(Cube::new_xy(13,-12), &grid));

    let state = &mut State {
        grid,
        // font,
        screen_movex: 0,
        screen_movey: 0,
        screen_move_repeat: false,
        screen_resize_width: 0.,
        screen_resize_height: 0.,
    };

    event::run(ctx, state).unwrap();
}

