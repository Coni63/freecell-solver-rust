mod game;
mod ocr;
mod screen;
use crate::game::{Card, Game, Suit};
use dotenv::dotenv;
use rand::seq::SliceRandom;
use std::time::Instant;

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

    // let deck = if dotenv::var("USE_RANDOM").unwrap_or("0".to_string()) == "1" {
    //     eprintln!("🃏 Génération d'un jeu de cartes aléatoire...");
    //     generate_random_deck()
    // } else {
    //     eprintln!("🃏 Génération d'un jeu de cartes basé sur un screenshot...");
    //     let _screenshot = screen::start_screenshot();
    //     let cards = ocr::run_ocr();
    //     cards.iter().map(|p| p.card).collect::<Vec<_>>()
    // };

    let deck = generate_random_deck();

    let game = Game::new(&deck);
    println!("{:?}", game);

    let now = Instant::now();

    let mut all_games: Vec<Game> = vec![];
    let actions = game.get_all_possible_moves();
    for action in actions.iter() {
        let mut gc = game.clone();
        if gc.apply(action).is_ok() {
            all_games.push(gc);
        }
    }
    eprintln!("Found {} possible moves", all_games.len());

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}
