
mod assets;
mod entity;
mod movement;
mod menu;

use crate::assets::*;
use crate::entity::*;
use crate::movement::*;
use crate::menu::*;

use std::borrow::Cow;
use std::collections::HashMap;
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
use hexworld_ggez::image;

use nalgebra::{ Point2, Vector2 };

const RED:  Color = Color { r: 1.,  g: 0.,  b: 0.,  a: 0.7 };
const BLUE: Color = Color { r: 0.,  g: 0.,  b: 1.,  a: 1.  };
const GREY: Color = Color { r: 0.5, g: 0.5, b: 0.5, a: 0.7 };

// type Offset = Offset<OddCol>;

struct State {
    // ui: ui::State,
    // UI state
    view: gridview::State<Offset<OddCol>>,
    scroll_border: scroll::Border,
    hover: Option<Offset<OddCol>>,
    selected: Option<Selected>,
    info: Option<Info>,
    turn_info: graphics::Text, // TODO: Just 'turn' in UIState
    panel: ControlPanel,

    /// The next command to execute, if any.
    command: Option<Command>,
    /// Whether the update step of the game loop produced any changes
    /// that need rendering in the draw step.
    updated: bool,

    // Core game state (model / world) (world::State ?)
    turn: usize,
    entities: HashMap<Offset<OddCol>, Entity>,
    costs: HashMap<Offset<OddCol>, usize>,
    /// There is at most one ongoing movement at a time.
    movement: Option<Movement>,

    // Assets
    assets: Assets,
}

struct ControlPanel {
    info: Option<graphics::Text>,
    menu: Menu<Button>,
}

impl ControlPanel {
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if let Some(info) = &self.info {
            let info_w = info.width(ctx) as f32;
            let dest = Point2::new((200. - info_w) / 2., 100.);
            info.draw(ctx, DrawParam::default().dest(dest))?;
        }
        self.menu.draw(ctx)
    }

    fn main(_ctx: &mut Context) -> ControlPanel {
        let mut menu = Menu::new(Point2::new(25., 100.), 150., 30.);
        menu.add(Button::ToggleGrid, "Toggle Grid");
        menu.add(Button::ToggleCoords, "Toggle Coordinates");
        menu.add(Button::EndTurn, "End Turn");
        ControlPanel { info: None, menu }
    }

    fn hexagon(ctx: &mut Context, coords: Offset<OddCol>, entity: Option<&Entity>) -> ControlPanel {
        // Info
        let title = entity.map_or(Cow::Borrowed("Empty Space"), |e| e.name());
        let mut text = graphics::Text::new(format!("{} - {}", coords, title));
        match entity {
            None => {}
            Some(Entity::Ship(ship)) => {
                text.add(format!("\nRange: {}", ship.range));
            }
            Some(Entity::Shipyard(yard)) => {
                text.add(format!("\nCapacity: {}", yard.capacity));
            }
        }
        text.set_bounds(Point2::new(150., 100.), graphics::Align::Center);
        let text_h = text.height(ctx) as f32;
        let info = Some(text);
        // Menu
        let menu_y = 100. + text_h + 25.;
        let mut menu = Menu::new(Point2::new(25., menu_y), 150., 30.);
        match entity {
            None | Some(Entity::Ship(_)) => {
                menu.add(Button::IncreaseCost, "Increase Cost");
                menu.add(Button::DecreaseCost, "Decrease Cost");
            }
            Some(Entity::Shipyard(_)) => {
                menu.add(Button::NewFighter,
                    &format!("Fighter ({})",
                        ShipClass::Fighter.spec().shipyard_capacity));
                menu.add(Button::NewScout,
                    &format!("Scout ({})",
                        ShipClass::Scout.spec().shipyard_capacity));
                menu.add(Button::NewBattleship,
                    &format!("Battleship ({})",
                        ShipClass::Battleship.spec().shipyard_capacity));
                menu.add(Button::NewCarrier,
                    &format!("Carrier ({})",
                        ShipClass::Carrier.spec().shipyard_capacity));
            }
        }
        ControlPanel { info, menu }
    }

    // fn shipyard() -> ControlPanel {
    //     let mut menu = Menu::new(Point2::new(25.,100.), 150., 30.);
    //     let text = format!("{} - {}", coords, entity.name());
    //     let info = Some(graphics::Text::new(text));
    //     ControlPanel { info, menu }
    // }
}

