
use crate::assets::*;
use crate::entity::*;
use crate::menu::*;
use crate::movement::*;
use crate::world;

use ggez::{ Context, GameResult };
use ggez::graphics;
use ggez::graphics::*;
use hexworld::geo::*;
use hexworld::grid::Grid;
use hexworld::grid::offset::{ Offset, OddCol };
use hexworld::grid::shape;
use hexworld::ui::gridview;
use hexworld::ui::scroll;
use hexworld::search;
use hexworld_ggez::animation;
use hexworld_ggez::image;
use hexworld_ggez::mesh;
use nalgebra::{ Point2, Vector2 };

use std::collections::VecDeque;
use std::borrow::Cow;

pub const RED:  graphics::Color = graphics::Color { r: 1.,  g: 0.,  b: 0.,  a: 0.7 };
pub const BLUE: graphics::Color = graphics::Color { r: 0.,  g: 0.,  b: 1.,  a: 1.  };
pub const GREY: graphics::Color = graphics::Color { r: 0.5, g: 0.5, b: 0.5, a: 0.7 };

pub const UPDATES_PER_SEC: u16 = 60;
    const MOVE_HEX_SECS:   f32 = 0.15;

// TODO: type Offset = coords::Offset<coords::OddCol>;
// TODO: type OffsetMap<T> = HashMap<Offset,T>;

/// The commands that drive the game (state).
pub enum Command { // Input?
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

pub struct State {
        view: gridview::State<Offset<OddCol>>,
    pub scroll_border: scroll::Border,
    pub hover: Option<Offset<OddCol>>,
        selected: Option<Selected>,
        info: Option<Info>,
        turn: graphics::Text,
    pub panel: ControlPanel,
        settings: Settings,
        movement: Option<Movement>,
        assets: Assets,
}

impl State {
    pub fn new(
        ctx: &mut Context,
        turn: usize,
        width: f32,
        height: f32,
        assets: Assets,
    ) -> State {
        // A border region for scrolling the view
        let scroll_border = scroll::Border {
            bounds: Bounds { position: Point2::origin(), width, height },
            scale: 1.0,
            width: 25.0,
        };

        // Setup the hexagonal grid
        let schema = Schema::new(SideLength(50.), Orientation::FlatTop);
        let grid = Grid::new(schema, shape::rectangle_xz_odd(30, 30));
        let bounds = Bounds {
            position: Point2::new(201., 101.),
            width: width - 302.,
            height: height - 302.,
        };
        let view = gridview::State::new(grid, bounds);

        State {
            view,
            scroll_border,
            turn: graphics::Text::new(format!("Turn {}", turn)),
            selected: None,
            hover: None,
            info: None,
            panel: ControlPanel::main(ctx),
            settings: Settings::default(),
            movement: None,
            assets,
        }
    }

    pub fn view(&self) -> &gridview::State<Offset<OddCol>> {
        &self.view
    }

    /// Apply a command to the game state, updating it appropriately.
    /// Execution of a command optionally yields another command to
    /// execute, e.g. to repeat an operation.
    pub fn apply(&mut self, ctx: &mut Context, world: &mut world::State, cmd: Command) -> GameResult<Option<Command>> {
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
                    let entity = world.entities.get(&c);
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
                    self.begin_move(world)?;
                } else {
                    match coords {
                        Some(c) => self.select(ctx, c, world),
                        None => self.panel = ControlPanel::main(ctx)
                    };
                }
                self.assets.sounds.select.play()?;
                Ok(None)
            }

            SelectButton(btn) => {
                match btn {
                    Button::NewShip(class) => {
                        if let Some(c) = self.new_ship(world, class) {
                            self.select(ctx, c, world);
                        }
                    },
                    Button::NewAsteroid(size) => {
                        if let Some(s) = &self.selected {
                            if !world.entities.contains_key(&s.coords) {
                                world.entities.insert(s.coords, Entity::Asteroid(size));
                            }
                        }
                    },
                    Button::IncreaseCost => for s in &self.selected {
                        let v = world.costs.entry(s.coords).or_insert(1);
                        *v = usize::min(100, *v + 1);
                    },
                    Button::DecreaseCost => for s in &self.selected {
                        let v = world.costs.entry(s.coords).or_insert(1);
                        *v = usize::max(1, *v - 1);
                    },
                    Button::ToggleGrid => {
                        self.settings.show_grid = !self.settings.show_grid;
                    },
                    Button::ToggleCoords => {
                        self.settings.show_coords = !self.settings.show_coords;
                    },
                    Button::ToggleCost => {
                        self.settings.show_cost = !self.settings.show_cost;
                    }
                    Button::EndTurn => {
                        self.end_turn(world)?;
                    }
                }
                self.assets.sounds.button.play()?;
                Ok(None)
            }

