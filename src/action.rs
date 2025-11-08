use crate::card::{Card, Suit};
use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionType {
    ColToFoundation,
    FreecellToFoundation,
    ColToFreecell,
    FreecellToCol,
    ColToCol,
}

#[derive(Clone, Debug)]
pub struct Action {
    pub action_type: ActionType,
    pub source: usize,
    pub dest: usize,
    pub pile_size: usize,
}
