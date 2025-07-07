mod game;
mod ocr;
mod screen;

use crate::game::{Card, Game, Suit};
use dotenv::dotenv;
use rand::seq::SliceRandom;

#[allow(dead_code)]
fn generate_random_deck() -> Vec<Card> {
    let mut deck: Vec<Card> = (0..52)
        .map(|i| Card {
            rank: ((i % 13) + 1) as u8,
            suit: match i / 13 {
                0 => Suit::Diamond,
                1 => Suit::Club,
                2 => Suit::Spade,
                _ => Suit::Heart,
            },
        })
        .collect();

    let mut rng = rand::rng();
    deck.shuffle(&mut rng);
    deck
}

fn main() {
    dotenv().ok();

    // eprintln!("🃏 Génération d'un jeu de cartes basé sur un screenshot...");
    // let screenshot = screen::start_screenshot();
    // let cards = ocr::run_ocr();
    // eprintln!("{:?}", cards.iter().map(|p| p.card).collect::<Vec<_>>());

    eprintln!("🃏 Génération d'un jeu de cartes aléatoire...");
    let deck = generate_random_deck();
    let game = Game::new(&deck);
    println!("{:?}", game);

    for c1 in 0..8 {
        for c2 in 0..8 {
            if let Some(offset) = game.has_move(c1, c2) {
                eprintln!(
                    "🃏 Déplacement possible de la colonne {} vers la colonne {} avec un offset de {}",
                    c1, c2, offset
                );
            } else {
                eprintln!(
                    "🃏 Aucun déplacement possible de la colonne {} vers la colonne {}",
                    c1, c2
                );
            }
        }
    }
}
