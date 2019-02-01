
use crate::entity::*;
// use crate::movement::*;

use hexworld::grid::offset::{ Offset, OddCol };
use hexworld::search;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::VecDeque;

pub struct State {
    pub turn: usize,
    pub entities: HashMap<Offset<OddCol>, Entity>,
    pub costs: HashMap<Offset<OddCol>, usize>,
}

impl State {
    pub fn new() -> State {
        State {
            turn: 1,
            entities: HashMap::new(),
            costs: HashMap::new(),
        }
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

    // pub fn begin_move(&mut self, path: search::Path<Offset<OddCol>>) -> Option<Movement>
    pub fn begin_move(&mut self, path: VecDeque<search::Node<Offset<OddCol>>>) -> Option<Movement> {
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
                            path: Vec::from(path),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
    }

    pub fn entity(&self, at: Offset<OddCol>) -> Option<&Entity> {
        self.entities.get(&at)
    }

    pub fn cost(&self, at: Offset<OddCol>) -> Option<usize> {
        self.costs.get(&at).map(|c| *c).or_else(||
            match self.entities.get(&at) {
                // Other entities are impassable
                Some(_) => None,
                // Empty space has a default cost of 1
                _ => Some(1)
            })
    }

    pub fn new_ship(&mut self, yard_at: Offset<OddCol>, ship_at: Offset<OddCol>, class: ShipClass) -> Option<&Entity> {
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
    pub start: Offset<OddCol>,
    pub goal: Offset<OddCol>,
    pub cost: usize,
    pub path: Vec<search::Node<Offset<OddCol>>>,
}

