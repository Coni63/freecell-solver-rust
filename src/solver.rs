use crate::action::{Action, ActionType};
use crate::card::{Card, Suit};
use crate::game::Game;
use crate::heap::HeapNode;
use std::collections::{BinaryHeap, HashSet};
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};

pub struct Solver {
    pub initial_game: Game,
    pub visited_states: std::collections::HashSet<u64>,
    pub nodes_explored: u64,
}

impl Solver {
    pub fn new(game: Game) -> Self {
        Solver {
            initial_game: game,
            visited_states: std::collections::HashSet::new(),
            nodes_explored: 0,
        }
    }

    pub fn heuristic(&self, game: &Game) -> i32 {
        let mut score: i32 = 0;

        // Cartes pas encore en fondation (poids principal)
        let cards_remaining = 52 - game.foundations.iter().map(|&f| f as i32).sum::<i32>();
        score += cards_remaining * 10;

        // Bonus de sequences bien ordonnées dans les colonnes
        for col in &game.columns {
            for window in col.windows(2) {
                if game.can_stack_on(&window[0], &window[1]) {
                    score -= 3;
                }
            }
        }

        // Pénalité pour cellules libres occupées
        score += (4 - game.count_free_cells() as i32) * 5;

        // Pénalité pour les cartes bloquees
        for col in &game.columns {
            for window in col.windows(2) {
                if &window[0].rank < &window[1].rank {
                    score += 5;
                }
            }
        }

        score
    }

    pub fn get_moves(&self, game: &Game) -> Vec<Action> {
        let mut all_moves = vec![];

        for (i, col) in game.columns.iter().enumerate() {
            if col.is_empty() {
                continue;
            }

            // Move to foundations
            let top_card = col.last().unwrap();
            if game.can_move_to_foundation(top_card) {
                all_moves.push(Action {
                    action_type: ActionType::ColToFoundation,
                    source: i,
                    dest: top_card.suit as usize,
                    pile_size: 1,
                });
            }
        }

        // Freecell to foundations
        for (fc_index, freecell) in game.freecells.iter().enumerate() {
            if let Some(card) = freecell {
                if game.can_move_to_foundation(card) {
                    all_moves.push(Action {
                        action_type: ActionType::FreecellToFoundation,
                        source: fc_index,
                        dest: card.suit as usize,
                        pile_size: 1,
                    });
                }
            }
        }

        for (i, source_col) in game.columns.iter().enumerate() {
            if source_col.is_empty() {
                continue;
            }

            // Calculer la longueur de la séquence déplaçable
            let mut seq_len = 1;
            for window in source_col.windows(2).rev() {
                if game.can_stack_on(&window[0], &window[1]) {
                    seq_len += 1;
                } else {
                    break;
                }
            }

            // Move between columns
            for (j, target_col) in game.columns.iter().enumerate() {
                if i == j {
                    continue;
                }

                if seq_len == target_col.len() && target_col.is_empty() {
                    continue; // Skip moving full sequence to empty column
                }

                for pile_size in 1..seq_len {
                    if target_col.is_empty() {
                        // Can move any sequence to empty column
                        all_moves.push(Action {
                            action_type: ActionType::ColToCol,
                            source: i,
                            dest: j,
                            pile_size,
                        });
                    } else {
                        let target_top_card = target_col.last().unwrap();
                        let moving_card = &source_col[source_col.len() - pile_size];
                        if game.can_stack_on(target_top_card, moving_card) {
                            all_moves.push(Action {
                                action_type: ActionType::ColToCol,
                                source: i,
                                dest: j,
                                pile_size,
                            });
                        }
                    }
                }
            }

            // Move to freecells
            for freecell_index in 0..4 {
                if game.freecells[freecell_index].is_none() {
                    all_moves.push(Action {
                        action_type: ActionType::ColToFreecell,
                        source: i,
                        dest: freecell_index,
                        pile_size: 1,
                    });
                    break; // Only need one freecell move
                }
            }

            // Move from freecells to columns
            for (fc_index, freecell) in game.freecells.iter().enumerate() {
                if let Some(card) = freecell {
                    if source_col.is_empty() {
                        all_moves.push(Action {
                            action_type: ActionType::FreecellToCol,
                            source: fc_index,
                            dest: i,
                            pile_size: 1,
                        });
                    } else {
                        let target_top_card = source_col.last().unwrap();
                        if game.can_stack_on(target_top_card, card) {
                            all_moves.push(Action {
                                action_type: ActionType::FreecellToCol,
                                source: fc_index,
                                dest: i,
                                pile_size: 1,
                            });
                        }
                    }
                }
            }
        }

        all_moves
    }

    pub fn apply_move(&self, game: &Game, action: &Action) -> Game {
        let mut copy = game.clone();

        match action.action_type {
            ActionType::ColToFoundation => {
                let card = copy.columns[action.source].pop().unwrap();
                copy.foundations[card.suit as usize] += 1;
            }
            ActionType::FreecellToFoundation => {
                let card = copy.freecells[action.source].take().unwrap();
                copy.foundations[card.suit as usize] += 1;
            }
            ActionType::ColToFreecell => {
                let card = copy.columns[action.source].pop().unwrap();
                copy.freecells[action.dest] = Some(card);
            }
            ActionType::FreecellToCol => {
                let card = copy.freecells[action.source].take().unwrap();
                copy.columns[action.dest].push(card);
            }
            ActionType::ColToCol => {
                let moving_cards: Vec<Card> = copy.columns[action.source]
                    .drain(copy.columns[action.source].len() - action.pile_size..)
                    .collect();
                copy.columns[action.dest].extend(moving_cards);
            }
        }

        copy
    }

    pub fn solve(&self, max_nodes: u32) -> Option<Vec<Action>> {
        let start_h = self.heuristic(&self.initial_game);

        let mut counter = 0;

        let mut heap = BinaryHeap::new();

        heap.push(HeapNode {
            f_score: start_h,
            counter,
            state: self.initial_game.clone(),
            path: Vec::new(),
        });

        let mut visited = HashSet::new();
        visited.insert(self.initial_game.hash_key());
        let mut nodes_explored = 0;

        while let Some(node) = heap.pop() {
            if nodes_explored >= max_nodes {
                break;
            }

            let g_score = node.path.len() as i32;
            nodes_explored += 1;

            if nodes_explored % 1000 == 0 {
                println!(
                    "Explored: {}, Queue: {}, Path: {}, H: {:.1}",
                    nodes_explored,
                    heap.len(),
                    node.path.len(),
                    node.f_score - g_score
                );
            }

            if node.state.is_won() {
                println!("\n✓ Solution trouvée en {} coups!", node.path.len());
                println!("Nœuds explorés: {}", nodes_explored);
                return Some(node.path);
            }

            // Générer les mouvements
            for mov in self.get_moves(&node.state) {
                let new_state = self.apply_move(&node.state, &mov);
                let state_hash = new_state.hash_key();

                if !visited.contains(&state_hash) {
                    visited.insert(state_hash);
                    let new_g = g_score + 1;
                    let new_h = self.heuristic(&new_state);
                    let new_f = new_g + new_h;

                    counter += 1;
                    let mut new_path = node.path.clone();
                    new_path.push(mov);

                    heap.push(HeapNode {
                        f_score: new_f,
                        counter,
                        state: new_state,
                        path: new_path,
                    });
                }
            }
        }

        println!("\n✗ Pas de solution trouvée après {} nœuds", nodes_explored);
        None
    }
}
