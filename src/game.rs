use std::fmt::Debug;

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Diamond,
    Club,
    Spade,
    Heart,
}

#[derive(Clone, PartialEq, Eq, Copy)]
pub struct Card {
    pub rank: u8,
    pub suit: Suit,
}

impl Card {
    pub fn can_stack(&self, other: &Card) -> bool {
        // Cards can be stacked if they are of different colors and the rank is one less
        // Call top_card.can_stack(bottom_card) to check if the top card can be placed on the bottom card
        let same_color = (self.suit == Suit::Diamond || self.suit == Suit::Heart)
            == (other.suit == Suit::Diamond || other.suit == Suit::Heart);
        !same_color && self.rank - 1 == other.rank
    }
}

impl Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            match self.rank {
                1 => "A".to_string(),
                11 => "J".to_string(),
                12 => "Q".to_string(),
                13 => "K".to_string(),
                _ => self.rank.to_string(),
            },
            match self.suit {
                Suit::Diamond => "♦",
                Suit::Club => "♣",
                Suit::Spade => "♠",
                Suit::Heart => "♥",
            }
        )
    }
}

impl From<&str> for Card {
    fn from(txt: &str) -> Self {
        let (r, s) = txt.split_at(txt.len() - 1);
        let rank = r.parse::<u8>().unwrap();
        let suit = match s.chars().next() {
            Some('D') => Suit::Diamond,
            Some('C') => Suit::Club,
            Some('S') => Suit::Spade,
            Some('H') => Suit::Heart,
            _ => panic!("Invalid suit character: {}", s),
        };

        Card { rank, suit }
    }
}

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

    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        self.foundations.iter().all(|&f| f == 13)
    }

    #[allow(dead_code)]
    fn max_card_move(&self) -> usize {
        // The maximum number of cards that can be moved at once is determined by the number of freecells
        // and the number of empty columns.
        let freecells_count = self.freecells.iter().filter(|&&c| c.is_none()).count();
        let free_columns_count = self.columns.iter().filter(|c| c.is_empty()).count();
        eprintln!("{} {}", freecells_count, free_columns_count);
        ((1 << free_columns_count) * (freecells_count + 1)).min(13)
    }

    fn max_sequence(&self, column: usize) -> usize {
        if self.columns[column].len() < 2 {
            return self.columns[column].len();
        }

        for i in (0..self.columns[column].len() - 1).rev() {
            let a = self.columns[column][i + 1];
            let b = self.columns[column][i];

            eprintln!("Checking if {:?} can stack on {:?}", a, b);

            if !b.can_stack(&a) {
                return i;
            }

            eprintln!("OK");
        }

        self.columns[column].len() - 1
    }

    #[allow(dead_code)]
    pub fn has_move(&self, from: usize, to: usize) -> Option<usize> {
        if from >= 8 || to >= 8 {
            return None; // Invalid column indices
        }

        if from == to {
            return None; // Cannot move to the same column
        }

        if self.columns[from].is_empty() {
            return None; // Cannot move from an empty column
        }

        let target_card = self.columns[to].last().unwrap();
        let max_moves = self.max_card_move().min(self.columns[from].len());

        for i in 1..=max_moves {
            let offset = self.columns[from].len() - i;
            let card_to_move = self.columns[from][offset];

            if i > 1 {
                // Check if the stack is sorted correctly
                let previous_card = self.columns[from][offset + 1];
                if !card_to_move.can_stack(&previous_card) {
                    return None;
                }
            }

            if target_card.can_stack(&card_to_move) {
                return Some(offset);
            }
        }

        None
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Game")
            .field("columns", &self.columns)
            .field("freecells", &self.freecells)
            .field("foundations", &self.foundations)
            .finish()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_card_can_stack() {
        let card1 = Card {
            rank: 5,
            suit: Suit::Diamond,
        };
        let card2 = Card {
            rank: 4,
            suit: Suit::Club,
        };
        let card3 = Card {
            rank: 4,
            suit: Suit::Heart,
        };
        let card4 = Card {
            rank: 3,
            suit: Suit::Spade,
        };
        let card5 = Card {
            rank: 8,
            suit: Suit::Heart,
        };

        assert!(card1.can_stack(&card2));
        assert!(!card1.can_stack(&card3));
        assert!(!card1.can_stack(&card4));
        assert!(!card1.can_stack(&card5));
    }

    #[test]
    fn test_max_card_move1() {
        let game = Game {
            columns: [
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![],
            ],
            freecells: [None, None, None, None],
            foundations: [0; 4],
        };

        assert_eq!(game.max_card_move(), 10); // 4 freecell + 1 empty column
    }

    #[test]
    fn test_max_card_move2() {
        let game = Game {
            columns: [
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ],
            freecells: [Some(Card::from("1S")), None, None, None],
            foundations: [0; 4],
        };

        assert_eq!(game.max_card_move(), 13);
    }

    #[test]
    fn test_max_card_move3() {
        let game = Game {
            columns: [
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
            ],
            freecells: [
                Some(Card::from("1S")),
                Some(Card::from("1S")),
                Some(Card::from("1S")),
                None,
            ],
            foundations: [0; 4],
        };

        assert_eq!(game.max_card_move(), 2); // 4 freecell + 1 empty column
    }

    #[test]
    fn test_max_card_move4() {
        let game = Game {
            columns: [
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
                vec![Card::from("1S")],
            ],
            freecells: [
                Some(Card::from("1S")),
                Some(Card::from("1S")),
                Some(Card::from("1S")),
                Some(Card::from("1S")),
            ],
            foundations: [0; 4],
        };

        assert_eq!(game.max_card_move(), 1); // only 1 move
    }

    #[test]
    fn test_max_sequence() {
        let game = Game {
            columns: [
                vec![Card::from("3C"), Card::from("2H"), Card::from("1S")],
                vec![Card::from("4D"), Card::from("5S")],
                vec![Card::from("6H")],
                vec![
                    Card::from("8D"),
                    Card::from("3C"),
                    Card::from("2H"),
                    Card::from("1S"),
                ],
                vec![],
                vec![],
                vec![],
                vec![],
            ],
            freecells: [None, None, None, None],
            foundations: [0; 4],
        };

        assert_eq!(game.max_sequence(0), 3);
        // assert_eq!(game.max_sequence(1), 1);
        // assert_eq!(game.max_sequence(2), 1);
        // assert_eq!(game.max_sequence(3), 3);
        assert_eq!(game.max_sequence(4), 0);
    }
}
