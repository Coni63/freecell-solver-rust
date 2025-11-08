use crate::action::{Action, ActionType};
use crate::card::{Card, Suit};
use crate::game::Game;
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

    pub fn heuristic(&self, game: &Game) -> f32 {
        let mut score = 0.0f32;

        // Cartes pas encore en fondation (poids principal)
        let cards_remaining: u32 = 52 - game.foundations.iter().map(|&f| f as u32).sum::<u32>();
        score += cards_remaining as f32;

        // Bonus de sequences bien ordonnées dans les colonnes
        for col in &game.columns {
            for window in col.windows(2) {
                if game.can_stack_on(&window[0], &window[1]) {
                    score -= 0.3;
                }
            }
        }

        // Pénalité pour cellules libres occupées
        score += (4 - game.count_free_cells()) as f32 * 0.5;

        // Pénalité pour les cartes bloquees
        for col in &game.columns {
            for window in col.windows(2) {
                if &window[0].rank < &window[1].rank {
                    score += 0.5;
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

    pub fn apply_move(&self, game: &mut Game, action: &Action) -> Game {
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
        // Placeholder for the actual solving logic
        None
    }
}

// #[derive(Clone)]
// pub struct Solver {
//     pub columns: [Vec<Card>; 8],
//     pub freecells: [Option<Card>; 4],
//     pub foundations: [u8; 4],
// }

// impl Game {
//     pub fn new(cards: &[Card]) -> Self {
//         let mut game = Game {
//             columns: Default::default(),
//             freecells: Default::default(),
//             foundations: [0; 4],
//         };

//         for (i, card) in cards.iter().enumerate() {
//             let column_index = i % 8;
//             game.columns[column_index].push(*card);
//         }

//         game.apply_foundation_moves();

//         game
//     }

//     pub fn hash_key(&self) -> u64 {
//         let mut hasher = DefaultHasher::new();
//         self.hash(&mut hasher);
//         hasher.finish()
//     }

//     #[allow(dead_code)]
//     pub fn is_won(&self) -> bool {
//         self.foundations.iter().all(|&f| f == 13)
//     }

//     pub fn count_free_cells(&self) -> usize {
//         self.freecells.iter().filter(|c| c.is_none()).count()
//     }

//     pub fn count_empty_columns(&self) -> usize {
//         self.columns.iter().filter(|c| c.is_empty()).count()
//     }

//     pub fn can_stack_on(&self, card_below: &Card, card_above: &Card) -> bool {
//         // Cards can be stacked if they are of different colors and the rank is one less
//         // Call top_card.can_stack(bottom_card) to check if the top card can be placed on the bottom card
//         let same_color = card_below.is_black() == card_above.is_black();
//         !same_color && card_below.rank + 1 == card_above.rank
//     }

//     #[allow(dead_code)]
//     pub fn max_movable_sequence(&self, remove_one_column: bool) -> usize {
//         // The maximum number of cards that can be moved at once is determined by the number of freecells
//         // and the number of empty columns.
//         let freecells_count = self.count_free_cells();
//         let mut free_columns_count = self.count_empty_columns();

//         if remove_one_column && free_columns_count > 0 {
//             // If we are moving card to an ampty column, we need to adjust the max number of card moved
//             free_columns_count -= 1;
//         }

//         ((1 << free_columns_count) * (freecells_count + 1)).min(13)
//     }

//     pub fn can_move_to_foundation(&self, card: &Card) -> bool {
//         self.foundations[card.suit as usize] + 1 == card.rank
//     }

//     pub fn apply(&mut self, action: &Action) -> Result<(), String> {
//         if action.pile_size >= self.columns[action.from].len() {
//             return Err("Invalid offset".to_string());
//         }

//         if action.destock() {
//             self.move_from_freecell(action).unwrap();
//         } else if action.stock() {
//             self.move_to_freecell(action).unwrap();
//         } else {
//             self.move_between_columns(action).unwrap();
//         }

//         self.apply_foundation_moves();

//         Ok(())
//     }

//     fn move_between_columns(&mut self, action: &Action) -> Result<(), String> {
//         let cards_to_move = self.columns[action.from][action.pile_size..].to_vec();
//         self.columns[action.from].truncate(action.pile_size);
//         // Moving between columns
//         if let Some(target_card) = self.columns[action.to].last() {
//             if !cards_to_move.iter().all(|card| target_card.can_stack(card)) {
//                 return Err("Cannot stack cards on the target card".to_string());
//             }
//         }
//         self.columns[action.to].extend(cards_to_move);
//         Ok(())
//     }

//     fn move_to_freecell(&mut self, action: &Action) -> Result<(), String> {
//         let freecell_index = action.to - 8;
//         if self.freecells[freecell_index].is_some() {
//             return Err("Freecell is already occupied".to_string());
//         }
//         self.freecells[freecell_index] = self.columns[action.from].pop();
//         Ok(())
//     }

//     fn move_from_freecell(&mut self, action: &Action) -> Result<(), String> {
//         let freecell_index = action.to - 8;
//         if let Some(card) = self.freecells[freecell_index].take() {
//             // If there is a card in the target column, check if it can be stacked
//             if let Some(target_card) = self.columns[action.to].last() {
//                 if !target_card.can_stack(&card) {
//                     return Err("Cannot stack card on the target column".to_string());
//                 }
//             }
//             self.columns[action.to].push(card);
//         }
//         Ok(())
//     }

//     #[allow(dead_code)]
//     fn max_sequence(&self, column: usize) -> usize {
//         let column_size = self.columns[column].len();

//         if column_size < 2 {
//             return column_size;
//         }

//         for i in (0..column_size - 1).rev() {
//             let bottom_card = self.columns[column][i + 1];
//             let top_card = self.columns[column][i];
//             if !top_card.can_stack(&bottom_card) {
//                 return column_size - i - 1;
//             }
//         }

//         column_size
//     }

//     #[allow(dead_code)]
//     fn get_all_possible_moves_between_columns(&self, from: usize, to: usize) -> Vec<Action> {
//         let mut ans = Vec::new();

//         if self.columns[from].is_empty() {
//             return ans; // Cannot move from an empty column
//         }

//         match self.columns[to].last() {
//             Some(target_card) => {
//                 let max_moves = self.max_movable_sequence(false);
//                 let source_column_sequence = self.max_sequence(from);
//                 let max_moves = max_moves.min(source_column_sequence);
//                 for i in 1..=max_moves {
//                     let offset = self.columns[from].len() - i;
//                     let card_to_move = self.columns[from][offset];

//                     if target_card.can_stack(&card_to_move) {
//                         ans.push(Action::new(from, to, offset).unwrap());
//                     }
//                 }
//             }
//             None => {
//                 let max_moves = self.max_movable_sequence(true);
//                 let source_column_sequence = self.max_sequence(from);
//                 let max_moves = max_moves.min(source_column_sequence);
//                 for i in 1..=max_moves {
//                     let offset = self.columns[from].len() - i;
//                     ans.push(Action::new(from, to, offset).unwrap());
//                 }
//             }
//         };

//         ans
//     }

//     pub fn get_all_possible_moves(&self) -> Vec<Action> {
//         let mut ans = Vec::new();

//         for from in 0..8 {
//             for to in 0..8 {
//                 ans.extend(self.get_all_possible_moves_between_columns(from, to));
//             }

//             // Check if we can move to freecells
//             for freecell_index in 0..4 {
//                 if self.freecells[freecell_index].is_none() {
//                     // If the freecell is empty, we can move any card from the column to it
//                     if self.columns[from].last().is_some() {
//                         if let Ok(action) = Action::new(from, 8 + freecell_index, 1) {
//                             ans.push(action);
//                         }
//                         break; // it makes no sense to check other freecells
//                     }
//                 }
//             }
//         }

//         // Check if we can move from freecells to columns
//         for freecell_index in 0..4 {
//             if let Some(card) = self.freecells[freecell_index] {
//                 for to in 0..8 {
//                     if let Some(target_card) = self.columns[to].last() {
//                         if target_card.can_stack(&card) {
//                             if let Ok(action) = Action::new(8 + freecell_index, to, 1) {
//                                 ans.push(action);
//                             }
//                         }
//                     } else {
//                         // If the column is empty, we can move the card to it
//                         if let Ok(action) = Action::new(8 + freecell_index, to, 1) {
//                             ans.push(action);
//                         }
//                     }
//                 }
//             }
//         }

//         ans
//     }

//     fn apply_foundation_moves(&mut self) {
//         loop {
//             let mut has_move = false;
//             for col in 0..8 {
//                 if self.columns[col].is_empty() {
//                     continue; // Skip empty columns
//                 }

//                 let card = self.columns[col].last().unwrap();
//                 if self.can_move_to_foundation(card) {
//                     // Move the card to the foundation
//                     self.foundations[card.suit] += 1;
//                     self.columns[col].pop();
//                     has_move = true;
//                     break; // Exit the loop to re-evaluate the game state
//                 }
//             }

//             if !has_move {
//                 break;
//             }
//         }
//     }
// }

// impl Debug for Game {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         // First row: Freecells and Foundations
//         for cell in &self.freecells {
//             match cell {
//                 Some(card) => write!(f, "{:^4?}", card)?,
//                 None => write!(f, " -- ")?,
//             }
//         }

//         for &count in &self.foundations {
//             write!(f, "{:>4}", count)?;
//         }
//         writeln!(f)?;
//         writeln!(f)?;

//         // Determine the max column height
//         let max_rows = self.columns.iter().map(Vec::len).max().unwrap_or(0);

//         // Print columns row by row
//         for row in 0..max_rows {
//             for col in 0..8 {
//                 if let Some(card) = self.columns[col].get(row) {
//                     write!(f, "{:?}", card)?;
//                 } else {
//                     write!(f, "    ")?; // 4 spaces
//                 }
//             }
//             writeln!(f)?;
//         }

//         Ok(())
//     }
// }

// impl Hash for Game {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         // 1. Colonnes : encoder + canonicaliser (trier)
//         let mut cols_data: Vec<Vec<u8>> = self
//             .columns
//             .iter()
//             .map(|col| col.iter().map(|c| c.encode()).collect())
//             .collect();

//         cols_data.sort(); // canonicalisation

//         // 2. Free cells : encoder et trier
//         let mut free_data: Vec<u8> = self
//             .freecells
//             .iter()
//             .map(|cell| cell.map(|c| c.encode()).unwrap_or(0))
//             .collect();

//         free_data.sort();

//         // 3. On hash tout proprement
//         cols_data.hash(state);
//         free_data.hash(state);
//         self.foundations.hash(state);
//     }
// }

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn test_max_movable_sequence1() {
//         let game = Game {
//             columns: [
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![],
//             ],
//             freecells: [None, None, None, None],
//             foundations: [0; 4],
//         };

//         assert_eq!(game.max_movable_sequence(false), 10); // 4 freecell + 1 empty column
//     }

//     #[test]
//     fn test_max_movable_sequence2() {
//         let game = Game {
//             columns: [
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![],
//                 vec![],
//                 vec![],
//                 vec![],
//                 vec![],
//             ],
//             freecells: [Some(Card::from("1S")), None, None, None],
//             foundations: [0; 4],
//         };

//         assert_eq!(game.max_movable_sequence(false), 13);
//     }

//     #[test]
//     fn test_max_movable_sequence3() {
//         let game = Game {
//             columns: [
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//             ],
//             freecells: [
//                 Some(Card::from("1S")),
//                 Some(Card::from("1S")),
//                 Some(Card::from("1S")),
//                 None,
//             ],
//             foundations: [0; 4],
//         };

//         assert_eq!(game.max_movable_sequence(false), 2); // 4 freecell + 1 empty column
//     }

//     #[test]
//     fn test_max_movable_sequence4() {
//         let game = Game {
//             columns: [
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//                 vec![Card::from("1S")],
//             ],
//             freecells: [
//                 Some(Card::from("1S")),
//                 Some(Card::from("1S")),
//                 Some(Card::from("1S")),
//                 Some(Card::from("1S")),
//             ],
//             foundations: [0; 4],
//         };

//         assert_eq!(game.max_movable_sequence(false), 1); // only 1 move
//     }

//     #[test]
//     fn test_max_sequence() {
//         let game = Game {
//             columns: [
//                 vec![Card::from("3C"), Card::from("2H"), Card::from("1S")],
//                 vec![Card::from("4D"), Card::from("5S")],
//                 vec![Card::from("6H")],
//                 vec![
//                     Card::from("8D"),
//                     Card::from("3C"),
//                     Card::from("2H"),
//                     Card::from("1S"),
//                 ],
//                 vec![Card::from("5S"), Card::from("4D")],
//                 vec![],
//                 vec![],
//                 vec![],
//             ],
//             freecells: [None, None, None, None],
//             foundations: [0; 4],
//         };

//         assert_eq!(game.max_sequence(0), 3);
//         assert_eq!(game.max_sequence(1), 1);
//         assert_eq!(game.max_sequence(2), 1);
//         assert_eq!(game.max_sequence(3), 3);
//         assert_eq!(game.max_sequence(4), 2);
//         assert_eq!(game.max_sequence(5), 0);
//     }
// }
