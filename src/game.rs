use crate::card::Card;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Clone)]
pub struct Game {
    pub columns: [Vec<Card>; 8],
    pub freecells: [Option<Card>; 4],
    pub foundations: [u8; 4],
}

impl Game {
    pub fn new(cards: &[Card]) -> Self {
        let mut game = Game {
            columns: Default::default(),
            freecells: Default::default(),
            foundations: [0; 4],
        };

        for (i, card) in cards.iter().enumerate() {
            let column_index = i % 8;
            game.columns[column_index].push(*card);
        }

        game
    }

    pub fn hash_key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    #[allow(dead_code)]
    pub fn is_won(&self) -> bool {
        self.foundations.iter().all(|&f| f == 13)
    }

    pub fn count_free_cells(&self) -> usize {
        self.freecells.iter().filter(|c| c.is_none()).count()
    }

    pub fn count_empty_columns(&self) -> usize {
        self.columns.iter().filter(|c| c.is_empty()).count()
    }

    #[allow(dead_code)]
    pub fn max_movable_sequence(&self, remove_one_column: bool) -> u32 {
        // The maximum number of cards that can be moved at once is determined by the number of freecells
        // and the number of empty columns.
        let freecells_count = self.count_free_cells();
        let mut free_columns_count = self.count_empty_columns();

        if remove_one_column && free_columns_count > 0 {
            // If we are moving card to an ampty column, we need to adjust the max number of card moved
            free_columns_count -= 1;
        }

        ((1 << free_columns_count) * (freecells_count + 1)).min(13) as u32
    }

    pub fn can_move_to_foundation(&self, card: &Card) -> bool {
        self.foundations[card.suit as usize] + 1 == card.rank
    }

    pub fn can_stack_on(&self, card_below: &Card, card_above: &Card) -> bool {
        // Cards can be stacked if they are of different colors and the rank is one less
        // Call top_card.can_stack(bottom_card) to check if the top card can be placed on the bottom card
        let same_color = card_below.is_black() == card_above.is_black();
        !same_color && card_below.rank + 1 == card_above.rank
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // First row: Freecells and Foundations
        for cell in &self.freecells {
            match cell {
                Some(card) => write!(f, "{:^4?}", card)?,
                None => write!(f, " -- ")?,
            }
        }

        for &count in &self.foundations {
            write!(f, "{:>4}", count)?;
        }
        writeln!(f)?;
        writeln!(f)?;

        // Determine the max column height
        let max_rows = self.columns.iter().map(Vec::len).max().unwrap_or(0);

        // Print columns row by row
        for row in 0..max_rows {
            for col in 0..8 {
                if let Some(card) = self.columns[col].get(row) {
                    write!(f, "{:?}", card)?;
                } else {
                    write!(f, "    ")?; // 4 spaces
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Hash for Game {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // 1. Colonnes : encoder + canonicaliser (trier)
        let mut cols_data: Vec<Vec<u8>> = self
            .columns
            .iter()
            .map(|col| col.iter().map(|c| c.encode()).collect())
            .collect();

        cols_data.sort(); // canonicalisation

        // 2. Free cells : encoder et trier
        let mut free_data: Vec<u8> = self
            .freecells
            .iter()
            .map(|cell| cell.map(|c| c.encode()).unwrap_or(0))
            .collect();

        free_data.sort();

        // 3. On hash tout proprement
        cols_data.hash(state);
        free_data.hash(state);
        self.foundations.hash(state);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // #[test]
    // fn test_max_movable_sequence1() {
    //     let game = Game {
    //         columns: [
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![],
    //         ],
    //         freecells: [None, None, None, None],
    //         foundations: [0; 4],
    //     };

    //     assert_eq!(game.max_movable_sequence(false), 10); // 4 freecell + 1 empty column
    // }

    // #[test]
    // fn test_max_movable_sequence2() {
    //     let game = Game {
    //         columns: [
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![],
    //             vec![],
    //             vec![],
    //             vec![],
    //             vec![],
    //         ],
    //         freecells: [Some(Card::from("1S")), None, None, None],
    //         foundations: [0; 4],
    //     };

    //     assert_eq!(game.max_movable_sequence(false), 13);
    // }

    // #[test]
    // fn test_max_movable_sequence3() {
    //     let game = Game {
    //         columns: [
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //         ],
    //         freecells: [
    //             Some(Card::from("1S")),
    //             Some(Card::from("1S")),
    //             Some(Card::from("1S")),
    //             None,
    //         ],
    //         foundations: [0; 4],
    //     };

    //     assert_eq!(game.max_movable_sequence(false), 2); // 4 freecell + 1 empty column
    // }

    // #[test]
    // fn test_max_movable_sequence4() {
    //     let game = Game {
    //         columns: [
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //             vec![Card::from("1S")],
    //         ],
    //         freecells: [
    //             Some(Card::from("1S")),
    //             Some(Card::from("1S")),
    //             Some(Card::from("1S")),
    //             Some(Card::from("1S")),
    //         ],
    //         foundations: [0; 4],
    //     };

    //     assert_eq!(game.max_movable_sequence(false), 1); // only 1 move
    // }

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
}
