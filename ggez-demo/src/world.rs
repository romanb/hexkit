
use crate::entity::*;
use crate::movement::*;

use hexworld::grid::offset::{ Offset, OddCol };

use std::collections::HashMap;

pub struct State {
    pub turn: usize,
    pub entities: HashMap<Offset<OddCol>, Entity>,
    pub costs: HashMap<Offset<OddCol>, usize>,
    /// There is at most one ongoing movement at a time.
    pub movement: Option<Movement>,
}

impl State {
    pub fn new() -> State {
        State {
            turn: 1,
            entities: HashMap::new(),
            costs: HashMap::new(),
            movement: None,
        }
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

