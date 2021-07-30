// This is a custom trie implementation to register key words for parsing operation.

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{RefCell, RefMut};

#[derive(Debug)]
struct Node {
    token: char,
    is_word: bool,
    children: HashMap<char, Node>,
}

impl Node {
    pub fn new(token: char, is_word: bool) -> Self {
        Node { token, is_word, children: HashMap::new() }
    }
}

#[derive(Debug)]
pub struct Trie {
    root: Node,
}


impl Trie {
    pub fn new(op_words: Option<&Vec<&str>>) -> Self {
        let mut trie = Trie { root: Node::new(' ', false) };
        match op_words {
            Some(words) => {
                for word in words.iter() {
                    trie.insert_word(word);
                }
                trie
            },
            None =>  {
                trie    
            }
        }
    }

    pub fn insert_word(&mut self, word: &str) {
        let mut trie_iter = &mut self.root;

        for (index, c) in word.chars().enumerate() {
            let iter_children = &mut trie_iter.children;
            if !iter_children.contains_key(&c) {
                iter_children.insert(c, Node::new(c, index == word.len() - 1));
            }
            trie_iter = iter_children.get_mut(&c).unwrap();
             
        }
    }

    pub fn contains_word(&self, word: &str) -> bool {
        let mut trie_iter = &self.root;

        for c in word.chars() {
            let iter_children = &trie_iter.children;
            if iter_children.contains_key(&c) {
                trie_iter = iter_children.get(&c).unwrap(); 
            } else {
                return false;
            }
        }
        trie_iter.is_word
    }
}


#[test]
fn test_single_insert() {
    let mut trie = Trie::new(None);
    trie.insert_word("status");
    assert_eq!(trie.contains_word("status"), true);
    assert_eq!(trie.contains_word("status1"), false);
}

#[test]
fn test_multiple_insert() {
    let words = vec!["status", "score", "is_rewatching", "anime_num_episodes",
                     "anime_airing_status", "anime_id", "anime_title",
                     "start_date_string"];
    let mut trie = Trie::new(Some(&words));
    for word in words.iter() {
        trie.insert_word(&word);
    }

    for word in words.iter() {
        assert_eq!(trie.contains_word(&word), true);
    }

    assert_eq!(trie.contains_word("start_date_stringg"), false);
    assert_eq!(trie.contains_word("start_date_strin"), false);
    assert_eq!(trie.contains_word("start_"), false);
}
