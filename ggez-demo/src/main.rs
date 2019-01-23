
use std::collections::{ HashMap, VecDeque };
use std::thread;
use std::time;
use std::path::Path;

use ggez::{ self, GameResult, GameError, Context, ContextBuilder };
use ggez::audio;
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
use hexworld::ui::scroll;
use hexworld::search;

use hexworld_ggez::mesh;
use hexworld_ggez::animation;
use hexworld_ggez::image;

use nalgebra::{ Point2, Vector2 };

const UPDATES_PER_SEC: u16 = 60;
const MOVE_HEX_SECS:   f32 = 0.15;

const RED:  Color = Color { r: 1.,  g: 0.,  b: 0.,  a: 0.7 };
const BLUE: Color = Color { r: 0.,  g: 0.,  b: 1.,  a: 1.  };
const GREY: Color = Color { r: 0.5, g: 0.5, b: 0.5, a: 0.7 };

// type Offset = Offset<OddCol>;

struct State {
    // UI elements
    view: gridview::State<Offset<OddCol>>,
    scroll_border: scroll::Border,

    /// The next command to execute, if any.
    command: Option<Command>,
    updated: bool,

    // Core game state
    ships: HashMap<Offset<OddCol>, Ship>,
    costs: HashMap<Offset<OddCol>, usize>,

    // Assets
    images: Images,
    sounds: Sounds,

    // UI state
    hover: Option<Offset<OddCol>>,
    selected: Option<Selected>,

    /// There is at most one ongoing movement at a time.
    movement: Option<Movement>,
}

impl State {
    /// Apply a command to the game state, updating it appropriately.
    /// Execution of a command optionally yields another command to
    /// execute, e.g. to repeat an operation.
    fn apply(&mut self, ctx: &mut Context, cmd: Command) -> GameResult<Option<Command>> {
        use Command::*;
        match cmd {
            ResizeView(width, height) => {
                self.view.resize(width as u32 - 200, height as u32 - 200);
                let screen = graphics::Rect::new(0., 0., width, height);
                graphics::set_screen_coordinates(ctx, screen)?;
                graphics::present(ctx)?;
                self.scroll_border = scroll::Border {
                    bounds: Bounds {
                        position: Point2::origin(),
                        width,
                        height
                    }, .. self.scroll_border
                };
                Ok(None)
            }

            ScrollView(delta, repeat) => {
                self.view.scroll_x(delta.dx);
                self.view.scroll_y(delta.dy);
                if repeat {
                    Ok(Some(ScrollView(delta, repeat)))
                } else {
                    Ok(None)
                }
            }

            HoverHexagon(coords) => {
                self.hover = coords;
                Ok(None)
            }

            SelectHexagon(selected) => {
                self.sounds.select.play()?;
                self.selected = selected.map(|(coords, hexagon)| {
                    Selected { coords, hexagon, range: None }
                });
                Ok(None)
            }

            SelectShip(coords, hexagon, _ship_id) => {
                self.sounds.select.play()?;
                if let Some(ship) = self.ships.get(&coords) {
                    let mut mv_ctx = MovementContext {
                        costs: &self.costs,
                        grid: self.view.grid(),
                        range: ship.range,
                    };
                    let tree = search::astar::tree(coords, None, &mut mv_ctx);
                    // let path = tree.path(coords); TODO
                    let path = None;
                    self.selected = Some(Selected {
                        coords,
                        hexagon,
                        range: Some(MovementRange { tree, path })
                    });
                }
                Ok(None)
            }

            PlanMove(path) => {
                if let Some(ref mut s) = self.selected {
                    if let Some(ref mut r) = s.range {
                        r.path = Some(path);
                    }
                }
                Ok(None)
            }

            Move() => {
                let path = self.selected.take().and_then(|s| s.range.and_then(|r| r.path)).unwrap_or(
                    VecDeque::new());
                let path_vec = Vec::from(path);
                for from in path_vec.first() {
                    for to in path_vec.last() {
                        if from.coords != to.coords {
                            if let Some(ship) = self.ships.remove(&from.coords) {
                                let anim = animation::path(UPDATES_PER_SEC, MOVE_HEX_SECS, self.view.grid(), &path_vec);
                                // TODO: Cut short any previous movement.
                                let sound = ship.class.sound(&mut self.sounds);
                                sound.play()?;
                                sound.set_volume(0.25);
                                self.movement = Some(Movement {
                                    goal: to.coords,
                                    pixel_path: anim,
                                    pixel_pos: Point2::origin(),
                                    ship,
                                });
                            }
                        }
                    }
                }
                Ok(None)
            }

            IncreaseCost(coords) => {
                let v = self.costs.entry(coords).or_insert(1);
                *v = usize::min(100, *v + 1);
                Ok(None)
            }

            DecreaseCost(coords) => {
                let v = self.costs.entry(coords).or_insert(1);
                *v = usize::max(1, *v - 1);
                Ok(None)
            }
        }
    }
}

