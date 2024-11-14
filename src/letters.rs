use std::collections::HashMap;
use image::{DynamicImage, Rgb, RgbImage, RgbaImage};

pub const WHITE: Rgb<u8> = Rgb([255, 255, 255]);

pub struct Letters {
    pub letters: HashMap<char, RgbaImage>
}

impl Letters {
    /// Creates a new letter set/matcher.
    /// path: The path to the letter images.
    pub fn new(path: &String) -> Self {
        let mut letters = HashMap::new();

        // There are currently no known letters for 'Q' and 'Z'.
        for c in "abcdefghijklmnoprstuvwxy".chars() {
            let image = image::open(format!("{}/{}.png", path, c)).unwrap();
            letters.insert(c, image.to_rgba8());
        }

        Self { letters }
    }

    /// Determines which letter is the closest match.
    /// image: The image to compare.
    pub fn compare(&self, image: &RgbImage) -> char {
        let mut best = (' ', 0f64);
        let mut second_best = (' ', 0f64);

        let image = DynamicImage::ImageRgb8(image.clone()).to_rgba8();
        for (letter, letter_image) in &self.letters {
            if let Ok(result) = image_compare::rgba_blended_hybrid_compare(
                (&image).into(), letter_image.into(), WHITE
            ) {
                if result.score > best.1 {
                    second_best = best;
                    best = (*letter, result.score);
                } else if result.score > second_best.1 {
                    second_best = (*letter, result.score);
                }
            }
        }

        best.0
    }
}

impl Default for Letters {
    fn default() -> Self {
        Self {
            letters: HashMap::new()
        }
    }
}
