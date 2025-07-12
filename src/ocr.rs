use ::core::panic;

use glob::glob;
use opencv::{
    core::{self, Mat, Point},
    imgcodecs, imgproc,
    prelude::*,
};

use crate::game::Card;

#[derive(Debug, Clone)]
pub struct CardPosition {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub confidence: f64,
    pub card: Card,
}

pub fn run_ocr() -> Vec<CardPosition> {
    let mut card_positions: Vec<CardPosition> = Vec::new();

    // Load images
    let img_scene = imgcodecs::imread("capture.png", imgcodecs::IMREAD_COLOR)
        .expect("Error while loading capture.png");

    // Check if images loaded successfully
    if img_scene.empty() {
        panic!("Could not load the scene image");
    }

    for path in glob("templates/*.png")
        .expect("Failed to read glob pattern")
        .flatten()
    {
        let img_query = imgcodecs::imread(path.to_str().unwrap(), imgcodecs::IMREAD_COLOR)
            .unwrap_or_else(|_| panic!("Error while loading {:?}", path));

        if img_query.empty() {
            panic!("Could not load the query image: {:?}", path);
        }

        // Perform template matching
        let mut result = Mat::default();
        imgproc::match_template(
            &img_scene,
            &img_query,
            &mut result,
            imgproc::TM_CCOEFF_NORMED,
            &Mat::default(),
        )
        .unwrap_or_else(|_| panic!("Template matching failed for {:?}", path));

        // Find the best match location
        let mut min_val = 0.0;
        let mut max_val = 0.0;
        let mut min_loc = Point::default();
        let mut max_loc = Point::default();

        core::min_max_loc(
            &result,
            Some(&mut min_val),
            Some(&mut max_val),
            Some(&mut min_loc),
            Some(&mut max_loc),
            &Mat::default(),
        )
        .unwrap_or_else(|_| panic!("min_max_loc failed for {:?}", path));

        // println!("Filename: {:?}", path.file_name());
        // println!("Best match confidence: {:.4}", max_val);
        // println!("Best match location: ({}, {})", max_loc.x, max_loc.y);

        card_positions.push(CardPosition {
            x: max_loc.x,
            y: max_loc.y,
            width: img_query.cols(),
            height: img_query.rows(),
            confidence: max_val,
            card: Card::from(path.file_stem().unwrap().to_str().unwrap()),
        });
    }

    card_positions.sort_by_key(|p| (p.y, p.x));

    card_positions
}