#[derive(Copy, Clone, Debug)]
enum Button {
    NewFighter,
    NewCarrier,
    NewBattleship,
    NewScout,
    IncreaseCost,
    DecreaseCost,
    ToggleGrid,
    ToggleCoords,
    EndTurn,
}

struct Settings {
}

// pub struct Menu(Menu<Button>);

// struct Hud {
//     turn: graphics::Text,
//     info: Option<Info>,
//     /// The currently active menu.
//     menu: Menu,
//     menu_text: graphics::Text,
// }

// impl Hud {
//     pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
//         let size = graphics::drawable_size(ctx);
//         let (width, height) = (size.0 as f32, size.1 as f32);
//         let mut mesh = MeshBuilder::new();
//         mesh.rectangle(DrawMode::Fill, Rect::new(0.,0.,200.,height), BLACK);
//         mesh.rectangle(DrawMode::Fill, Rect::new(0.,0.,width,100.), BLACK);
//         mesh.rectangle(DrawMode::Fill, Rect::new(0.,height - 100.,width ,100.), BLACK);
//         mesh.rectangle(DrawMode::Fill, Rect::new(width - 100.,0.,100.,height), BLACK);
//         let hud = mesh.build(ctx)?;
//         graphics::draw(ctx, &hud, DrawParam::default())?;
// 
//         // Info box (part of HUD)
//         if let Some(info) = &self.info {
//             let info_width = info.text.width(ctx);
//             let dest = Point2::new(width / 2. - info_width as f32 / 2., height - 50.);
//             info.text.draw(ctx, DrawParam::default().dest(dest))?;
//         }
// 
//         // Turn tracker (part of HUD)
//         let turn_width = self.turn.width(ctx);
//         let dest = Point2::new(width / 2. - turn_width as f32 / 2., 50.);
//         self.turn.draw(ctx, DrawParam::default().dest(dest))?;
// 
//         Menu (part of HUD)
//         let menu_text = match self.menu {
//             Menu::Main => graphics::Text::new("<Main>"),
//             Menu::Shipyard => graphics::Text::new("<Shipyard>"),
//             Menu::Space => graphics::Text::new("<Space>"),
//         };
//         let menu_width = menu_text.width(ctx);
//         let dest = Point2::new(100. - menu_width as f32 / 2., 100.);
//         menu_text.draw(ctx, DrawParam::default().dest(dest))
//     }
// }

// struct UIState {
//     hud: Hud,
//     view: gridview::State<Offset<OddCol>>,
//     scroll_border: scroll::Border,
//     hover: Option<Offset<OddCol>>,
//     selected: Option<Selected>,
// }

// pub enum Menu {
//     Main,
//     Space,
//     Shipyard,
// }

/// Information about a hexagon.
struct Info {
    text: graphics::Text
}

impl Info {
    fn new(coords: Offset<OddCol>, entity: Option<&Entity>) -> Info {
        let name = entity.map_or(Cow::Borrowed("Empty Space"), |e| e.name());
        let info = format!("{} - {}", coords, name);
        let text = graphics::Text::new(info);
        Info { text }
    }

    // fn draw(&self, &mut ctx: Context, dp: DrawParam) -> GameResult<()> {
    //     let width = self.text.width(ctx);
    //     let dest = Point2::new(width / 2. - info_width as f32, height - 50.);
    //     info.text.draw(ctx, DrawParam::default().dest(dest))?;
    // }
}

// struct GameState {
//     turn: usize,
//     shipyard: Shipyard,
//     ships: HashMap<Offset<OddCol>, Ship>, // fleet
//     costs: HashMap<Offset<OddCol>, usize>,
//     /// There is at most one ongoing movement at a time.
//     movement: Option<Movement>,
// }