/// Commands are the result of handling user input
/// and checking it against the current game state.
enum Command {
    /// The mouse is close to a border of the game window, thus requesting
    /// scrolling of the grid.
    ScrollView(scroll::Delta, bool),
    ResizeView(f32, f32),

    /// The cursor is hovering over some part of the grid, which may or may not
    /// be valid coordinates.
    HoverHexagon(Option<Offset<OddCol>>),
    SelectHexagon(Option<(Offset<OddCol>, Hexagon)>),
    SelectShip(Offset<OddCol>, Hexagon, ShipId),

    /// Plan a move for the currently selected ship.
    PlanMove(VecDeque<search::Node<Offset<OddCol>>>),
    /// Execute the planned move, if any.
    Move(),

    IncreaseCost(Offset<OddCol>),
    DecreaseCost(Offset<OddCol>),

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

struct Sounds {
    select: audio::Source,
    engine: audio::Source,
}

impl Sounds {
    fn load(ctx: &mut Context) -> GameResult<Sounds> {
        let select = audio::Source::new(ctx, "/select.wav")?;
        let engine = audio::Source::new(ctx, "/engine2.mp3")?;
        Ok(Sounds {
            select, engine
        })
    }
}

struct Images {
    scout: graphics::Image,
    fighter: graphics::Image,
    battleship: graphics::Image,
    mothership: graphics::Image,
}

impl Images {
    fn load(ctx: &mut Context) -> GameResult<Images> {
        let scout = Image::new(ctx, "/scout.png")?;
        let fighter = Image::new(ctx, "/fighter.png")?;
        let battleship = Image::new(ctx, "/battleship.png")?;
        let mothership = Image::new(ctx, "/mothership.png")?;
        Ok(Images {
            scout, fighter, battleship, mothership
        })
    }
}

type ShipId = u16;

enum ShipClass {
    Fighter, Scout, Battleship, Mothership
}

struct ShipSpec {
    range: u16,
    // attack: u16,
    // cost: u16,
}

impl ShipClass {
    /// Get the (technical) specifications of a ship class,
    /// describing its game-relevant attributes.
    fn spec(&self) -> ShipSpec {
        use ShipClass::*;
        match self {
            Fighter => ShipSpec {
                range: 2,
            },
            Scout => ShipSpec {
                range: 10,
            },
            Battleship => ShipSpec {
                range: 5,
            },
            Mothership => ShipSpec {
                range: 3,
            }
        }
    }

    /// Select an image for a ship class.
    fn image(&self, images: &Images) -> Image {
        use ShipClass::*;
        match self {
            Fighter => images.fighter.clone(),
            Scout => images.scout.clone(),
            Battleship => images.battleship.clone(),
            Mothership => images.mothership.clone()
        }
    }

    fn sound<'a>(&'a self, sounds: &'a mut Sounds) -> &'a mut audio::Source {
        &mut sounds.engine
        // use ShipClass::*;
        // match self {

        // }
    }
}

struct Ship {
    id: ShipId,
    class: ShipClass,
    range: u16,
}

struct Movement {
    // hex_path: Vec<search::Node<Offset<OddCol>>>,
    goal: Offset<OddCol>,
    pixel_path: animation::PathIter,
    pixel_pos: Point2<f32>,
    ship: Ship,
}

impl Ship {
    fn new(id: ShipId, class: ShipClass) -> Ship {
        let range = class.spec().range;
        Ship { id, class, range }
    }
}

struct MovementContext<'a> {
    range: u16,
    grid: &'a Grid<Offset<OddCol>>,
    costs: &'a HashMap<Offset<OddCol>, usize>,
}

