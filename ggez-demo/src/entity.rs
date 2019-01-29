
use ggez::graphics;
use ggez::audio;
// use hexworld::grid::offset::*;
use std::borrow::Cow;

use crate::assets::{ Images, Sounds };

pub enum Entity {
    Shipyard(Shipyard),
    Ship(Ship),
}

pub trait SomeEntity {
    fn name(&self) -> Cow<str>;
    fn image<'a>(&'a self, images: &'a Images) -> &'a graphics::Image;
    fn sound<'a>(&'a self, sounds: &'a mut Sounds) -> &'a mut audio::Source;
    fn range(&self) -> u16;
    fn reduce_range(&mut self, sub: u16);
}

impl Entity {
    pub fn name(&self) -> Cow<str> {
        use Entity::*;
        match self {
            Ship(ship) => Cow::Owned(ship.name()),
            Shipyard(_) => Cow::Borrowed("Shipyard"),
        }
    }

    pub fn image<'a>(&'a self, images: &'a Images) -> &'a graphics::Image {
        use Entity::*;
        match self {
            Ship(ship) => ship.class.image(images),
            Shipyard(_) => &images.shipyard,
        }
    }

    pub fn range(&self) -> u16 {
        use Entity::*;
        match self {
            Ship(ship) => ship.range,
            Shipyard(_) => 0,
        }
    }

    pub fn reduce_range(&mut self, sub: u16) {
        use Entity::*;
        match self {
            Ship(ship) => ship.range -= sub,
            Shipyard(_) => {}
        }
    }

    pub fn sound<'a>(&'a self, sounds: &'a mut Sounds) -> &'a mut audio::Source {
        use Entity::*;
        match self {
            Ship(ship) => ship.class.sound(sounds),
            Shipyard(_) => &mut sounds.engine
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

// impl Entity for Shipyard {
//     fn name(&self) -> &str {
//         "Shipyard"
//     }
// }

pub type ShipId = u16;

#[derive(Debug, Clone)]
pub enum ShipClass {
    Fighter, Scout, Battleship, Carrier
}

#[derive(Debug, Clone)]
pub struct ShipSpec {
    pub range: u16,
    pub shipyard_capacity: u16,
}

impl ShipClass {
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

    pub fn sound<'a,'b>(&'a self, sounds: &'b mut Sounds) -> &'b mut audio::Source {
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

// impl Entity for Ship {
//     fn name(&self) -> &str {
//         self.class.name()
//     }
// }