impl State {

    // Selected::new()?
    fn selected(&self, coords: Offset<OddCol>, hexagon: Hexagon, entity: Option<&Entity>) -> Selected {
        match entity {
            None => Selected { coords, hexagon, range: None },
            Some(entity) => {
                let mut mvc = MovementContext {
                    costs: &self.costs,
                    grid: self.view.grid(),
                    range: entity.range(),
                };
                let tree = search::astar::tree(coords, None, &mut mvc);
                Selected {
                    coords,
                    hexagon,
                    range: Some(MovementRange { tree, path: None })
                }
            }
        }
    }

    fn select(&self, coords: Offset<OddCol>) -> Option<Selected> {
        self.view.grid().get(coords).map(|h|
            self.selected(coords, h.clone(),
                self.entities.get(&coords)))
    }

    fn begin_move(&mut self) -> GameResult<()> {
        // Cut short / complete any previous movement.
        if let Some(mv) = self.movement.take() {
            self.entities.insert(mv.goal, mv.entity);
        }
        // Take the currently selected movement path.
        let path = self.selected.take()
            .and_then(|s| s.range
            .and_then(|r| r.path
        )).map_or(Vec::new(), Vec::from);
        // Setup the new movement.
        self.movement = path.first()
            .and_then(|from| self.entities.remove(&from.coords))
            .and_then(|entity| Movement::new(entity, &path, self.view.grid()));
        // Play movement sound.
        for mv in &self.movement {
            let sound = mv.entity.sound(&mut self.assets.sounds);
            sound.play()?;
            sound.set_volume(0.25);
        }
        Ok(())
    }

    fn end_move(&mut self, ctx: &mut Context, mv: Movement) {
        debug_assert!(mv.cost as u16 <= mv.entity.range());
        let goal = mv.goal;
        let mut entity = mv.entity;
        // Reduce remaining ship range.
        entity.reduce_range(mv.cost as u16);
        // If nothing else has been selected meanwhile, select the
        // ship again to continue movement.
        self.selected = self.selected.take().or_else(|| {
            self.panel = ControlPanel::hexagon(ctx, goal, Some(&entity));
            self.view.grid().get(goal).map(|h|
                self.selected(goal, h.clone(), Some(&entity)))
        });
        self.entities.insert(goal, entity);
    }

    fn end_turn(&mut self) {
        // Recharge shipyard capacity, ship ranges, etc.
        for entity in self.entities.values_mut() {
            match entity {
                Entity::Ship(ship) => {
                    let spec = ship.class.spec();
                    ship.range = spec.range;
                }
                Entity::Shipyard(yard) => {
                    yard.capacity += 1;
                }
            }
        }
        self.turn += 1;
        self.turn_info = graphics::Text::new(format!("Turn {}", self.turn));
    }

    /// If the shipyard is selected that has sufficient capacity and
    /// there is a free neighbouring hexagon, place a new ship.
    fn new_ship(&mut self, class: ShipClass) -> Option<(Offset<OddCol>, &Entity)> {
        if let Some(s) = &self.selected {
            if let Some(free) = hexworld::grid::Cube::from(s.coords)
                .neighbours()
                .find_map(|n|
                    Some(Offset::from(n)).filter(|o|
                        self.view.grid().get(*o).is_some() &&
                        !self.entities.contains_key(o))) {
                if let Some(e) = self.entities.get_mut(&s.coords) {
                    if let Entity::Shipyard(yard) = e {
                        if let Some(ship) = yard.new_ship(class) {
                            let entity = Entity::Ship(ship);
                            self.entities.insert(free, entity);
                            // return Some((s.coords, &entity))
                            return self.entities.get(&free).map(|e| (free,e))
                            //self.panel = ControlPanel::hexagon(ctx, s.coords, Some(&*e));
                        }
                    }
                }
            }
        }
        return None
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
                if let Some(c) = coords {
                    let entity = self.entities.get(&c);
                    self.info = Some(Info::new(c, entity));
                    if let Some(ref mut s) = self.selected {
                        if let Some(ref mut r) = s.range {
                            if entity.is_none() {
                                r.path = r.tree.path(c);
                            } else {
                                r.path = None;
                            }
                        }
                    }
                } else {
                    self.info = None;
                }
                Ok(None)
            }