impl<'a> search::Context<Offset<OddCol>> for MovementContext<'a> {
    fn max_cost(&self) -> usize {
        self.range as usize // self.ships.get(self.selected.coords).range
    }
    fn cost(&mut self, _from: Offset<OddCol>, to: Offset<OddCol>) -> Option<usize> {
        self.grid.get(to).and_then(|_|
            self.costs.get(&to).map(|c| *c).or(Some(1)))
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, UPDATES_PER_SEC as u32) {
            // Apply command
            let view_updated = self.view.update();
            if let Some(cmd) = self.command.take() {
                self.command = self.apply(ctx, cmd)?;
                self.updated = true;
            }
            // Progress movement
            if let Some(ref mut movement) = self.movement {
                if let Some(pos) = movement.pixel_path.next() {
                    movement.pixel_pos = pos;
                }
                else if let Some(mv) = self.movement.take() {
                    let goal = mv.goal; // .hex_path.last();
                    let ship = mv.ship;
                    // goal.map(|to| {
                        self.ships.insert(goal, ship);
                    //});
                }
                self.updated = true;
            }
            self.updated = self.updated || view_updated;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.updated {
            thread::sleep(time::Duration::from_millis(10));
            return Ok(())
        }

        graphics::clear(ctx, BLACK);

        // Base grid
        let mesh = &mut MeshBuilder::new();
        let grid_dest = self.view.grid_position();
        let grid_dp = DrawParam::default().dest(grid_dest);
        let schema = self.view.grid().schema();
        for (coords, hex) in self.view.iter_viewport() {
            mesh.polygon(DrawMode::Line(1.), hex.corners(), GREY)?;

            hexworld_ggez::text::queue_label(
                ctx, schema, &hex, coords.to_string(),
                VAlign::Bottom, WHITE, Scale::uniform(12.));

            let cost = *self.costs.get(coords).unwrap_or(&1);
            hexworld_ggez::text::queue_label(
                ctx, schema, &hex, cost.to_string(),
                VAlign::Middle, WHITE, Scale::uniform(graphics::DEFAULT_FONT_SCALE));
        }

        // Selection on grid
        if let Some(ref s) = self.selected {
            if let Some(ref r) = s.range {
                let coords = r.tree.iter().map(|(&c,_)| c).filter(|c| *c != s.coords);
                mesh::hexagons(&self.view, mesh, coords, DrawMode::Fill, GREY)?;
                r.path.as_ref().map_or(Ok(()), |p| {
                    let path = p.iter().skip(1).map(|n| n.coords); // .filter(|c| *c != s.coords);
                    mesh::hexagons(&self.view, mesh, path, DrawMode::Line(3.), BLUE)
                })?;
            } else {
                mesh.polygon(DrawMode::Line(3.), s.hexagon.corners(), RED)?;
            }
        };

        let grid = mesh.build(ctx)?;
        graphics::draw(ctx, &grid, grid_dp)?;
        graphics::draw_queued_text(ctx, grid_dp)?;

        // Ships
        for (pos, ship) in &self.ships {
            let img = ship.class.image(&self.images);
            let hex = self.view.grid().get(*pos).unwrap(); // TODO
            image::draw_into(ctx, &img, hex, schema, grid_dest)?;
        }

        // Movement
        if let Some(ref movement) = self.movement {
            let img = movement.ship.class.image(&self.images);
            let vec = Vector2::new(img.width() as f32 / 2., img.height() as f32 / 2.);
            let img_dest = grid_dest + movement.pixel_pos.coords - vec;
            img.draw(ctx, DrawParam::default().dest(img_dest))?;
        }

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
        if let Some((coords, hexagon)) = selected {
            // Clicked on a valid hexagon ...
            if let Some(ship) = self.ships.get(&coords) {
                // ... that is a ship position.
                self.command = Some(Command::SelectShip(coords, hexagon, ship.id));
            }
            else if let Some(ref r) = self.selected.as_ref().and_then(|s| s.range.as_ref()) {
                // ... with an active movement plan.
                if r.path.as_ref().filter(|p| p.back().map(|n| n.coords) == Some(coords)).is_some() {
                    self.command = Some(Command::Move());
                } else {
                    self.command = Some(Command::SelectHexagon(Some((coords, hexagon))));
                }
            } else {
                // ... that is not occupied.
                self.command = Some(Command::SelectHexagon(Some((coords, hexagon))));
            }
        } else {
            // Clicked on something that is not a hexagon on the map.
            self.command = Some(Command::SelectHexagon(None));
        }
    }

