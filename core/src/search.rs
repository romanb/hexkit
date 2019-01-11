
pub mod astar;
pub mod bfs;

use crate::grid::Coords;

use std::collections::HashMap;
use std::collections::VecDeque;

/// The context of a search defines the cost and bounds of the search space.
pub trait Context<C: Coords> {
    fn max_cost(&self) -> usize {
        std::usize::MAX
    }
    fn max_distance(&self) -> usize {
        std::usize::MAX
    }
    fn exit(&mut self, _next: C) -> bool {
        false
    }
    fn heuristic(&mut self, from: C, to: C) -> usize {
        from.into().distance(to.into())
    }
    fn cost(&mut self, from: C, to: C) -> Option<usize>;
}

/// A tree is constructed as the result of a search on a grid.
/// The root node of the tree is the start coordinates of the search
/// and the paths to the leaves are paths on the grid from the start
/// coordinates to other grid coordinates.
pub struct Tree<C> {
    root: C,
    parents: HashMap<C, C>,
    costs: HashMap<C, usize>,
}

/// A node in a path of a search tree.
#[derive(Debug)]
pub struct Node<C> {
    pub coords: C,
    pub cost: usize,
}

impl<C> Node<C> {
    fn new(coords: C, cost: usize) -> Node<C> {
        Node { coords, cost }
    }
}

impl<C: Coords> Tree<C> {
    /// Trace a path from the given goal back to the root of the tree. The path
    /// is returned in the natural (i.e. reverse) order from start to goal.
    pub fn path(&self, goal: C) -> Option<VecDeque<Node<C>>> {
        // let mut path = VecDeque::with_capacity(coords::distance(self.start, goal));
        let mut path = VecDeque::with_capacity(self.root.into().distance(goal.into()));
        let gnode = Node::new(goal, *self.costs.get(&goal).unwrap_or(&0));
        path.push_front(gnode);
        let mut current = &goal;
        while current != &self.root {
            if let Some(parent) = self.parents.get(current) {
                let cost = self.costs.get(parent).unwrap_or(&0);
                path.push_front(Node::new(*parent, *cost));
                current = parent;
            } else {
                return None
            }
        }
        Some(path)
    }
}

