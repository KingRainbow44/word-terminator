use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct TrieNode {
    children: HashMap<char, TrieNode>,
    is_end_of_word: bool,
}

impl TrieNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<S: AsRef<str>>(&mut self, word: S) {
        let mut current = self;
        for ch in word.as_ref().chars() {
            current = current.children.entry(ch).or_default();
        }
        current.is_end_of_word = true;
    }

    pub fn has_prefix(&self, prefix: &str) -> bool {
        let mut node = self;
        for ch in prefix.chars() {
            match node.children.get(&ch) {
                Some(next) => node = next,
                None => return false,
            }
        }
        true
    }

    pub fn is_word(&self, word: &str) -> bool {
        let mut node = self;
        for ch in word.chars() {
            match node.children.get(&ch) {
                Some(next) => node = next,
                None => return false,
            }
        }
        node.is_end_of_word
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }
}