            SelectHexagon(coords) => {
                if self.selected.as_ref()
                    .and_then(|s| s.range.as_ref())
                    .and_then(|r| r.path.as_ref())
                    .and_then(|p| p.back())
                    .map_or(false, |n| Some(n.coords) == coords)
                {
                    // Selected the target hexagon of the currently selected
                    // movement path, thus execute the move.
                    self.begin_move()?;
                } else {
                    self.selected = coords.and_then(|c| self.select(c));
                    self.panel = match coords {
                        Some(c) => ControlPanel::hexagon(ctx, c, self.entities.get(&c)),
                        None    => ControlPanel::main(ctx)
                    };
                }
                self.assets.sounds.select.play()?;
                // TODO: Update menu based on selection.
                Ok(None)
            }

            SelectButton(btn) => {
                match btn {
                    Button::IncreaseCost => for s in &self.selected {
                        let v = self.costs.entry(s.coords).or_insert(1);
                        *v = usize::min(100, *v + 1);
                    },
                    Button::DecreaseCost => for s in &self.selected {
                        let v = self.costs.entry(s.coords).or_insert(1);
                        *v = usize::max(1, *v - 1);
                    },
                    Button::NewFighter => {
                        if let Some((c,e)) = self.new_ship(ShipClass::Fighter) {
                            self.panel = ControlPanel::hexagon(ctx, c, Some(e));
                            self.selected = self.select(c);
                        }
                    },
                    Button::NewScout => {
                        if let Some((c,e)) = self.new_ship(ShipClass::Scout) {
                            self.panel = ControlPanel::hexagon(ctx, c, Some(e));
                            self.selected = self.select(c);
                        }
                    },
                    Button::NewCarrier => {
                        if let Some((c,e)) = self.new_ship(ShipClass::Carrier) {
                            self.panel = ControlPanel::hexagon(ctx, c, Some(e));
                            self.selected = self.select(c);
                        }
                    },
                    Button::NewBattleship => {
                        if let Some((c,e)) = self.new_ship(ShipClass::Battleship) {
                            self.panel = ControlPanel::hexagon(ctx, c, Some(e));
                            self.selected = self.select(c);
                        }
                    },
                    Button::ToggleGrid => {
                        //
                    },
                    Button::ToggleCoords => {
                        //
                    },
                    Button::EndTurn => {
                        self.end_turn()
                    }
                }
                Ok(None)
            }

            EndTurn() => {
                self.end_turn();
                Ok(None)
            }
        }
    }
}

/// The commands that drive the game (state).
enum Command { // Input?
    /// Scroll the grid view.
    ScrollView(scroll::Delta, bool),
    /// Resize the window contents.
    ResizeView(f32, f32),
    /// Hover over the specified grid coordinates, or a part of the grid
    /// that does not correspond to any valid coordinates.
    HoverHexagon(Option<Offset<OddCol>>),
    /// Select the specified grid coordinates, or a part of the grid
    /// that does not correspond to any valid coordinates.
    SelectHexagon(Option<Offset<OddCol>>),
    /// Select a button from the control panel.
    SelectButton(Button),
    /// End the current turn.
    EndTurn()
}

struct Selected {
    coords: Offset<OddCol>,
    hexagon: Hexagon,
    range: Option<MovementRange>,
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
                    // Movement is complete.
                    self.end_move(ctx, mv);
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
            thread::sleep(time::Duration::from_millis(5));
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

        // Selection
        if let Some(ref s) = self.selected {
            mesh.polygon(DrawMode::Line(3.), s.hexagon.corners(), RED)?;
            if let Some(ref r) = s.range {
                let coords = r.tree.iter().map(|(&c,_)| c).filter(|c| *c != s.coords);
                mesh::hexagons(&self.view, mesh, coords, DrawMode::Fill, GREY)?;
                r.path.as_ref().map_or(Ok(()), |p| {
                    let path = p.iter().skip(1).map(|n| n.coords);
                    mesh::hexagons(&self.view, mesh, path, DrawMode::Line(3.), BLUE)
                })?;
            }
        };

