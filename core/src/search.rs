
pub mod astar;
pub mod bfs;

use crate::grid::coords::{ self, Coords };

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
        coords::distance(from, to)
    }
    fn cost(&mut self, from: C, to: C) -> Option<usize>;
}

/// A node in a path of a search tree.
#[derive(Debug, Clone)]
pub struct Node<C> {
    pub coords: C,
    pub cost: usize,
}

impl<C: Coords> PartialEq for Node<C> {
    fn eq(&self, other: &Node<C>) -> bool {
        self.coords == other.coords
    }
}

impl<C: Coords> Eq for Node<C> {}

impl<C> Node<C> {
    fn new(coords: C, cost: usize) -> Node<C> {
        Node { coords, cost }
    }
}

impl<C> std::borrow::Borrow<C> for Node<C> {
    fn borrow(&self) -> &C {
        &self.coords
    }
}

pub struct Path<C>(VecDeque<Node<C>>);

impl<C> std::ops::Deref for Path<C> {
    type Target = VecDeque<Node<C>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C> Path<C> {
    pub fn empty() -> Path<C> {
        Path(VecDeque::new())
    }

    pub fn to_vec(self) -> Vec<Node<C>> {
        Vec::from(self.0)
    }
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

impl<C: Coords> Tree<C> {
    pub fn root(&self) -> Node<C> {
        Node::new(self.root, 0)
    }

    /// Get the total cost of the path from the root node to the given
    /// coordinates, if it exists.
    pub fn cost(&self, coords: C) -> Option<usize> {
        self.costs.get(&coords).map(|c| *c)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&C, &usize)> {
        self.costs.iter()
    }

    /// Trace a path from the given goal back to the root of the tree. The path
    /// is returned in the natural (i.e. reverse) order from start to goal.
    pub fn path(&self, goal: C) -> Option<Path<C>> {
        let mut path = VecDeque::with_capacity(coords::distance(self.root, goal));
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
        Some(Path(path))
    }
}

