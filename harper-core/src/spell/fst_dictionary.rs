use core::str;

use fst::Map as FstMap;
use fst::{automaton::Levenshtein, IntoStreamer};

use super::hunspell::{parse_default_attribute_list, parse_default_word_list};
use crate::{CharString, Lrc, WordMetadata};

use super::Dictionary;

#[derive(Debug)]
pub struct FstDictionary {
    /// Sorted by string in lexicographic order
    metadata: Vec<WordMetadata>,
    /// Sorted in lexicographic order
    words: Vec<CharString>,
    /// Used for fuzzy-finding the index of words or metadata
    word_map: FstMap<Vec<u8>>,
}

/// The uncached function that is used to produce the original copy of the
/// curated dictionary.
fn uncached_inner_new() -> Lrc<FstDictionary> {
    let word_list = parse_default_word_list().unwrap();
    let attr_list = parse_default_attribute_list();

    // There will be at _least_ this number of words
    // This creates a memory map, which enables searching the map without loading
    // all of it into memory.
    let word_map = FstMap::new(include_bytes!("../../dictionary.fst").to_vec()).unwrap();

    let mut words: Vec<CharString> = word_list
        .iter()
        .map(|mw| mw.letters.as_ref().into())
        .collect();
    words.sort();
    words.dedup();
    let metadata: Vec<WordMetadata> = todo!();

    Lrc::new(FstDictionary {
        metadata,
        words,
        word_map,
    })
}

thread_local! {
    static DICT: Lrc<FstDictionary> = uncached_inner_new();
}

impl Dictionary for FstDictionary {
    fn contains_word(&self, word: &[char]) -> bool {
        self.word_map.contains_key(word.iter().collect::<String>())
    }

    fn contains_word_str(&self, word: &str) -> bool {
        self.word_map.contains_key(word)
    }

    fn get_word_metadata(&self, word: &[char]) -> WordMetadata {
        let index: usize = self.word_map.get(word.iter().collect::<String>()).unwrap() as usize;
        self.metadata[index]
    }

    fn get_word_metadata_str(&self, word: &str) -> WordMetadata {
        let index: usize = self.word_map.get(word).unwrap() as usize;
        self.metadata[index]
    }

    fn fuzzy_match(
        &self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<(&[char], u8, WordMetadata)> {
        self.fuzzy_match_str(&word.iter().collect::<String>(), max_distance, max_results)
    }

    fn fuzzy_match_str(
        &self,
        word: &str,
        max_distance: u8,
        max_results: usize,
    ) -> Vec<(&[char], u8, WordMetadata)> {
        let aut = Levenshtein::new(word, max_distance as u32).unwrap();
        let word_indexes: Vec<u64> = self.word_map.search(aut).into_stream().into_values();
        word_indexes
            .into_iter()
            .take(max_results)
            .map(|i| (self.words.get(i as usize).unwrap(), i))
            .map(|(word, i)| (word.as_slice(), i as u8, self.metadata[i as usize]))
            .collect()
    }
}