            EndTurn() => {
                self.end_turn(world)?;
                Ok(None)
            }
        }
    }

    /// If the shipyard is selected that has sufficient capacity and
    /// there is a free neighbouring hexagon, place a new ship.
    fn new_ship(&mut self, world: &mut world::State, class: ShipClass) -> Option<Offset<OddCol>> {
        if let Some(s) = &self.selected {
            if let Some(free) = hexworld::grid::Cube::from(s.coords)
                .neighbours()
                .find_map(|n|
                    Some(Offset::from(n))
                        .filter(|o|
                            self.view.grid().get(*o).is_some() &&
                            world.entity(*o).is_none()))
            {
                if world.new_ship(s.coords, free, class).is_some() {
                    return Some(free)
                }
            }
        }
        return None
    }

    fn selected(&self, coords: Offset<OddCol>, hexagon: Hexagon, entity: Option<&Entity>, world: &world::State) -> Selected {
        match entity {
            None => Selected { coords, hexagon, range: None },
            Some(entity) => {
                let mut mvc = MovementContext {
                    world,
                    entity,
                    grid: self.view.grid(),
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

    pub fn select(&mut self, ctx: &mut Context, coords: Offset<OddCol>, world: &world::State) {
        let entity = world.entity(coords);
        self.selected = self.view.grid().get(coords).map(|h|
            self.selected(coords, h.clone(), entity, world));
        self.panel = ControlPanel::hexagon(ctx, coords, entity);
    }

    pub fn update(&mut self, ctx: &mut Context, world: &mut world::State) -> bool {
        let view_updated = self.view.update(); // TODO: Remove
        // Progress movement(s)
        if let Some(mv) = &mut self.movement {
            if let Some(pos) = mv.pixel_path.next() {
                mv.pixel_pos = pos;
            }
            else if let Some(mv) = self.movement.take() {
                // Movement is complete.
                self.end_move(ctx, world, mv);
            }
            true
        } else {
            view_updated
        }
    }

    pub fn begin_move(&mut self, world: &mut world::State) -> GameResult<()> {
        // Cut short / complete any previous movement.
        if let Some(prev) = self.movement.take() {
            world.end_move(prev.inner);
        }
        // Take the currently selected movement path.
        let path = self.selected.take()
            .and_then(|s| s.range
            .and_then(|r| r.path
        )).unwrap_or(VecDeque::new());
        // Setup the new movement.
        for world_move in world.begin_move(path) {
            let mv = Movement::new(world_move, self.view.grid());
            for sound in mv.inner.entity.sound(&mut self.assets.sounds) {
                sound.play()?;
                sound.set_volume(0.25);
            }
            self.movement = Some(mv);
        }
        Ok(())
    }

    fn end_move(&mut self, ctx: &mut Context, world: &mut world::State, mv: Movement) {
        let goal = mv.inner.goal;
        world.end_move(mv.inner);
        let entity = world.entity(goal);
        // If nothing else has been selected meanwhile, select the
        // ship again to continue movement.
        self.selected = self.selected.take().or_else(|| {
            self.panel = ControlPanel::hexagon(ctx, goal, entity);
            self.view.grid().get(goal).map(|h|
                self.selected(goal, h.clone(), entity, world))
        });
    }

    pub fn end_turn(&mut self, world: &mut world::State) -> GameResult<()> {
        world.end_turn();
        // TODO: Refresh control panel
        self.turn = graphics::Text::new(format!("Turn {}", world.turn));
        Ok(())
    }

    pub fn draw(&mut self, ctx: &mut Context, world: &world::State) -> GameResult<()> {
        // The base grid
        let mesh = &mut MeshBuilder::new();
        let grid_dest = self.view.grid_position();
        let grid_dp = DrawParam::default().dest(grid_dest);
        let schema = self.view.grid().schema();
        for (coords, hex) in self.view.iter_viewport() {
            // Hexagon
            if self.settings.show_grid {
                mesh.polygon(DrawMode::Line(1.), hex.corners(), GREY)?;
            }
            // Coordinates label
            if self.settings.show_coords {
                hexworld_ggez::text::queue_label(
                    ctx, schema, &hex, coords.to_string(),
                    VAlign::Bottom, WHITE, Scale::uniform(12.));
            }
            // Cost label
            if self.settings.show_cost {
                let cost = *world.costs.get(coords).unwrap_or(&1);
                hexworld_ggez::text::queue_label(
                    ctx, schema, &hex, cost.to_string(),
                    VAlign::Middle, WHITE, Scale::uniform(graphics::DEFAULT_FONT_SCALE));
            }
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
        for (pos, entity) in &world.entities {
            let img = entity.image(&mut self.assets.images);
            for hex in self.view.grid().get(*pos) {
                image::draw_into(ctx, &img, hex, schema, grid_dest)?;
            }
        }

        // Movement
        if let Some(mv) = &self.movement {
            let img = mv.inner.entity.image(&mut self.assets.images);
            let vec = Vector2::new(img.width() as f32 / 2., img.height() as f32 / 2.);
            let img_dest = grid_dest + mv.pixel_pos.coords - vec;
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

        // Hover info box (part of HUD)
        if let Some(info) = &self.info {
            let info_width = info.text.width(ctx);
            let dest = Point2::new(width / 2. - info_width as f32 / 2., height - 50.);
            info.text.draw(ctx, DrawParam::default().dest(dest))?;
        }

        // Turn tracker (part of HUD)
        let turn_width = self.turn.width(ctx);
        let dest = Point2::new(width / 2. - turn_width as f32 / 2., 50.);
        self.turn.draw(ctx, DrawParam::default().dest(dest))?;

        // Menu (part of HUD)
        self.panel.draw(ctx)?;

        Ok(())
    }
}

pub struct Movement {
    pub inner: world::Movement,
    pub pixel_path: animation::PathIter,
    pub pixel_pos: Point2<f32>,
}

impl Movement {
    pub fn new(
        mv: world::Movement,
        grid: &Grid<Offset<OddCol>>
    ) -> Movement {
        let pixel_path = animation::path(UPDATES_PER_SEC, MOVE_HEX_SECS, grid, &mv.path);
        Movement {
            inner: mv,
            pixel_path,
            pixel_pos: Point2::origin(),
        }
    }
}

pub struct Settings {
    pub show_grid: bool,
    pub show_coords: bool,
    pub show_cost: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            show_grid: true,
            show_coords: true,
            show_cost: true,
        }
    }
}

pub struct ControlPanel {
    pub info: Option<graphics::Text>,
    pub menu: Menu<Button>,
}

impl ControlPanel {
    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if let Some(info) = &self.info {
            let info_w = info.width(ctx) as f32;
            let dest = Point2::new((200. - info_w) / 2., 100.);
            info.draw(ctx, DrawParam::default().dest(dest))?;
        }
        self.menu.draw(ctx)
    }

    pub fn main(_ctx: &mut Context) -> ControlPanel {
        let mut menu = Menu::new(Point2::new(25., 100.), 150., 30.);
        menu.add(Button::ToggleGrid, "Toggle Grid");
        menu.add(Button::ToggleCoords, "Toggle Coordinates");
        menu.add(Button::ToggleCost, "Toggle Costs");
        menu.add(Button::EndTurn, "End Turn");
        ControlPanel { info: None, menu }
    }

    pub fn hexagon(ctx: &mut Context, coords: Offset<OddCol>, entity: Option<&Entity>) -> ControlPanel {
        // Info
        let title = entity.map_or(Cow::Borrowed("Empty Space"), |e| e.name());
        let mut text = graphics::Text::new(format!("{} - {}", coords, title));
        match entity {
            None => {}
            Some(Entity::Ship(ship)) => {
                text.add(format!("\nRange: {}/{}",
                    ship.range,
                    ship.class.spec().range));
            }
            Some(Entity::Shipyard(yard)) => {
                text.add(format!("\nCapacity: {}\n(+1 per turn)", yard.capacity));
            }
            Some(Entity::Asteroid(size)) => {
                text.add(format!("\nSize: {:?}", size));
            }
        }
        text.set_bounds(Point2::new(150., 100.), graphics::Align::Center);
        let text_h = text.height(ctx) as f32;
        let info = Some(text);
        // Menu
        let menu_y = 100. + text_h + 25.;
        let mut menu = Menu::new(Point2::new(25., menu_y), 150., 30.);
        match entity {
            None => {
                menu.add(Button::IncreaseCost, "Increase Cost");
                menu.add(Button::DecreaseCost, "Decrease Cost");
                menu.add(Button::NewAsteroid(Asteroid::Small), "Small Asteroid");
                menu.add(Button::NewAsteroid(Asteroid::Large), "Large Asteroid");
            }
            Some(Entity::Ship(_)) => {}
            Some(Entity::Shipyard(_)) => {
                for class in ShipClass::iter() {
                    menu.add(Button::NewShip(class),
                        &format!("{} ({}C)",
                            class.name(),
                            class.spec().shipyard_capacity));
                }
            }
            Some(Entity::Asteroid(_)) => {}
        }
        ControlPanel { info, menu }
    }
}

/// Context-sensitive control panel buttons.
#[derive(Copy, Clone, Debug)]
pub enum Button {
    NewShip(ShipClass),
    NewAsteroid(Asteroid),
    IncreaseCost,
    DecreaseCost,
    ToggleGrid,
    ToggleCoords,
    ToggleCost,
    EndTurn,
}

/// Information about a hexagon.
pub struct Info {
    pub text: graphics::Text
}

impl Info {
    pub fn new(coords: Offset<OddCol>, entity: Option<&Entity>) -> Info {
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

pub struct Selected {
    pub coords: Offset<OddCol>,
    pub hexagon: Hexagon,
    pub range: Option<MovementRange>,
}

// impl Selected {
//     pub fn new(&self, coords: Offset<OddCol>, hexagon: Hexagon, entity: Option<&Entity>) -> Selected {
//         match entity {
//             None => Selected { coords, hexagon, range: None },
//             Some(entity) => {
//                 let mut mvc = MovementContext {
//                     costs: &self.costs,
//                     grid: self.view.grid(),
//                     entities: &self.entities,
//                     range: entity.range(),
//                 };
//                 let tree = search::astar::tree(coords, None, &mut mvc);
//                 Selected {
//                     coords,
//                     hexagon,
//                     range: Some(MovementRange { tree, path: None })
//                 }
//             }
//         }
//     }
// }
