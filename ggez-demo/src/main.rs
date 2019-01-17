
use std::collections::{ HashMap, VecDeque };
use std::thread;
use std::time;
use std::path::Path;

use ggez::{ self, GameResult, GameError, Context, ContextBuilder };
use ggez::conf::Conf;
use ggez::event::{ self, EventHandler };
use ggez::filesystem;
use ggez::graphics::{ self, Image, Rect, Color, DrawParam, DrawMode };
use ggez::graphics::{ Drawable, Scale, MeshBuilder, BLACK, WHITE };
use ggez::input::keyboard::{ KeyCode, KeyMods };
use ggez::input::mouse::MouseButton;
use ggez::timer;

use hexworld::geo::{ Schema, SideLength, Bounds, VAlign, Orientation, Hexagon };
use hexworld::grid::Grid;
use hexworld::grid::shape;
use hexworld::grid::offset::{ Offset, OddCol };
use hexworld::ui::gridview;
use hexworld::search;

use hexworld_ggez::mesh;
use hexworld_ggez::animation;

use nalgebra::{ Point2, Vector2 };

const UPDATES_PER_SECOND: u16 = 60;
const MOVE_SECS_PER_HEX:  f32 = 0.15;

const RED:  Color = Color { r: 1.,  g: 0.,  b: 0.,  a: 0.7 };
const BLUE: Color = Color { r: 0.,  g: 0.,  b: 1.,  a: 1.  };
const GREY: Color = Color { r: 0.5, g: 0.5, b: 0.5, a: 0.7 };

struct State {
    // ships: HashMap<Offset<OddCol>, Ship>,
    view: gridview::State<Offset<OddCol>>,
    resize: Option<(f32,f32)>,
    costs: HashMap<Offset<OddCol>, usize>,
    hover: Option<Offset<OddCol>>,
    selected: Option<Selected>, // selection?
    updated: bool,

    // TODO
    image: Image,
    position: Option<Offset<OddCol>>,
    path_anim: Option<animation::PathIter>,
    path_anim_pos: Option<Point2<f32>>,
    max_cost: usize,
}

impl State {
    // TODO
}

struct Selected {
    coords: Offset<OddCol>,
    hexagon: Hexagon,
    range: Option<MovementRange>,
}

struct MovementRange {
    tree: search::Tree<Offset<OddCol>>,
    path: Option<VecDeque<search::Node<Offset<OddCol>>>>,
}

struct Ship {
    image: Image,
    range: u16,
    movement: Option<Movement>,
}

struct Movement {
    path: animation::PathIter,
    position: Point2<f32>,
}

impl Ship {
    fn new(image: Image, range: u16) -> Ship {
        Ship {
            image, range, movement: None
        }
    }
}

