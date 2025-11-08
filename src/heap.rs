use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

use crate::action::Action;
use crate::game::Game;

// Structure pour les éléments de la priority queue
#[derive(Eq, PartialEq)]
pub struct HeapNode {
    pub f_score: i32,
    pub counter: u64,
    pub state: Game,
    pub path: Vec<Action>,
}

// we want a min-heap based on f_score
impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Inverse pour avoir un min-heap
        other
            .f_score
            .cmp(&self.f_score)
            .then_with(|| other.counter.cmp(&self.counter))
    }
}

impl PartialOrd for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
