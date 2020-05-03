
//! The game world.

use crate::assets::*;

use hexkit::grid::coords;
use hexkit::grid::Grid;
use hexkit::search;

use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub type Coords = coords::Offset<coords::OddCol>;
pub type WorldMap<T> = HashMap<Coords,T>;
pub type Range = search::Tree<Coords>;
pub type Path = search::Path<Coords>;

/// The core state of the game world.
pub struct State {
    /// The current turn.
    turn: usize,
    entities: WorldMap<Entity>,
    costs: WorldMap<usize>,
}

impl State {
    /// Creates a new, empty world state that begins at turn 1.
    pub fn new() -> State {
        State {
            turn: 1,
            entities: HashMap::new(),
            costs: HashMap::new(),
        }
    }

    pub fn turn(&self) -> usize {
        self.turn
    }

    pub fn range(&self, entity: &Entity, at: Coords, grid: &Grid<Coords>) -> Range {
        let mut mvc = MovementContext { world: self, entity, grid };
        search::astar::tree(at, None, &mut mvc)
    }

    pub fn begin_move(&mut self, path: Path) -> Option<Movement> {
        path.front()
            .and_then(|start| path.back()
                .and_then(|end|
                    if start != end {
                        Some((start.clone(), end.clone()))
                    } else {
                        None
                    }))
            .and_then(|(start, end)| {
                if let Entry::Occupied(e) = self.entities.entry(start.coords) {
                    if e.get().range() >= end.cost as u16 {
                        Some(Movement {
                            entity: e.remove(),
                            start: start.coords,
                            goal: end.coords,
                            cost: end.cost,
                            path: path.to_vec(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
    }

    pub fn end_move(&mut self, mv: Movement) -> &Entity {
        let mut entity = mv.entity;
        entity.reduce_range(mv.cost as u16);
        &*match self.entities.entry(mv.goal) {
            Entry::Vacant(v) => v.insert(entity),
            Entry::Occupied(mut o) => {
                o.insert(entity);
                o.into_mut()
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=(&Coords, &Entity)> {
        self.entities.iter()
    }

    pub fn entity(&self, at: Coords) -> Option<&Entity> {
        self.entities.get(&at)
    }

    pub fn cost(&self, at: Coords) -> Option<usize> {
        self.costs.get(&at).cloned().or_else(||
            match self.entities.get(&at) {
                // Other entities are impassable
                Some(_) => None,
                // Empty space has a default cost of 1
                _ => Some(1)
            })
    }

    pub fn new_ship(&mut self, yard_at: Coords, ship_at: Coords, class: ShipClass) -> Option<&Entity> {
        self.entities.get_mut(&yard_at)
            .and_then(|e|
                if let Entity::Shipyard(yard) = e {
                    yard.new_ship(class)
                } else {
                    None
                })
            .map(move |ship| {
                let entity = Entity::Ship(ship);
                &*match self.entities.entry(ship_at) {
                    Entry::Vacant(v) => v.insert(entity),
                    Entry::Occupied(mut o) => {
                        o.insert(entity);
                        o.into_mut()
                    }
                }
            })
    }

    pub fn new_asteroid(&mut self, at: Coords, size: Asteroid) {
        self.entities.insert(at, Entity::Asteroid(size));
    }

    pub fn new_shipyard(&mut self, at: Coords, yard: Shipyard) {
        self.entities.insert(at, Entity::Shipyard(yard));
    }

    pub fn increase_cost(&mut self, at: Coords) {
        let v = self.costs.entry(at).or_insert(1);
        *v = usize::min(100, *v + 1);
    }

    pub fn decrease_cost(&mut self, at: Coords) {
        let v = self.costs.entry(at).or_insert(1);
        *v = usize::max(1, *v - 1);
    }

    pub fn end_turn(&mut self) {
        for entity in self.entities.values_mut() {
            match entity {
                Entity::Ship(ship) => {
                    let spec = ship.class.spec();
                    ship.range = spec.range;
                }
                Entity::Shipyard(yard) => {
                    yard.capacity += 1;
                }
                Entity::Asteroid(_) => {}
            }
        }
        self.turn += 1;
    }
}

pub struct Movement {
    pub entity: Entity,
    pub start: Coords,
    pub goal: Coords,
    pub cost: usize,
    pub path: Vec<search::Node<Coords>>,
}

struct MovementContext<'a> {
    pub grid: &'a Grid<Coords>,
    pub world: &'a State,
    pub entity: &'a Entity,
}

impl<'a> search::Context<Coords> for MovementContext<'a> {
    fn max_cost(&self) -> usize {
        self.entity.range() as usize
    }

    fn cost(&mut self, _from: Coords, to: Coords) -> Option<usize> {
        self.grid.get(to).and_then(|_| self.world.cost(to))
    }
}

pub enum Entity {
    Shipyard(Shipyard),
    Ship(Ship),
    Asteroid(Asteroid),
}

impl Entity {

    pub fn name(&self) -> Cow<str> {
        match self {
            Entity::Ship(ship)  => Cow::Owned(ship.name()),
            Entity::Shipyard(_) => Cow::Borrowed("Shipyard"),
            Entity::Asteroid(_) => Cow::Borrowed("Asteroid"),
        }
    }

    pub fn image<'a>(&'a self, images: &'a Images) -> &'a graphics::Image {
        match self {
            Entity::Ship(ship)     => ship.class.image(images),
            Entity::Shipyard(_)    => &images.shipyard,
            Entity::Asteroid(size) => match size {
                Asteroid::Small => &images.asteroid_small,
                Asteroid::Large => &images.asteroid_large,
            }
        }
    }

    pub fn range(&self) -> u16 {
        match self {
            Entity::Ship(ship)  => ship.range,
            Entity::Shipyard(_) => 0,
            Entity::Asteroid(_) => 0,
        }
    }

    pub fn reduce_range(&mut self, sub: u16) {
        match self {
            Entity::Ship(ship)  => ship.range -= sub,
            Entity::Shipyard(_) => {}
            Entity::Asteroid(_) => {}
        }
    }

    pub fn sound<'a>(&'a self, sounds: &'a mut Sounds) -> Option<&'a mut audio::Source> {
        match self {
            Entity::Ship(ship) => Some(ship.class.sound(sounds)),
            _                  => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Shipyard {
    /// The total number of produced ships.
    pub count: u16,
    /// The remaining production capacity.
    pub capacity: u16,
}

impl Shipyard {
    pub fn new(capacity: u16) -> Shipyard {
        Shipyard { capacity, count: 0 }
    }

    pub fn new_ship(&mut self, class: ShipClass) -> Option<Ship> {
        let ship_capacity = class.spec().shipyard_capacity;
        if self.capacity >= ship_capacity {
            self.count += 1;
            self.capacity -= ship_capacity;
            Some(Ship::new(self.count, class))
        } else {
            None
        }
    }
}

pub type ShipId = u16;

#[derive(Debug, Copy, Clone)]
pub enum ShipClass {
    Fighter, Scout, Battleship, Carrier
}

#[derive(Debug, Clone)]
pub struct ShipSpec {
    pub range: u16,
    pub shipyard_capacity: u16,
}

const SHIP_CLASSES: [ShipClass; 4] =
    [ ShipClass::Fighter
    , ShipClass::Scout
    , ShipClass::Carrier
    , ShipClass::Battleship
    ];

impl ShipClass {
    pub fn iter() -> impl Iterator<Item=ShipClass> {
        SHIP_CLASSES.iter().map(|c| *c)
    }

    /// Get the (technical) specifications of a ship class,
    /// describing its game-relevant attributes.
    pub fn spec(&self) -> ShipSpec {
        use ShipClass::*;
        match self {
            Fighter => ShipSpec {
                range: 2,
                shipyard_capacity: 1,
            },
            Scout => ShipSpec {
                range: 10,
                shipyard_capacity: 3,
            },
            Battleship => ShipSpec {
                range: 5,
                shipyard_capacity: 10,
            },
            Carrier => ShipSpec {
                range: 3,
                shipyard_capacity: 8,
            }
        }
    }

    pub fn name(&self) -> &str {
        use ShipClass::*;
        match self {
            Fighter    => "Fighter",
            Scout      => "Scout",
            Battleship => "Battleship",
            Carrier    => "Carrier",
        }
    }

    /// Select an image for a ship class.
    pub fn image<'a>(&'a self, images: &'a Images) -> &'a graphics::Image {
        use ShipClass::*;
        match self {
            Fighter    => &images.fighter,
            Scout      => &images.scout,
            Battleship => &images.battleship,
            Carrier    => &images.carrier
        }
    }

    pub fn sound<'a>(&'a self, sounds: &'a mut Sounds) -> &'a mut audio::Source {
        &mut sounds.engine
    }
}

#[derive(Debug, Clone)]
pub struct Ship {
    pub id: ShipId,
    pub class: ShipClass,
    pub range: u16,
}

impl Ship {
    fn new(id: ShipId, class: ShipClass) -> Ship {
        let range = class.spec().range;
        Ship { id, class, range }
    }

    fn name(&self) -> String {
        format!("{} (#{})", self.class.name(), self.id)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Asteroid {
    Small,
    Large
}