    fn key_down_event(&mut self, _ctx: &mut Context, code: KeyCode, _mod: KeyMods, repeat: bool) {
        let delta = (10 * if repeat { 2 } else { 1 }) as f32;
        self.command = match code {
            // Scrolling
            KeyCode::Right => Some(Command::ScrollView(scroll::Delta { dx: delta, dy: 0.0 }, false)),
            KeyCode::Left  => Some(Command::ScrollView(scroll::Delta { dx: -delta, dy: 0.0 }, false)),
            KeyCode::Down  => Some(Command::ScrollView(scroll::Delta { dx: 0.0, dy: delta }, false)),
            KeyCode::Up    => Some(Command::ScrollView(scroll::Delta { dx: 0.0, dy: -delta }, false)),

            // Costs & Ranges
            KeyCode::I => self.selected.as_ref().and_then(|s| {
                Some(Command::IncreaseCost(s.coords))
            }),
            KeyCode::D => self.selected.as_ref().and_then(|s| {
                Some(Command::DecreaseCost(s.coords))
            }),

            // Deselect
            KeyCode::Escape => Some(Command::SelectHexagon(None)),

            // Unknown
            _ => None
        }
    }

    fn mouse_motion_event(&mut self, _: &mut Context, x: f32, y: f32, _: f32, _: f32) {
        let scroll = self.scroll_border.eval(x, y);
        self.command = if scroll.dx != 0.0 || scroll.dy != 0.0 {
            Some(Command::ScrollView(scroll, true))
        } else {
            let coords = self.view.from_pixel(Point2::new(x,y)).map(|(c,_h)| c);
            coords.and_then(|c| {
                if !self.ships.contains_key(&c) {
                    self.selected.as_ref().and_then(|s| {
                        s.range.as_ref().and_then(|r| {
                            r.tree.path(c).map(|p| {
                                Command::PlanMove(p)
                            })
                        })
                    })
                } else {
                    // None
                    Some(Command::PlanMove(VecDeque::new()))
                }
            }).or_else(|| {
                Some(Command::HoverHexagon(coords))
            })
        }
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) {
        self.command = Some(Command::ResizeView(width, height));
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
    // let image = Image::new(ctx, "/ship4.png")?;
    let images = Images::load(ctx)?;
    let sounds = Sounds::load(ctx)?;

    let scroll_border = scroll::Border {
        bounds: Bounds {
            position: Point2::origin(),
            width,
            height
        },
        scale: 1.0,
        width: 25.0,
    };

    let mut ships = HashMap::new();
    ships.insert(Offset::new(0,0), Ship::new(1, ShipClass::Mothership));
    ships.insert(Offset::new(1,1), Ship::new(2, ShipClass::Scout));
    ships.insert(Offset::new(0,3), Ship::new(3, ShipClass::Fighter));
    ships.insert(Offset::new(2,2), Ship::new(4, ShipClass::Battleship));

    let state = &mut State {
        view,
        scroll_border,
        images,
        sounds,
        updated: false,
        costs: HashMap::new(),
        ships,
        selected: None,
        hover: None,
        command: None,
        movement: None,
    };

    let mut music = audio::Source::new(ctx, "/background-track.mp3")?;
    music.set_repeat(true);
    music.play()?;

    event::run(ctx, evl, state)
}

