use std::cmp::Ordering;
use std::collections::HashSet;
use crate::DICTIONARY;

/// All valid directions for locating adjacent characters.
const DIRECTIONS: [(i32, i32); 8] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1),           (0, 1),
    (1, -1),  (1, 0),  (1, 1),
];

/// A word found on the game board.
#[derive(Clone, Debug)]
#[derive(Eq, Hash, PartialEq)]
pub struct Word {
    /// The word itself.
    pub word: String,
    /// The characters that make up the word.
    /// These are paired in X, Y coordinates.
    /// They follow the order of the word.
    pub characters: Vec<(usize, usize)>
}

impl Word {
    /// Creates a new word.
    pub fn new() -> Self {
        Word {
            word: String::new(),
            characters: Vec::new()
        }
    }
    
    /// Returns the length of the word.
    pub fn len(&self) -> usize {
        self.word.len()
    }

    /// Compares this word with the given.
    /// other: The other word to compare.
    pub fn cmp(&self, other: &Word) -> Ordering {
        self.word.cmp(&other.word)
    }

    /// Appends a character to the word.
    /// char: The character to append.
    /// x: The X coordinate of the character.
    /// y: The Y coordinate of the character.
    pub fn append(&mut self, char: &String, x: usize, y: usize) {
        self.word.push_str(char);
        self.characters.push((x, y));
    }
    
    /// Truncates the word to a given length.
    pub fn truncate(&mut self, len: usize) {
        self.word.truncate(len);
        self.characters.truncate(len);
    }
}

/// Finds all valid words in a 2D board.
/// board: The game board.
pub fn find_all_words(board: &[Vec<String>]) -> Vec<Word> {
    let mut words = HashSet::new();
    let rows = board.len();
    let cols = board[0].len();
    let mut visited = vec![vec![false; cols]; rows];
    let mut current_word = Word::new();

    for row in 0..rows {
        for col in 0..cols {
            visit(
                board,
                row,
                col,
                &mut visited,
                &mut current_word,
                &mut words
            );
        }
    }

    let mut result: Vec<Word> = words.into_iter()
        .filter(|word| word.len() >= 3)
        .collect();

    // Sort by highest length.
    result.sort_by(|a, b| {
        b.len().cmp(&a.len()).then(a.cmp(b))
    });

    // Remove any duplicate entries.
    result.dedup_by(|a, b| a.word == b.word);

    result
}

/// Visits a position on the game board.
/// board: The game board.
/// row: The row index.
/// col: The column index.
/// visited: The visited positions.
/// current_word: The current word.
/// words: The set of valid words.
fn visit(
    board: &[Vec<String>],
    row: usize,
    col: usize,
    visited: &mut Vec<Vec<bool>>,
    current_word: &mut Word,
    words: &mut HashSet<Word>
) {
    let word_trie = DICTIONARY.read().unwrap();

    if !in_bounds(board, row, col) || visited[row][col] {
        return;
    }

    visited[row][col] = true;
    current_word.append(&board[row][col], col, row);

    if word_trie.has_prefix(&*current_word.word) {
        if word_trie.is_word(&*current_word.word) {
            words.insert(current_word.clone());
        }

        for &(dx, dy) in &DIRECTIONS {
            let new_row = row as i32 + dx;
            let new_col = col as i32 + dy;

            if new_row >= 0 && new_col >= 0 {
                visit(
                    board,
                    new_row as usize,
                    new_col as usize,
                    visited,
                    current_word,
                    words
                );
            }
        }
    }

    visited[row][col] = false;
    current_word.truncate(current_word.len() - board[row][col].len());
}

/// Checks if a position is within the boundaries of a game board.
/// board: The game board.
/// row: The row index.
/// col: The column index.
fn in_bounds(board: &[Vec<String>], row: usize, col: usize) -> bool {
    row < board.len() && col < board[0].len()
}
