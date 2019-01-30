
use crate::entity::{ Entity };

use hexworld::grid::Grid;
use hexworld::grid::offset::*;
use hexworld::search;

use hexworld_ggez::animation;

use std::collections::HashMap;
use std::collections::VecDeque;

use nalgebra::Point2;

pub const UPDATES_PER_SEC: u16 = 60;
const MOVE_HEX_SECS:   f32 = 0.15;

pub struct MovementRange {
    pub tree: search::Tree<Offset<OddCol>>,
    pub path: Option<VecDeque<search::Node<Offset<OddCol>>>>,
}

pub struct Movement {
    pub entity: Entity,
    pub goal: Offset<OddCol>,
    pub cost: usize,
    pub pixel_path: animation::PathIter,
    pub pixel_pos: Point2<f32>,
}

impl Movement {
    pub fn new(
        entity: Entity,
        path: &Vec<search::Node<Offset<OddCol>>>,
        grid: &Grid<Offset<OddCol>>
    ) -> Option<Movement> {
        path.first()
            .and_then(|from| path.last().filter(|c| *c != from)
            .map(|to| {
                let pixel_path = animation::path(UPDATES_PER_SEC, MOVE_HEX_SECS, grid, &path);
                Movement {
                    entity,
                    goal: to.coords,
                    cost: to.cost,
                    pixel_path,
                    pixel_pos: Point2::origin(),
                }
            }))
    }
}

pub struct MovementContext<'a> {
    pub range: u16,
    pub grid: &'a Grid<Offset<OddCol>>,
    pub entities: &'a HashMap<Offset<OddCol>, Entity>,
    pub costs: &'a HashMap<Offset<OddCol>, usize>,
}

impl<'a> search::Context<Offset<OddCol>> for MovementContext<'a> {
    fn max_cost(&self) -> usize {
        self.range as usize
    }
    fn cost(&mut self, _from: Offset<OddCol>, to: Offset<OddCol>) -> Option<usize> {
        self.grid.get(to).and_then(|_|
            self.costs.get(&to).map(|c| *c).or_else(||
                match self.entities.get(&to) {
                    // Other entities are impassable
                    Some(_) => None,
                    // Empty space has a default cost of 1
                    _ => Some(1)
                }))
    }
}

