
mod assets;
mod entity;
mod movement;
mod menu;
mod ui;
mod world;

use crate::assets::*;
use crate::entity::*;
use crate::movement::*;

use std::thread;
use std::time;
use std::path::Path;

use ggez::{ self, GameResult, GameError, Context, ContextBuilder };
use ggez::conf::Conf;
use ggez::event::{ self, EventHandler };
use ggez::filesystem;
use ggez::graphics::{ self };
use ggez::graphics::{ BLACK };
use ggez::input::keyboard::{ KeyCode, KeyMods };
use ggez::input::mouse::MouseButton;
use ggez::timer;

use hexworld::geo::{ Bounds, Hexagon };
use hexworld::grid::offset::{ Offset, OddCol };
use hexworld::ui::scroll;
use hexworld::search;

use nalgebra::{ Point2 };

/// The complete game state.
struct State {
    ui: ui::State,
    world: world::State,
    assets: Assets,
    /// The next command to execute, if any.
    command: Option<Command>,
    /// Whether the update step of the game loop produced any changes
    /// that need rendering in the draw step.
    updated: bool,
}

impl State {

    fn selected(&self, coords: Offset<OddCol>, hexagon: Hexagon, entity: Option<&Entity>) -> ui::Selected {
        match entity {
            None => ui::Selected { coords, hexagon, range: None },
            Some(entity) => {
                let mut mvc = MovementContext {
                    costs: &self.world.costs,
                    entities: &self.world.entities,
                    grid: self.ui.view.grid(),
                    range: entity.range(),
                };
                let tree = search::astar::tree(coords, None, &mut mvc);
                ui::Selected {
                    coords,
                    hexagon,
                    range: Some(MovementRange { tree, path: None })
                }
            }
        }
    }

    fn select(&self, coords: Offset<OddCol>) -> Option<ui::Selected> {
        self.ui.view.grid().get(coords).map(|h|
            self.selected(coords, h.clone(),
                self.world.entities.get(&coords)))
    }

