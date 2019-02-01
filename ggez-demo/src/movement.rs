
use crate::entity::{ Entity };
use crate::world;

use hexworld::grid::Grid;
use hexworld::grid::offset::*;
use hexworld::search;

use hexworld_ggez::animation;

use std::collections::HashMap;
use std::collections::VecDeque;

use nalgebra::Point2;

pub struct MovementRange {
    pub tree: search::Tree<Offset<OddCol>>,
    pub path: Option<VecDeque<search::Node<Offset<OddCol>>>>,
}

pub struct MovementContext<'a> {
    pub grid: &'a Grid<Offset<OddCol>>,
    pub world: &'a world::State,
    pub entity: &'a Entity,
}

impl<'a> search::Context<Offset<OddCol>> for MovementContext<'a> {
    fn max_cost(&self) -> usize {
        self.entity.range() as usize
    }
    fn cost(&mut self, _from: Offset<OddCol>, to: Offset<OddCol>) -> Option<usize> {
        self.grid.get(to).and_then(|_| self.world.cost(to))
    }
}