        let grid = mesh.build(ctx)?;
        graphics::draw(ctx, &grid, grid_dp)?;
        graphics::draw_queued_text(ctx, grid_dp)?;

        // Entities
        for (pos, entity) in &self.entities {
            let img = entity.image(&self.assets.images);
            for hex in self.view.grid().get(*pos) {
                image::draw_into(ctx, &img, hex, schema, grid_dest)?;
            }
        }

        // Movement
        if let Some(ref movement) = self.movement {
            let img = movement.entity.image(&self.assets.images);
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

        // Info box (part of HUD)
        if let Some(info) = &self.info {
            let info_width = info.text.width(ctx);
            let dest = Point2::new(width / 2. - info_width as f32 / 2., height - 50.);
            info.text.draw(ctx, DrawParam::default().dest(dest))?;
        }

        // Turn tracker (part of HUD)
        let turn_width = self.turn_info.width(ctx);
        let dest = Point2::new(width / 2. - turn_width as f32 / 2., 50.);
        self.turn_info.draw(ctx, DrawParam::default().dest(dest))?;

        // Menu (part of HUD)
        self.panel.draw(ctx)?;

        graphics::present(ctx)?;
        self.updated = false;
        timer::yield_now();

        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _btn: MouseButton, x: f32, y: f32) {
        let p = Point2::new(x, y);
        if let Some(&btn) = self.panel.menu.select(p) {
            self.command = Some(Command::SelectButton(btn))
        } else {
            let selected = self.view.from_pixel(p).map(|(c,_)| c);
            self.command = Some(Command::SelectHexagon(selected));
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
    // Setup the Context
    let mut cfg = Conf::new();
    cfg.window_mode.resizable = true;
    // cfg.window_mode.hidpi = false;
    cfg.window_setup.title = "Hexworld".to_string();
    let width = cfg.window_mode.width;
    let height = cfg.window_mode.height;
    let (ctx, game_loop) = &mut ContextBuilder
        ::new("ggez-demo", "nobody")
         .conf(cfg)
         .build()?;

    // println!("{:?}", graphics::os_hidpi_factor(ctx));
    // mouse::set_grabbed(ctx, true);

    // Setup the hexagonal grid
    let schema = Schema::new(SideLength(50.), Orientation::FlatTop);
    let grid = Grid::new(schema, shape::rectangle_xz_odd(30, 30));
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
    filesystem::mount(ctx, Path::new("ggez-demo/assets"), true);
    let assets = Assets::load(ctx)?;

    // Intitial ship setup
    let mut shipyard = Shipyard::new(1);
    let mut entities = HashMap::new();
    // for carrier in shipyard.new_ship(ShipClass::Carrier);
    // entities.insert(Offset::new(3,0), Entity::Ship(shipyard.new_ship(ShipClass::Carrier)));
    // entities.insert(Offset::new(1,1), Entity::Ship(shipyard.new_ship(ShipClass::Scout)));
    // entities.insert(Offset::new(0,3), Entity::Ship(shipyard.new_ship(ShipClass::Fighter)));
    // entities.insert(Offset::new(2,2), Entity::Ship(shipyard.new_ship(ShipClass::Battleship)));
    entities.insert(Offset::new(0,0), Entity::Shipyard(shipyard));

    // Start background music
    // TODO: Move into assets
    let mut music = audio::Source::new(ctx, "/background-track.mp3")?;
    music.set_repeat(true);
    music.play()?;

    // Run the game
    let state = &mut State {
        turn: 1,
        turn_info: graphics::Text::new("Turn 1"),
        view,
        scroll_border,
        assets,
        updated: false,
        costs: HashMap::new(),
        entities,
        selected: None,
        hover: None,
        command: None,
        movement: None,
        info: None,
        panel: ControlPanel::main(ctx),
    };
    event::run(ctx, game_loop, state)
}

