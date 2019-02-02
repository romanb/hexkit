
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;

use crate::grid::Coords;
use crate::grid::coords::{ self, Cube };

use super::{ Context, Tree, Path };

/// A node in the "open" list of the A* algorithm to prioritise the search.
struct Open {
    coords: Cube,
    priority: usize
}

impl PartialEq for Open {
    fn eq(&self, other: &Open) -> bool {
        self.priority == other.priority
    }
}

impl Eq for Open {}

impl PartialOrd for Open {
    fn partial_cmp(&self, other: &Open) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Open {
    fn cmp(&self, other: &Open) -> Ordering {
        // Lower priorities (= estimated total costs)
        // are considered "greater" for the binary heap.
        other.priority.cmp(&self.priority)
    }
}

/// Beginning at the given start coordinates, perform a cost-aware search across
/// the grid, subject to the constraints of the given options, returning the
/// resulting search tree from which paths may be extracted.
///
/// The search stops when any of the following conditions is met:
///
///   * Goal coordinates are given and found.
///   * The `exit` function signals termination.
///   * The grid has been exhaustively searched.
pub fn tree<C: Coords>(
    start: C,
    goal: Option<C>,
    ctx: &mut impl Context<C>
) -> Tree<C> {
    let root         = start.into();
    let max_cost     = ctx.max_cost();
    let max_distance = ctx.max_distance();
    let mut parents  = HashMap::new();
    let mut costs    = HashMap::new();
    let mut open     = BinaryHeap::new();
    open.push(Open { coords: root, priority: 0 });
    costs.insert(start, 0);
    while let Some(parent) = open.pop() {
        let pc = C::from(parent.coords);
        if ctx.exit(pc) || goal.map_or(false, |g| g == pc) {
            break
        }
        // for n in coords::neighbours(c) {
        for child in coords::neighbours(parent.coords) {
            let cc = C::from(child);
            if coords::distance(child, root) > max_distance {
                continue
            }
            let new_cost = if let Some(cost) = ctx.cost(pc, cc) {
                *costs.get(&pc).unwrap_or(&0) + cost
            } else {
                continue
            };
            if new_cost > max_cost {
                continue
            }
            let old_cost = *costs.get(&cc).unwrap_or(&std::usize::MAX);
            if !costs.contains_key(&cc) || new_cost < old_cost {
                parents.insert(cc, pc);
                costs.insert(cc, new_cost);
                let estimate = goal.map_or(0, |g| ctx.heuristic(cc, g));
                let priority = new_cost + estimate;
                open.push(Open { coords: child, priority });
            }
        }
    }
    Tree { root: start, parents, costs }
}

/// Beginning at the given start coordinates, perform a cost-aware search for
/// a path to the given goal coordinates across the grid, subject to the
/// constraints of the given options.
///
/// This is equivalent to:
/// ```raw
/// tree(start, Some(goal), ctx).path(goal)
/// ```
pub fn path<C: Coords>(
    start: C,
    goal: C,
    ctx: &mut impl Context<C>
) -> Option<Path<C>> {
    tree(start, Some(goal), ctx).path(goal)
}

