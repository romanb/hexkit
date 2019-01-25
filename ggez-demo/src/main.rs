
mod assets;
mod entity;

use crate::assets::*;
use crate::entity::*;

use std::collections::{ HashMap, VecDeque };
use std::thread;
use std::time;
use std::path::Path;

use ggez::{ self, GameResult, GameError, Context, ContextBuilder };
use ggez::audio;
use ggez::conf::Conf;
use ggez::event::{ self, EventHandler };
use ggez::filesystem;
use ggez::graphics::{ self, Rect, Color, DrawParam, DrawMode };
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
    // UI state
    view: gridview::State<Offset<OddCol>>,
    scroll_border: scroll::Border,
    hover: Option<Offset<OddCol>>,
    selected: Option<Selected>,
    info: Option<Info>,
    turn_info: graphics::Text, // TODO: Just 'turn' in UIState
    menu: Menu,

    /// The next command to execute, if any.
    command: Option<Command>,
    /// Whether the update step of the game loop produced any changes
    /// that need rendering in the draw step.
    updated: bool,

    // Core game state
    turn: usize,
    shipyard: Shipyard,
    ships: HashMap<Offset<OddCol>, Ship>,
    costs: HashMap<Offset<OddCol>, usize>,
    /// There is at most one ongoing movement at a time.
    movement: Option<Movement>,

    // Assets
    images: Images,
    sounds: Sounds,

}

pub enum Menu {
    Main,
    Space,
    Shipyard,
}

struct Info {
    text: graphics::Text
}

impl Info {
    fn new(coords: Offset<OddCol>, entity: &Entity) -> Info {
        let info = format!("{} - {}", coords, entity.name());
        let text = graphics::Text::new(info);
        Info { text }
    }

    // fn draw(&self, &mut ctx: Context, dp: DrawParam) -> GameResult<()> {
    //     let width = self.text.width(ctx);
    //     let dest = Point2::new(width / 2. - info_width as f32, height - 50.);
    //     info.text.draw(ctx, DrawParam::default().dest(dest))?;
    // }
}

// struct Assets {
//     images: Images,
//     sounds: Sounds,
// }
// 
// struct UIState {
//     view: gridview::State<Offset<OddCol>>,
//     scroll_border: scroll::Border,
//     hover: Option<Offset<OddCol>>,
//     selected: Option<Selected>,
// }
// 
// struct GameState {
//     turn: usize,
//     shipyard: Shipyard,
//     ships: HashMap<Offset<OddCol>, Ship>, // fleet
//     costs: HashMap<Offset<OddCol>, usize>,
//     /// There is at most one ongoing movement at a time.
//     movement: Option<Movement>,
// }

impl State {
    fn entity(&self, coords: Offset<OddCol>) -> Entity {
        if self.shipyard.coords == coords {
            Entity::Shipyard(&self.shipyard)
        }
        else if let Some(ship) = self.ships.get(&coords) {
            Entity::Ship(ship)
        } else {
            Entity::Space
        }
    }

