
use ggez::graphics;
use ggez::audio;
use hexworld::grid::offset::*;
use std::borrow::Cow;

use crate::assets::{ Images, Sounds };

pub enum Entity<'a> {
    Shipyard(&'a Shipyard),
    Ship(&'a Ship),
    Space,
}

impl<'a> Entity<'a> {
    pub fn name(&self) -> Cow<str> {
        use Entity::*;
        match self {
            Ship(ship) => Cow::Owned(ship.name()),
            Shipyard(_) => Cow::Borrowed("Shipyard"),
            Space => Cow::Borrowed("Empty Space")
        }
    }
}

// pub struct Space;

// impl Entity for Space {
//     fn name(&self) -> &str {
//         "Empty Space"
//     }
// }

pub struct Shipyard {
    /// The coordinates of the shipyard.
    pub coords: Offset<OddCol>,
    /// The total number of produced ships.
    pub count: u16,
    /// The remaining production capacity.
    pub capacity: u16,
}

impl Shipyard {
    pub fn new(coords: Offset<OddCol>, capacity: u16) -> Shipyard {
        Shipyard { coords, capacity, count: 0 }
    }

    pub fn new_ship(&mut self, class: ShipClass) -> Ship {
        self.count += 1;
        Ship::new(self.count, class)
    }
}

// impl Entity for Shipyard {
//     fn name(&self) -> &str {
//         "Shipyard"
//     }
// }

pub type ShipId = u16;

pub enum ShipClass {
    Fighter, Scout, Battleship, Carrier
}

pub struct ShipSpec {
    pub range: u16,
    // name: String,
    // attack: u16,
    // cost: u16,
}

impl ShipClass {
    /// Get the (technical) specifications of a ship class,
    /// describing its game-relevant attributes.
    pub fn spec(&self) -> ShipSpec {
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
            Carrier => ShipSpec {
                range: 3,
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
    pub fn image(&self, images: &Images) -> graphics::Image {
        use ShipClass::*;
        match self {
            Fighter    => images.fighter.clone(),
            Scout      => images.scout.clone(),
            Battleship => images.battleship.clone(),
            Carrier    => images.carrier.clone()
        }
    }

    pub fn sound<'a>(&'a self, sounds: &'a mut Sounds) -> &'a mut audio::Source {
        &mut sounds.engine
    }
}

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

// impl Entity for Ship {
//     fn name(&self) -> &str {
//         self.class.name()
//     }
// }