// TODO: Context of a selected ship, not the entire game state.
impl search::Context<Offset<OddCol>> for State {
    fn max_cost(&self) -> usize {
        self.max_cost // self.ships.get(self.selected.coords).range
    }
    fn cost(&mut self, _from: Offset<OddCol>, to: Offset<OddCol>) -> Option<usize> {
        self.view.grid().get(to).and_then(|_|
            self.costs.get(&to).map(|c| *c).or(Some(1)))
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, UPDATES_PER_SECOND as u32) {
            let view_updated = self.view.update();
            self.updated = self.updated || view_updated;
            if let Some(ref mut iter) = self.path_anim {
                self.path_anim_pos = iter.next().or_else(|| {
                    self.position = self.path_anim_pos.and_then(|p| self.view.grid().from_pixel(p)).map(|(o,_h)| o);
                    self.path_anim = None;
                    None
                });
                self.updated = true;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.updated {
            thread::sleep(time::Duration::from_millis(10));
            return Ok(())
        }

        if let Some((width, height)) = self.resize {
            let screen = graphics::Rect::new(0., 0., width, height);
            graphics::set_screen_coordinates(ctx, screen)?;
            graphics::present(ctx)?;
            self.resize = None;
        }

        graphics::clear(ctx, WHITE);

        // Grid
        let mesh = &mut MeshBuilder::new();
        let grid_dest = self.view.grid_position();
        let grid_dp = DrawParam::default().dest(grid_dest);
        let schema = self.view.grid().schema();
        for (coords, hex) in self.view.iter_viewport() {
            mesh.polygon(DrawMode::Line(1.), hex.corners(), BLACK)?;

            hexworld_ggez::text::queue_label(
                ctx, schema, &hex, coords.to_string(),
                VAlign::Bottom, BLACK, Scale::uniform(12.));

            let cost = *self.costs.get(coords).unwrap_or(&1);
            hexworld_ggez::text::queue_label(
                ctx, schema, &hex, cost.to_string(),
                VAlign::Middle, BLUE, Scale::uniform(graphics::DEFAULT_FONT_SCALE));

            if Some(*coords) == self.position {
                hexworld_ggez::image::draw_into(ctx, &self.image, &hex, schema, grid_dest)?;
            }
        }
        if let Some(pos) = self.path_anim_pos {
            let img = &self.image;
            let vec = Vector2::new(img.width() as f32 / 2., img.height() as f32 / 2.);
            let img_dest = grid_dest + pos.coords - vec;
            img.draw(ctx, DrawParam::default().dest(img_dest))?;
        }
        if let Some(ref s) = self.selected {
            if let Some(ref r) = s.range {
                let coords = r.tree.iter().map(|(&c,_)| c).filter(|c| Some(*c) != self.position);
                mesh::hexagons(&self.view, mesh, coords, DrawMode::Fill, GREY)?;
                r.path.as_ref().map_or(Ok(()), |p| {
                    let path = p.iter().map(|n| n.coords);
                    mesh::hexagons(&self.view, mesh, path, DrawMode::Line(3.), BLUE)
                })?;
            } else {
                mesh.polygon(DrawMode::Line(3.), s.hexagon.corners(), RED)?;
            }
        };
        let grid = mesh.build(ctx)?;
        graphics::draw(ctx, &grid, grid_dp)?;
        graphics::draw_queued_text(ctx, grid_dp)?;

        // "HUD" frame
        let mesh = &mut MeshBuilder::new();
        let size = graphics::drawable_size(ctx);
        let (width, height) = (size.0 as f32, size.1 as f32);
        mesh.rectangle(DrawMode::Fill, Rect::new(0.,0.,100.,height), BLACK);
        mesh.rectangle(DrawMode::Fill, Rect::new(0.,0.,width,100.), BLACK);
        mesh.rectangle(DrawMode::Fill, Rect::new(0.,height - 100.,width ,100.), BLACK);
        mesh.rectangle(DrawMode::Fill, Rect::new(width - 100.,0.,100.,height), BLACK);
        let hud = mesh.build(ctx)?;
        graphics::draw(ctx, &hud, DrawParam::default())?;

        graphics::present(ctx)?;
        self.updated = false;
        timer::yield_now();

        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _btn: MouseButton, x: f32, y: f32) {
        let selected = self.view.from_pixel(Point2::new(x,y)).map(|(c,h)| (c,h.clone()));
        // self.selected = self.view.from_pixel(Point2::new(x,y)).map(|(c,_h)| c);
        if let Some((coords, hexagon)) = selected {
            // Clicked on a valid hexagon ...
            if let Some(ref r) = self.selected.as_ref().and_then(|s| s.range.as_ref()) {
                // ... with an active movement range,
                if let Some(path) = r.tree.path(coords) {
                    // There is a path within the range to the hexagon,
                    // thus move the ship along that path.
                    let path_vec = Vec::from(path);
                    if path_vec.len() > 1 {
                        let anim = animation::path(
                            UPDATES_PER_SECOND,
                            MOVE_SECS_PER_HEX,
                            self.view.grid(),
                            &path_vec);
                        self.path_anim = Some(anim);
                        self.position = None;
                    }
                }
                // In any case, the ship is deselected.
                self.selected = None;
            }
            else if Some(coords) == self.position {
                // ... that is the current ship position, thus we want to
                // show the movement range. Using A* without a goal disables
                // the heuristic, yielding Dijkstra's algorithm.
                let tree = search::astar::tree(coords, None, self);
                let path = tree.path(coords);
                self.selected = Some(Selected {
                    coords,
                    hexagon,
                    range: Some(MovementRange { tree, path })
                });
            }
            else {
                // ... that is not occupied and not in an active movement range.
                self.selected = Some(Selected {
                    coords, hexagon, range: None
                });
            }
        } else {
            // Clicked on something that is not a hexagon on the map.
            self.selected = None;
        }
        self.updated = true;
    }

    fn key_down_event(&mut self, _ctx: &mut Context, code: KeyCode, _mod: KeyMods, repeat: bool) {
        let delta = (10 * if repeat { 2 } else { 1 }) as f32;
        match code {
            // Scrolling
            KeyCode::Right => self.view.scroll_x(delta),
            KeyCode::Left  => self.view.scroll_x(-delta),
            KeyCode::Down  => self.view.scroll_y(delta),
            KeyCode::Up    => self.view.scroll_y(-delta),
            // Costs
            KeyCode::I => for s in &self.selected {
                // TODO: if there is a ship on the selected coordinates,
                // increase its range.
                let v = self.costs.entry(s.coords).or_insert(1);
                if *v < 100 {
                    *v += 1;
                    self.updated = true;
                }
            },
            KeyCode::D => for s in &self.selected {
                // TODO: if there is a ship on the selected coordinates,
                // decrease its range.
                let v = self.costs.entry(s.coords).or_insert(1);
                if *v > 0 {
                    *v -= 1;
                    self.updated = true;
                }
            },
            _ => {}
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, _xrel: f32, _yrel: f32) {
        self.hover = self.view.from_pixel(Point2::new(x,y)).map(|(c,_h)| c);
        if let Some(hover) = self.hover {
            if let Some(ref mut s) = self.selected {
                if let Some(ref mut r) = s.range {
                    r.path = r.tree.path(hover)
                }
            }
        }
        self.updated = self.hover.is_some() || self.selected.is_some();
        let bounds = match graphics::drawable_size(ctx) {
            (w,h) => Bounds {
                position: Point2::origin(),
                width: w as f32,
                height: h as f32
            }
        };
        self.view.scroll_border(x, y, &bounds, 25., 1.0)
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) {
        self.view.resize(width as u32 - 200, height as u32 - 200);
        self.resize = Some((width, height));
    }
}

fn main() -> Result<(), GameError> {
    let mut cfg = Conf::new();
    cfg.window_mode.resizable = true;
    cfg.window_setup.title = "Hexworld".to_string();

    let width = cfg.window_mode.width;
    let height = cfg.window_mode.height;

    // mouse::set_grabbed(ctx, true);

    let schema = Schema::new(SideLength(50.), Orientation::FlatTop);
    let grid = Grid::new(schema, shape::rectangle_xz_odd(30,30));
    let bounds = Bounds {
        position: Point2::new(100., 100.),
        width: width - 200.,
        height: height - 200.,
    };
    let view = gridview::State::new(grid, bounds);

    let (ctx, evl) = &mut ContextBuilder::new("ggez-demo", "nobody").conf(cfg).build()?;
    filesystem::mount(ctx, Path::new("ggez-demo/assets"), true);

    // TODO: Create initial ship
    let image = Image::new(ctx, "/ship3-small.png")?;

    let state = &mut State {
        view,
        image,
        updated: false,
        resize: None,
        position: Some(Offset::new(0,0)),
        costs: HashMap::new(),
        max_cost: 10,
        selected: None,
        // range: None,
        hover: None,
        // path: None,
        path_anim: None,
        path_anim_pos: None,
    };

    event::run(ctx, evl, state)
}

