use std::fmt::Debug;

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
pub enum Suit {
    Diamond = 0,
    Club = 1,
    Spade = 2,
    Heart = 3,
}

#[derive(Clone, PartialEq, Eq, Copy)]
pub struct Card {
    pub rank: u8,
    pub suit: Suit,
}

impl Card {
    pub fn is_black(&self) -> bool {
        self.suit == Suit::Diamond || self.suit == Suit::Heart
    }

    #[allow(dead_code)]
    pub fn encode(&self) -> u8 {
        ((self.suit as u8) << 4) as u8 + self.rank
    }

    #[allow(dead_code)]
    pub fn decode(value: u8) -> Self {
        let rank = value & 0xF;
        let suit = match value >> 4 {
            0 => Suit::Diamond,
            2 => Suit::Spade,
            1 => Suit::Club,
            3 => Suit::Heart,
            _ => Suit::Heart, // will never be used
        };
        Card { rank, suit }
    }
}

impl Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:>3}{}",
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

#[cfg(test)]
mod tests {

    use super::*;

    // #[test]
    // fn test_card_can_stack() {
    //     let card1 = Card {
    //         rank: 5,
    //         suit: Suit::Diamond,
    //     };
    //     let card2 = Card {
    //         rank: 4,
    //         suit: Suit::Club,
    //     };
    //     let card3 = Card {
    //         rank: 4,
    //         suit: Suit::Heart,
    //     };
    //     let card4 = Card {
    //         rank: 3,
    //         suit: Suit::Spade,
    //     };
    //     let card5 = Card {
    //         rank: 8,
    //         suit: Suit::Heart,
    //     };

    //     assert!(card1.can_stack(&card2));
    //     assert!(!card1.can_stack(&card3));
    //     assert!(!card1.can_stack(&card4));
    //     assert!(!card1.can_stack(&card5));
    // }
}