    /// Apply a command to the game state, updating it appropriately.
    /// Execution of a command optionally yields another command to
    /// execute, e.g. to repeat an operation.
    fn apply(&mut self, ctx: &mut Context, cmd: Command) -> GameResult<Option<Command>> {
        use Command::*;
        match cmd {
            ResizeView(width, height) => {
                self.view.resize(width as u32 - 302, height as u32 - 202);
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
                let mut next = None;
                if let Some(c) = coords {
                    let entity = self.entity(c);
                    match entity {
                        Entity::Space => {
                            self.info = Some(Info::new(c, &Entity::Space));
                            if let Some(ref s) = self.selected {
                                if let Some(ref r) = s.range {
                                    if let Some(path) = r.tree.path(c) {
                                        next = self.apply(ctx, PlanMove(path))?;
                                    } else {
                                        next = self.apply(ctx, PlanMove(VecDeque::new()))?;
                                    }
                                }
                            }
                        },
                        Entity::Shipyard(yard) => {
                            self.info = Some(Info::new(c, &Entity::Shipyard(yard)));
                            next = self.apply(ctx, PlanMove(VecDeque::new()))?;
                        }
                        Entity::Ship(ship) => {
                            self.info = Some(Info::new(c, &Entity::Ship(ship)));
                            next = self.apply(ctx, PlanMove(VecDeque::new()))?;
                        }
                    }
                } else {
                    self.info = None;
                }
                Ok(next)
            }

            // TODO: Unify SelectHexagon + SelectShip
            // match self.entity(&coords) ?
            SelectHexagon(selected) => {
                self.sounds.select.play()?;
                self.selected = selected.map(|(coords, hexagon)| {
                    if coords == self.shipyard.coords {
                        self.menu = Menu::Shipyard
                    } else {
                        self.menu = Menu::Space
                    }
                    Selected { coords, hexagon, range: None }
                });
                if self.selected.is_none() {
                    self.menu = Menu::Main
                }
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
                    self.selected = Some(Selected {
                        coords,
                        hexagon,
                        range: Some(MovementRange { tree, path: None })
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
                let path = self.selected.take()
                    .and_then(|s| s.range
                    .and_then(|r| r.path
                )).unwrap_or(VecDeque::new());
                let path_vec = Vec::from(path);
                for from in path_vec.first() {
                    for to in path_vec.last().filter(|c| *c != from) {
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

            EndTurn() => {
                // TODO: Recharge shipyard capacity, ship ranges, etc.
                self.turn += 1;
                self.turn_info = graphics::Text::new(format!("Turn {}", self.turn));
                Ok(None)
            }
        }
    }
}

/// Commands are the result of handling user input
/// and checking it against the current game state.
enum Command {
    /// Scroll the grid view.
    ScrollView(scroll::Delta, bool),
    /// Resize the window contents.
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

    /// Increase the movement cost of the currently selected hexagon.
    IncreaseCost(Offset<OddCol>),
    /// Decrease the movement cost of the currently selected hexagon.
    DecreaseCost(Offset<OddCol>),

    EndTurn()
}

// enum Selected2 {
//     SelectedHexagon(Offset<OddCol>, Hexagon),
//     SelectedShip(Offset<OddCol>, Hexagon, search::Tree<Offset<OddCol>>),
// }
// enum Movement2 {
//     Planned(VecDeque<search::Node<Offset<OddCol>>>),
//     InProgress {
//         goal: Offset<OddCol>,
//         pixel_path: animation::PathIter,
//         pixel_pos: Point2<f32>,
//         ship: Ship,
//     },
// }

struct Selected {
    coords: Offset<OddCol>,
    hexagon: Hexagon,
    range: Option<MovementRange>,
}

struct MovementRange {
    tree: search::Tree<Offset<OddCol>>,
    path: Option<VecDeque<search::Node<Offset<OddCol>>>>,
}

struct Movement {
    goal: Offset<OddCol>,
    pixel_path: animation::PathIter,
    pixel_pos: Point2<f32>,
    ship: Ship,
}

struct MovementContext<'a> {
    range: u16,
    grid: &'a Grid<Offset<OddCol>>,
    costs: &'a HashMap<Offset<OddCol>, usize>,
}

impl<'a> search::Context<Offset<OddCol>> for MovementContext<'a> {
    fn max_cost(&self) -> usize {
        self.range as usize
    }
    fn cost(&mut self, _from: Offset<OddCol>, to: Offset<OddCol>) -> Option<usize> {
        self.grid.get(to).and_then(|_| self.costs.get(&to).map(|c| *c).or(Some(1)))
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, UPDATES_PER_SEC as u32) {
            // Process the command
            let view_updated = self.view.update(); // TODO: Remove
            if let Some(cmd) = self.command.take() {
                self.command = self.apply(ctx, cmd)?;
                self.updated = true;
            }
            // Progress movement(s)
            if let Some(ref mut movement) = self.movement {
                if let Some(pos) = movement.pixel_path.next() {
                    movement.pixel_pos = pos;
                }
                else if let Some(mv) = self.movement.take() {
                    self.ships.insert(mv.goal, mv.ship);
                }
                self.updated = true;
            }
            self.updated = self.updated || view_updated;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.updated {
            // If the game state did not change, do not unnecessarily
            // consume CPU time by redundant rendering, while still
            // being responsive to input without a noticable delay.
            thread::sleep(time::Duration::from_millis(10));
            return Ok(())
        }

        graphics::clear(ctx, BLACK);

        // The base grid
        let mesh = &mut MeshBuilder::new();
        let grid_dest = self.view.grid_position();
        let grid_dp = DrawParam::default().dest(grid_dest);
        let schema = self.view.grid().schema();
        for (coords, hex) in self.view.iter_viewport() {
            // Hexagon
            mesh.polygon(DrawMode::Line(1.), hex.corners(), GREY)?;
            // Coordinates label
            hexworld_ggez::text::queue_label(
                ctx, schema, &hex, coords.to_string(),
                VAlign::Bottom, WHITE, Scale::uniform(12.));
            // Costs label
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
                    let path = p.iter().skip(1).map(|n| n.coords);
                    mesh::hexagons(&self.view, mesh, path, DrawMode::Line(3.), BLUE)
                })?;
            } else {
                mesh.polygon(DrawMode::Line(3.), s.hexagon.corners(), RED)?;
            }
        };

        let grid = mesh.build(ctx)?;
        graphics::draw(ctx, &grid, grid_dp)?;
        graphics::draw_queued_text(ctx, grid_dp)?;

        // Shipyard
        for hex in self.view.grid().get(self.shipyard.coords) {
            let img = &self.images.shipyard;
            image::draw_into(ctx, &img, hex, schema, grid_dest)?;
        }

        // Ships
        for (pos, ship) in &self.ships {
            let img = ship.class.image(&self.images);
            for hex in self.view.grid().get(*pos) {
                image::draw_into(ctx, &img, hex, schema, grid_dest)?;
            }
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
        mesh.rectangle(DrawMode::Fill, Rect::new(0.,0.,200.,height), BLACK);
        mesh.rectangle(DrawMode::Fill, Rect::new(0.,0.,width,100.), BLACK);
        mesh.rectangle(DrawMode::Fill, Rect::new(0.,height - 100.,width ,100.), BLACK);
        mesh.rectangle(DrawMode::Fill, Rect::new(width - 100.,0.,100.,height), BLACK);
        let hud = mesh.build(ctx)?;
        graphics::draw(ctx, &hud, DrawParam::default())?;

        // Info box
        if let Some(info) = &self.info {
            let info_width = info.text.width(ctx);
            let dest = Point2::new(width / 2. - info_width as f32 / 2., height - 50.);
            info.text.draw(ctx, DrawParam::default().dest(dest))?;
        }

        // Turn tracker
        let turn_width = self.turn_info.width(ctx);
        let dest = Point2::new(width / 2. - turn_width as f32 / 2., 50.);
        self.turn_info.draw(ctx, DrawParam::default().dest(dest))?;

        // Menu
        let menu_text = match self.menu {
            Menu::Main => graphics::Text::new("<Main>"),
            Menu::Shipyard => graphics::Text::new("<Shipyard>"),
            Menu::Space => graphics::Text::new("<Space>"),
        };
        let menu_width = menu_text.width(ctx);
        let dest = Point2::new(100. - menu_width as f32 / 2., 100.);
        menu_text.draw(ctx, DrawParam::default().dest(dest))?;

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
            else if let Some(ref r) = self.selected.as_ref().and_then(|s| s.range.as_ref()) { // TODO
                // ... with an active movement plan.
                if r.path.as_ref().filter(|p| p.back().map(|n| n.coords) == Some(coords)).is_some() { // TODO
                    // Selected the target hexagon of the current movement plan,
                    // thus execute the planned move.
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

            // End turn
            KeyCode::Return => Some(Command::EndTurn()),

            // Unknown
            _ => None
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _: f32, _: f32) {
        let scroll = self.scroll_border.eval(x, y);
        self.command = if scroll.dx != 0.0 || scroll.dy != 0.0 {
            Some(Command::ScrollView(scroll, true))
        } else {
            let coords = self.view.from_pixel(Point2::new(x,y)).map(|(c,_)| c);
            if coords != self.hover {
                Some(Command::HoverHexagon(coords))
            } else {
                None
            }
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

    // Setup the hexagonal grid
    let schema = Schema::new(SideLength(50.), Orientation::FlatTop);
    let grid = Grid::new(schema, shape::rectangle_xz_odd(30,30));
    let bounds = Bounds {
        position: Point2::new(201., 101.),
        width: width - 302.,
        height: height - 302.,
    };
    let view = gridview::State::new(grid, bounds);

    // A border region for scrolling the view
    let scroll_border = scroll::Border {
        bounds: Bounds { position: Point2::origin(), width, height },
        scale: 1.0,
        width: 25.0,
    };

    // Load assets
    let (ctx, evl) = &mut ContextBuilder::new("ggez-demo", "nobody").conf(cfg).build()?;
    filesystem::mount(ctx, Path::new("ggez-demo/assets"), true);
    let images = assets::Images::load(ctx)?;
    let sounds = assets::Sounds::load(ctx)?;

    // Intitial ship setup
    let mut shipyard = Shipyard::new(Offset::new(0,0), 1);
    let mut ships = HashMap::new();
    ships.insert(Offset::new(3,0), shipyard.new_ship(ShipClass::Carrier));
    ships.insert(Offset::new(1,1), shipyard.new_ship(ShipClass::Scout));
    ships.insert(Offset::new(0,3), shipyard.new_ship(ShipClass::Fighter));
    ships.insert(Offset::new(2,2), shipyard.new_ship(ShipClass::Battleship));

    // Start background music
    let mut music = audio::Source::new(ctx, "/background-track.mp3")?;
    music.set_repeat(true);
    music.play()?;

    // Run the game
    let state = &mut State {
        turn: 1,
        turn_info: graphics::Text::new("Turn 1"),
        view,
        scroll_border,
        images,
        sounds,
        updated: false,
        costs: HashMap::new(),
        shipyard,
        ships,
        selected: None,
        hover: None,
        command: None,
        movement: None,
        info: None,
        menu: Menu::Main,
    };
    event::run(ctx, evl, state)
}

