use std::time::Duration;
use log::info;
use tokio::sync::Mutex;
use tokio::time::sleep;
use crate::{board, screen};
use crate::config::Config;
use crate::mouse::Mouse;
use crate::solver::Word;

/// This is the X, Y mouse coordinates of the start game button.
pub const START_BUTTON: (i32, i32) = (70, 245);

pub struct Game {
    device: String,
    mouse: Mutex<Mouse>
}

impl Game {
    /// Creates a new game instance.
    /// config: The application configuration.
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        // Connect to the mouse server.
        let mut mouse = Mouse::new(
            config.server_address.clone(),
            config.server_port
        ).await?;

        // Normalize the mouse connection.
        mouse.normalize().await;

        Ok(Game {
            device: config.device_name.clone(),
            mouse: Mutex::new(mouse)
        })
    }

    /// Starts the game instance.
    pub async fn start_game(&self) -> anyhow::Result<()> {
        // Lock the mouse object.
        let mut mouse = self.mouse.lock().await;

        // Start the game.
        mouse.move_absolute(START_BUTTON, true).await?;
        sleep(Duration::from_millis(50)).await;

        mouse.click().await;
        sleep(Duration::from_millis(50)).await;

        // Move the mouse so we aren't blocking the screen.
        mouse.normalize().await;
        sleep(Duration::from_millis(1000)).await;

        // Release the mouse lock.
        drop(mouse);

        // Get the words on the board.
        let words = self.get_board();
        info!("Found {} words.", words.len());

        // Start the primary loop.
        self.do_mouse_loop(words).await?;

        Ok(())
    }

    /// This is the primary loop used for solving the game.
    /// words: The words on the board to select.
    async fn do_mouse_loop(&self, words: Vec<Word>) -> anyhow::Result<()> {
        // Lock the mouse.
        let mut mouse = self.mouse.lock().await;

        // Iterate over every word.
        for word in words {
            info!("Trying to solve word: {}", word.word);

            mouse.move_absolute(board::START_POS, true).await?;
            sleep(Duration::from_millis(50)).await;

            let mut grid_pos = (0, 0);
            let mut points: Vec<(i32, i32)> = Vec::new();

            // Calculate the points to move to.
            for (x, y) in &word.characters {
                let delta = board::grid_to_mouse(
                    (grid_pos.0 as i32, grid_pos.1 as i32),
                    (*x) as i32, (*y) as i32
                );

                points.push(delta);
                grid_pos = (*x, *y);
            }

            // Move the mouse.
            mouse.move_group(points).await?;

            sleep(Duration::from_millis(100)).await;
        }

        info!("Done!");

        Ok(())
    }

    /// Takes a picture of the device.
    /// Returns a vector of words found on the board.
    /// This method assumes the game board is open.
    fn get_board(&self) -> Vec<Word> {
        // Take a screenshot of the board.
        let board = screen::take_screenshot(&self.device).unwrap();
        // Perform OCR on the board and find all words.
        board::words_in_image(&board)
    }
}
