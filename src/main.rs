mod action;
mod card;
mod game;
mod heap;
mod ocr;
mod screen;
mod solver;
use crate::card::{Card, Suit};
use crate::game::Game;
use crate::solver::Solver;
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

    let solver = Solver::new(game);
    let actions = solver.solve(1000000);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    if let Some(solution) = actions {
        eprintln!("✅ Solution trouvée en {} mouvements:", solution.len());
        for action in solution {
            eprintln!("  - {:?}", action);
        }
    } else {
        eprintln!("❌ Aucune solution trouvée dans la limite de mouvements.");
    }
}
