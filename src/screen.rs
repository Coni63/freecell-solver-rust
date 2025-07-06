use image::{ImageBuffer, RgbaImage};
use rdev::{Button, Event, EventType, listen};
use scrap::{Capturer, Display};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread};

pub struct Screenshot {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub img: RgbaImage,
}

fn ocr_with_cli(image_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output =
        Command::new(std::env::var("TESSERACT_PATH").unwrap_or_else(|_| "tesseract".to_string()))
            .arg(image_path)
            .arg("stdout") // affiche dans la sortie standard
            .arg("-l")
            .arg("eng")
            .output()?;

    if output.status.success() {
        let text = String::from_utf8(output.stdout)?;
        Ok(text)
    } else {
        Err(format!("Tesseract failed: {:?}", output.stderr).into())
    }
}

#[allow(dead_code)]
pub fn run_ocr(buffer: &RgbaImage) -> Option<String> {
    let temp_path = "temp_ocr.png";
    buffer.save(temp_path).unwrap();

    let mut ans = None;
    match ocr_with_cli(temp_path) {
        Ok(text) => {
            ans = Some(text);
        }
        Err(e) => eprintln!("Erreur OCR : {}", e),
    }

    fs::remove_file(temp_path).unwrap_or_else(|_| {
        eprintln!(
            "⚠️ Impossible de supprimer le fichier temporaire {}",
            temp_path
        )
    });

    ans
}

fn capture_region(x1: i32, y1: i32, x2: i32, y2: i32) -> RgbaImage {
    let display = Display::primary().unwrap();
    let mut capturer = Capturer::new(display).unwrap();
    let h = capturer.height();

    let frame = loop {
        if let Ok(buffer) = capturer.frame() {
            break buffer;
        }
        thread::sleep(Duration::from_millis(10));
    };

    let (x_min, x_max) = (x1.min(x2), x1.max(x2));
    let (y_min, y_max) = (y1.min(y2), y1.max(y2));
    let width = x_max - x_min;
    let height = y_max - y_min;

    let mut img: RgbaImage = ImageBuffer::new(width as u32, height as u32);

    let stride = frame.len() / h;

    for y in y_min..y_max {
        for x in x_min..x_max {
            let idx = y as usize * stride + 4 * x as usize;
            if idx + 3 < frame.len() {
                let pixel = image::Rgba([
                    frame[idx],     // R
                    frame[idx + 1], // G
                    frame[idx + 2], // B
                    frame[idx + 3], // A
                ]);
                img.put_pixel((x - x_min) as u32, (y - y_min) as u32, pixel);
            }
        }
    }

    img.save("capture.png").unwrap();
    println!("✅ Zone capturée sauvegardée dans `capture.png`");

    img
}

#[allow(dead_code)]
pub fn start_screenshot() -> Screenshot {
    let click_points: Arc<Mutex<Vec<(i32, i32)>>> = Arc::new(Mutex::new(vec![]));
    let click_points_clone = Arc::clone(&click_points);
    let current_pos: Arc<Mutex<(f64, f64)>> = Arc::new(Mutex::new((0.0, 0.0)));
    let current_pos_clone = Arc::clone(&current_pos);

    println!("🖱️ Cliquez deux fois pour définir la zone à capturer...");

    thread::spawn(move || {
        let _ = listen(move |event: Event| {
            match event.event_type {
                EventType::MouseMove { x, y } => {
                    // Track current mouse position
                    let mut pos = current_pos_clone.lock().unwrap();
                    *pos = (x, y);
                }
                EventType::ButtonPress(button) => {
                    if button == Button::Left {
                        // Capture current position when left click occurs
                        let pos = current_pos_clone.lock().unwrap();
                        let mut points = click_points_clone.lock().unwrap();
                        points.push((pos.0 as i32, pos.1 as i32));
                        println!("📍 Clic à : ({}, {})", pos.0 as i32, pos.1 as i32);
                    }
                }
                _ => {}
            }
        });
    });

    // Attendre que le thread se termine (c’est-à-dire 2 clics)
    loop {
        {
            let points = click_points.lock().unwrap();
            if points.len() == 2 {
                let (x1, y1) = points[0];
                let (x2, y2) = points[1];
                return Screenshot {
                    x1,
                    y1,
                    x2,
                    y2,
                    img: capture_region(x1, y1, x2, y2),
                };
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}
