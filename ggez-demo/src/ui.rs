
use crate::assets::*;
use crate::entity::*;
use crate::menu::*;
use crate::movement::*;
use crate::world;

use ggez::{ Context, GameResult };
use ggez::graphics;
use ggez::graphics::*; // { Drawable, DrawParam, MeshBuilder };
use hexworld::geo::*;
use hexworld::grid::Grid;
use hexworld::grid::offset::{ Offset, OddCol };
use hexworld::grid::shape;
use hexworld::ui::gridview;
use hexworld::ui::scroll;
use hexworld_ggez::image;
use hexworld_ggez::mesh;
use nalgebra::{ Point2, Vector2 };

use std::borrow::Cow;

pub const RED:  graphics::Color = graphics::Color { r: 1.,  g: 0.,  b: 0.,  a: 0.7 };
pub const BLUE: graphics::Color = graphics::Color { r: 0.,  g: 0.,  b: 1.,  a: 1.  };
pub const GREY: graphics::Color = graphics::Color { r: 0.5, g: 0.5, b: 0.5, a: 0.7 };

// TODO: type Offset = coords::Offset<coords::OddCol>;
// TODO: type OffsetMap<T> = HashMap<Offset,T>;

pub struct State {
    pub view: gridview::State<Offset<OddCol>>,
    pub scroll_border: scroll::Border,
    pub hover: Option<Offset<OddCol>>,
    pub selected: Option<Selected>,
    pub info: Option<Info>,
    pub turn: graphics::Text,
    pub panel: ControlPanel,
    pub settings: Settings,
}

impl State {
    pub fn new(
        ctx: &mut Context,
        turn: usize,
        width: f32,
        height: f32,
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
            turn: graphics::Text::new(format!("Turn {}", turn)),
            scroll_border,
            selected: None,
            hover: None,
            info: None,
            panel: ControlPanel::main(ctx),
            settings: Settings::default(),
        }
    }

    pub fn end_turn(&mut self, world: &world::State) -> GameResult<()> {
        self.turn = graphics::Text::new(format!("Turn {}", world.turn));
        Ok(())
    }

    pub fn draw(&mut self, ctx: &mut Context, world: &world::State, images: &Images) -> GameResult<()> {
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
            let img = entity.image(images);
            for hex in self.view.grid().get(*pos) {
                image::draw_into(ctx, &img, hex, schema, grid_dest)?;
            }
        }

        // Movement
        if let Some(mv) = &world.movement {
            let img = mv.entity.image(images);
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
