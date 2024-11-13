use image::{DynamicImage, RgbImage, RgbaImage};
use log::warn;
use crate::{screen, solver};
use crate::solver::Word;

/// This is the pair where the board starts.
pub const BOARD_TOP: (u32, u32) = (57, 480);

/// The default 4x4 board is a 412 pixel square.
pub const BOARD_SIZE: u32 = 412;
/// There is 22 pixels of padding around the board.
pub const BOARD_PADDING: u32 = 22;

/// A screen tile is an 83 pixel square.
pub const TILE_SIZE: u32 = 83;
/// The gap between tiles is 12 pixels.
pub const TILE_GAP: u32 = 12;

/// This is the mouse coordinates for the first tile on the board.
pub const START_POS: (i32, i32) = (35, 165);

/// This is the error for color comparison.
pub const ERROR: u8 = 16;

/// Identifies valid words in the image, sorted by length.
/// image: The image to process.
pub fn words_in_image(image: &RgbaImage) -> Vec<Word> {
    // Process the image.
    let filtered = filter_image(image);
    let image = DynamicImage::ImageRgb8(filtered)
        .crop(BOARD_TOP.0, BOARD_TOP.1, BOARD_SIZE, BOARD_SIZE)
        .to_rgb8();

    // Use OCR to process the board.
    let lines = screen::process_board(&image);

    // Convert the lines into the proper board layout.
    let mut board: Vec<Vec<String>> = Vec::new();
    for line in lines.split('\n') {
        let mut row: Vec<String> = Vec::new();
        for c in line.chars() {
            row.push(c.to_lowercase().to_string());
        }
        board.push(row);
    }

    // Find all valid words.
    solver::find_all_words(&board)
}

/// Crops an image to find the row and column specified.
/// image: The source image to crop.
/// row: The row to crop.
/// column: The column to crop.
pub fn crop_image(mut image: DynamicImage, row: u32, column: u32) -> RgbImage {
    // Check if the image is the correct dimensions.
    let (width, height) = (image.width(), image.height());
    if width != BOARD_SIZE || height != BOARD_SIZE {
        warn!("Image is not the correct size. (got {}, {})", width, height);
        return image.to_rgb8();
    }

    // Crop the image using the dimensions above.
    let x = (column * TILE_SIZE) + BOARD_PADDING + (column * TILE_GAP);
    let y = (row * TILE_SIZE) + BOARD_PADDING + (row * TILE_GAP);

    image
        .crop(x, y, TILE_SIZE, TILE_SIZE)
        .to_rgb8()
}

/// Filters an image turning all non-black pixels into white.
/// This uses the RGB limit defined in the `ERROR` constant.
/// image: The RGB image to filter.
pub fn filter_image(image: &RgbaImage) -> RgbImage {
    let mut filtered_pixels = vec![0; (image.width() * image.height() * 3) as usize];

    for (x, y, pixel) in image.enumerate_pixels() {
        let [r, g, b, _] = pixel.0;
        let idx = (y * image.width() + x) as usize * 3;

        if r <= ERROR && g <= ERROR && b <= ERROR {
            filtered_pixels[idx..idx + 3].copy_from_slice(&[0, 0, 0]);
        } else {
            filtered_pixels[idx..idx + 3].copy_from_slice(&[255, 255, 255]);
        }
    }

    RgbImage::from_vec(image.width(), image.height(), filtered_pixels)
        .expect("Failed to create filtered image")
}

/// Converts a grid (x, y) pair into mouse (x, y) coordinates.
/// grid_pos: The grid position.
/// x: The grid x-coordinate.
/// y: The grid y-coordinate.
pub fn grid_to_mouse(grid_pos: (i32, i32), x: i32, y: i32) -> (i32, i32) {
    // Using the current grid position,
    // find the delta we need to get to the tile specified.
    // Getting the absolute position of a tile can be done with:
    // (35 + (x * 30), 160 + (y * 20))

    // Use these to configure the offset.
    const OFFSET_X: i32 = 30;
    const OFFSET_Y: i32 = 33;
    
    let (gx, gy) = grid_pos;

    let current = (START_POS.0 + (gx * OFFSET_X), START_POS.1 + (gy * OFFSET_Y));
    let target = (START_POS.0 + (x * OFFSET_X), START_POS.1 + (y * OFFSET_Y));

    (target.0 - current.0, target.1 - current.1)
}