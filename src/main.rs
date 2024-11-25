use std::path::Path;
use std::sync::{Arc, RwLock};
use lazy_static::lazy_static;
use log::info;
use crate::game::Game;
use crate::letters::Letters;
use crate::trie::TrieNode;

mod solver;
mod screen;
mod config;
mod board;
mod trie;
mod letters;
mod game;
mod mouse;

lazy_static! {
    pub static ref LETTERS: RwLock<Arc<Letters>> = RwLock::new(Arc::new(Letters::default()));
    pub static ref DICTIONARY: RwLock<Arc<TrieNode>> = RwLock::new(Arc::new(TrieNode::new()));
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure the environment for logging.
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    // Initialize the logger.
    pretty_env_logger::init();

    // Initialize the configuration.
    let config = config::init_config()?;
    
    unsafe {
        // Fetch the window handle and set the size.
        let handle = screen::get_window(&config.device_name);
        screen::set_size(handle,
                         config.screen_width, config.screen_height,
                         (config.window_x, config.window_y));
    
        info!("Set {:?} to {}x{}.", handle, config.screen_width, config.screen_height);
    }
    
    // Read the dictionary.
    load_dictionary(&config.dictionary);
    // Load the letters library.
    load_letters(&config.font);
    
    // Create a new game instance.
    let game = Game::new(&config).await?;
    game.start_game().await?;
    
    Ok(())
}

/// Loads a dictionary file.
/// path: The path to the dictionary file.
pub fn load_dictionary(path: &String) {
    let mut dictionary = TrieNode::new();

    // Check if the file exists.
    let path = Path::new(&path);
    if !path.exists() {
        return;
    }

    // Read the dictionary file.
    let contents = std::fs::read_to_string(&path)
        .expect("Couldn't read the dictionary file.");

    // Split the contents by newlines.
    for word in contents.lines() {
        dictionary.insert(word.to_lowercase().to_string());
    }

    info!("Loaded the dictionary with {} root words.", dictionary.len());

    // Lock and write to the dictionary global.
    let mut lock = DICTIONARY.write().unwrap();
    *lock = Arc::new(dictionary);

    // Unlock the dictionary.
    drop(lock);
}

/// Loads the letters map.
/// path: The path to the letters directory.
pub fn load_letters(path: &String) {
    let letters = Letters::new(path);

    info!("Loaded the letters library.");

    // Lock and write to the letters global.
    let mut lock = LETTERS.write().unwrap();
    *lock = Arc::new(letters);

    // Unlock the letters.
    drop(lock);
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use image::{DynamicImage, RgbImage};
    use crate::{load_dictionary, load_letters, LETTERS};
    use crate::board::crop_image;

    #[test]
    pub fn solve_boards() {
        // Load the dictionary.
        load_dictionary(&"words.txt".to_string());
        // Read the font images.
        load_letters(&"images".to_string());

        // Load each board.
        for i in 1..10 {
            // Read the board's image data from the file system.
            let board_image = image::open(format!("samples/{}.png", i))
                .map(|image| image.to_rgb8())
                .unwrap();

            // Lock the letters library.
            let letters = LETTERS.read().unwrap();

            // Split the image into a 4x4 grid.
            let mut board = String::new();

            // Crop the board into 4x4 tiles.
            let mut map: HashMap<(u32, u32), RgbImage> = HashMap::new();
            let image = DynamicImage::ImageRgb8(board_image.clone());
            for column in 0..4 {
                for row in 0..4 {
                    let image = crop_image(image.clone(), row, column);
                    // image.save(format!("test/{}_{}_{}.png", i, row, column)).unwrap();

                    print!("({}, {}) - ", row, column);

                    // Determine which letter matches the image.
                    let letter = letters.compare(&image);
                    board.push(letter);

                    map.insert((row, column), image);
                }
                println!();
            }

            // Read the correct words from the file system.
            let correct = std::fs::read_to_string(format!("samples/{}.txt", i))
                .unwrap();

            println!("Board   - {board}");
            println!("Correct - {correct}");

            // Compare the words.
            let mut lines = board.chars();

            let mut correct_map: HashMap<(u32, u32), char> = HashMap::new();
            let mut found_map: HashMap<(u32, u32), char> = HashMap::new();

            let mut row = 0;
            for line in correct.split('\n') {
                for column in 0..4u32 {
                    let c = line.chars().nth(column as usize).unwrap();
                    correct_map.insert((row, column), c);
                }
                row += 1;
            }

            for i in 0..16 {
                let row = i / 4;
                let column = i % 4;

                let found = lines.nth(i).unwrap();
                found_map.insert((row as u32, column as u32), found);
            }

            for column in 0..4u32 {
                for row in 0..4u32 {
                    let correct = correct_map.get(&(row, column)).unwrap();
                    let found = found_map.get(&(row, column)).unwrap();

                    // Print the image similarity scores.
                    let found_image = map.get(&(row, column)).unwrap();
                    let correct_image = letters.letters.get(&correct).unwrap();

                    let found_image = DynamicImage::ImageRgb8(found_image.clone()).to_luma8();

                    // let result = image_compare::gray_similarity_histogram(Metric::Hellinger, &found_image, correct_image).unwrap();
                    // println!("({}, {}) - {} vs {}: {}", row, column, found, correct, result);

                    assert_eq!(found, correct);
                }
            }
        }
    }
}
