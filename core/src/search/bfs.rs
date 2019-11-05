
use std::collections::HashMap;
use std::collections::VecDeque;
use super::{ Context, Tree, Path };

use crate::grid::Coords;
use crate::grid::coords;

/// Beginning at the given start coordinates, perform a breadth-first-search
/// across the grid, subject to the constraints of the given options, returning
/// the resulting search tree from which paths may be extracted.
///
/// The search stops when any of the following conditions is met:
///
///   * Goal coordinates are given and found.
///   * The `exit` function signals termination.
///   * The grid has been exhaustively searched.
///
/// A BFS does not track costs, but any coordinates for which the cost function
/// returns a value greater than `max_cost` are considered impassable.
pub fn tree<C: Coords>(
    start: C,
    goal: Option<C>,
    ctx: &mut impl Context<C>
) -> Tree<C> {
    let max_cost     = ctx.max_cost();
    let max_distance = ctx.max_distance();
    let mut parents  = HashMap::new();
    let mut front    = VecDeque::new();
    front.push_back((start.into(), 0));
    while let Some((c,d)) = front.pop_front() {
        let cc = C::from(c);
        if ctx.exit(cc) || goal.map_or(false, |g| g == cc) {
            break
        }
        for n in coords::neighbours(c) {
            let nc = C::from(n);
            if d < max_distance
                && !parents.contains_key(&nc)
                && ctx.cost(cc, nc).map_or(false, |cost| cost <= max_cost)
            {
                parents.insert(nc, cc);
                front.push_back((n, d+1));
            }
        }
    }
    Tree { root: start, parents, costs: HashMap::new() }
}

/// Beginning at the given start coordinates, perform a breadth-first search for
/// a path to the given goal coordinates across the grid, subject to the
/// constraints of the context.
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