    fn begin_move(&mut self) -> GameResult<()> {
        // Cut short / complete any previous movement.
        if let Some(mv) = self.world.movement.take() {
            self.world.entities.insert(mv.goal, mv.entity);
        }
        // Take the currently selected movement path.
        let path = self.ui.selected.take()
            .and_then(|s| s.range
            .and_then(|r| r.path
        )).map_or(Vec::new(), Vec::from);
        // Setup the new movement.
        self.world.movement = path.first()
            .and_then(|from| self.world.entities.remove(&from.coords))
            .and_then(|entity| Movement::new(entity, &path, self.ui.view.grid()));
        // Play movement sound.
        for mv in &self.world.movement {
            for sound in mv.entity.sound(&mut self.assets.sounds) {
                sound.play()?;
                sound.set_volume(0.25);
            }
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
        self.ui.selected = self.ui.selected.take().or_else(|| {
            self.ui.panel = ui::ControlPanel::hexagon(ctx, goal, Some(&entity));
            self.ui.view.grid().get(goal).map(|h|
                self.selected(goal, h.clone(), Some(&entity)))
        });
        self.world.entities.insert(goal, entity);
    }

    fn end_turn(&mut self) -> GameResult<()> {
        self.world.end_turn();
        self.ui.end_turn(&self.world)
    }

    /// If the shipyard is selected that has sufficient capacity and
    /// there is a free neighbouring hexagon, place a new ship.
    fn new_ship(&mut self, class: ShipClass) -> Option<(Offset<OddCol>, &Entity)> {
        if let Some(s) = &self.ui.selected {
            if let Some(free) = hexworld::grid::Cube::from(s.coords)
                .neighbours()
                .find_map(|n|
                    Some(Offset::from(n)).filter(|o|
                        self.ui.view.grid().get(*o).is_some() &&
                        !self.world.entities.contains_key(o))) {
                if let Some(e) = self.world.entities.get_mut(&s.coords) {
                    if let Entity::Shipyard(yard) = e {
                        if let Some(ship) = yard.new_ship(class) {
                            let entity = Entity::Ship(ship);
                            self.world.entities.insert(free, entity);
                            // TODO: Just return free
                            return self.world.entities.get(&free).map(|e| (free,e))
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
                self.ui.view.resize(width as u32 - 302, height as u32 - 202);
                let screen = graphics::Rect::new(0., 0., width, height);
                graphics::set_screen_coordinates(ctx, screen)?;
                graphics::present(ctx)?;
                self.ui.scroll_border = scroll::Border {
                    bounds: Bounds {
                        position: Point2::origin(),
                        width,
                        height
                    }, .. self.ui.scroll_border
                };
                Ok(None)
            }

            ScrollView(delta, repeat) => {
                self.ui.view.scroll_x(delta.dx);
                self.ui.view.scroll_y(delta.dy);
                if repeat {
                    Ok(Some(ScrollView(delta, repeat)))
                } else {
                    Ok(None)
                }
            }

            HoverHexagon(coords) => {
                self.ui.hover = coords;
                if let Some(c) = coords {
                    let entity = self.world.entities.get(&c);
                    self.ui.info = Some(ui::Info::new(c, entity));
                    if let Some(ref mut s) = self.ui.selected {
                        if let Some(ref mut r) = s.range {
                            if entity.is_none() {
                                r.path = r.tree.path(c);
                            } else {
                                r.path = None;
                            }
                        }
                    }
                } else {
                    self.ui.info = None;
                }
                Ok(None)
            }

            SelectHexagon(coords) => {
                if self.ui.selected.as_ref()
                    .and_then(|s| s.range.as_ref())
                    .and_then(|r| r.path.as_ref())
                    .and_then(|p| p.back())
                    .map_or(false, |n| Some(n.coords) == coords)
                {
                    // Selected the target hexagon of the currently selected
                    // movement path, thus execute the move.
                    self.begin_move()?;
                } else {
                    self.ui.selected = coords.and_then(|c| self.select(c));
                    self.ui.panel = match coords {
                        Some(c) => ui::ControlPanel::hexagon(ctx, c, self.world.entities.get(&c)),
                        None    => ui::ControlPanel::main(ctx)
                    };
                }
                self.assets.sounds.select.play()?;
                Ok(None)
            }

            SelectButton(btn) => {
                match btn {
                    ui::Button::NewShip(class) => {
                        if let Some((c,e)) = self.new_ship(class) {
                            self.ui.panel = ui::ControlPanel::hexagon(ctx, c, Some(e));
                            self.ui.selected = self.select(c);
                        }
                    },
                    ui::Button::NewAsteroid(size) => {
                        if let Some(s) = &self.ui.selected {
                            if !self.world.entities.contains_key(&s.coords) {
                                self.world.entities.insert(s.coords, Entity::Asteroid(size));
                            }
                        }
                    },
                    ui::Button::IncreaseCost => for s in &self.ui.selected {
                        let v = self.world.costs.entry(s.coords).or_insert(1);
                        *v = usize::min(100, *v + 1);
                    },
                    ui::Button::DecreaseCost => for s in &self.ui.selected {
                        let v = self.world.costs.entry(s.coords).or_insert(1);
                        *v = usize::max(1, *v - 1);
                    },
                    ui::Button::ToggleGrid => {
                        self.ui.settings.show_grid = !self.ui.settings.show_grid;
                    },
                    ui::Button::ToggleCoords => {
                        self.ui.settings.show_coords = !self.ui.settings.show_coords;
                    },
                    ui::Button::ToggleCost => {
                        self.ui.settings.show_cost = !self.ui.settings.show_cost;
                    }
                    ui::Button::EndTurn => {
                        self.end_turn()?;
                    }
                }
                self.assets.sounds.button.play()?;
                Ok(None)
            }

            EndTurn() => {
                self.end_turn()?;
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
    SelectButton(ui::Button),
    /// End the current turn.
    EndTurn()
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, UPDATES_PER_SEC as u32) {
            // Process the command
            let view_updated = self.ui.view.update(); // TODO: Remove
            if let Some(cmd) = self.command.take() {
                self.command = self.apply(ctx, cmd)?;
                self.updated = true;
            }
            // Progress movement(s)
            if let Some(ref mut movement) = self.world.movement {
                if let Some(pos) = movement.pixel_path.next() {
                    movement.pixel_pos = pos;
                }
                else if let Some(mv) = self.world.movement.take() {
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

        self.ui.draw(ctx, &self.world, &self.assets.images)?;

        graphics::present(ctx)?;
        self.updated = false;
        timer::yield_now();

        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _btn: MouseButton, x: f32, y: f32) {
        let p = Point2::new(x, y);
        if let Some(&btn) = self.ui.panel.menu.select(p) {
            self.command = Some(Command::SelectButton(btn))
        } else {
            let coords = self.ui.view.from_pixel(p).map(|(c,_)| c);
            self.command = Some(Command::SelectHexagon(coords));
        }
    }

    fn key_down_event(&mut self, _ctx: &mut Context, code: KeyCode, _mod: KeyMods, repeat: bool) {
        let delta = (10 * if repeat { 2 } else { 1 }) as f32;
        self.command = match code {
            // Key scrolling
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
        // Mouse motion in the scroll-sensitive border region
        // always triggers scrolling.
        let scroll = self.ui.scroll_border.eval(x, y);
        if scroll.dx != 0.0 || scroll.dy != 0.0 {
            self.command = Some(Command::ScrollView(scroll, true))
        }
        // Mouse motion other than border scrolling should never override other
        // pending commands, except for repeated scrolling itself, so that the UI
        // feels responsive and e.g. clicks or key presses are not occasionally
        // "swallowed" by a subsequent (and possibly unintentional) mouse
        // movement.
        else {
            let coords = || self.ui.view.from_pixel(Point2::new(x,y)).map(|(c,_)| c);
            match &self.command {
                None => {
                    let coords = coords();
                    // Only issue a new command if the coordinates changed,
                    // to avoid needless repetitive work (mouse motion events
                    // fire plenty).
                    self.command = if coords != self.ui.hover {
                        Some(Command::HoverHexagon(coords))
                    } else {
                        None
                    }
                }
                // Stop border scrolling.
                Some(Command::ScrollView(_, true)) => {
                    let coords = coords();
                    self.command = Some(Command::HoverHexagon(coords));
                }
                _ => {}
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
    cfg.window_setup.title = "Hexspace".to_string();
    let width = cfg.window_mode.width;
    let height = cfg.window_mode.height;
    let (ctx, game_loop) = &mut ContextBuilder
        ::new("hexspace", "roman")
         .conf(cfg)
         .build()?;

    // mouse::set_grabbed(ctx, true);

    // Load assets
    filesystem::mount(ctx, Path::new("ggez-demo/assets"), true);
    let mut assets = Assets::load(ctx)?;

    // Setup the UI
    let ui = ui::State::new(ctx, 1, width, height);

    // Setup the game world
    let mut world = world::State::new();
    let shipyard = Shipyard::new(1);
    world.entities.insert(Offset::new(0,0), Entity::Shipyard(shipyard));

    // Start soundtrack
    assets.sounds.soundtrack.set_repeat(true);
    assets.sounds.soundtrack.play()?;

    // Run the game
    let state = &mut State {
        ui,
        world,
        assets,
        updated: false,
        command: None,
    };

    event::run(ctx, game_loop, state)
}

