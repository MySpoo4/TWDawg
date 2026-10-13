use std::collections::{BTreeSet, HashSet};

use twdawg::{TWDawg, TWDawgBuilder};

// Small deterministic RNG so failures are reproducible
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, n: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) % n
    }

    fn word(&mut self, alphabet: u64, max_len: u64) -> String {
        let len = 1 + self.next(max_len);
        (0..len)
            .map(|_| (b'a' + self.next(alphabet) as u8) as char)
            .collect()
    }
}

// Path stored for `word` split at `k`: rev(word[..k]) + '[' + word[k..]
fn accepts_split(dawg: &TWDawg, word: &str, k: usize) -> bool {
    let word = word.as_bytes();
    let path = word[..k]
        .iter()
        .rev()
        .chain(std::iter::once(&b'['))
        .chain(&word[k..]);

    let mut node = dawg.get_node(0);
    for &c in path {
        match node.get(c) {
            Some(id) => node = dawg.get_node(id),
            None => return false,
        }
    }
    node.is_terminal
}

fn node_ids_match_indices(dawg: &TWDawg) -> bool {
    let mut stack = vec![0];
    let mut seen = HashSet::new();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        let node = dawg.get_node(id);
        if node.id != id {
            return false;
        }
        stack.extend(node.edges.iter().flatten());
    }
    true
}

fn random_cases(seed: u64, mut check: impl FnMut(&[String], &[String])) {
    let mut rng = Lcg(seed);
    for t in 0..500 {
        let alphabet = [2, 3, 4, 26][t % 4];
        let max_len = [3, 6, 8][t % 3];
        let count = 1 + rng.next(40);
        let words: Vec<String> = (0..count).map(|_| rng.word(alphabet, max_len)).collect();
        let probes: Vec<String> = (0..100).map(|_| rng.word(alphabet, max_len)).collect();
        check(&words, &probes);
    }
}

fn upper_set(words: &[String]) -> BTreeSet<String> {
    words.iter().map(|w| w.to_ascii_uppercase()).collect()
}

#[test]
fn terminal_and_non_terminal_nodes_are_not_merged() {
    let dawg = TWDawgBuilder::build(["ab", "abc", "xbc"]);
    assert!(dawg.contains("ab"));
    assert!(dawg.contains("xbc"));
    assert!(!dawg.contains("xb"));
}

#[test]
fn build_stores_every_split_and_nothing_else() {
    random_cases(987654321, |words, probes| {
        let dawg = TWDawgBuilder::build(words.iter().map(String::as_str));
        let expected = upper_set(words);

        for word in &expected {
            for k in 1..=word.len() {
                assert!(accepts_split(&dawg, word, k), "{word} split {k} missing, words: {words:?}");
            }
        }
        for probe in probes {
            let upper = probe.to_ascii_uppercase();
            let is_word = expected.contains(&upper);
            assert_eq!(dawg.contains(probe), is_word, "contains({probe}), words: {words:?}");
            for k in 1..=upper.len() {
                assert_eq!(accepts_split(&dawg, &upper, k), is_word, "{probe} split {k}, words: {words:?}");
            }
        }

        let all: Vec<String> = dawg.get_all_words();
        assert_eq!(all.len(), expected.len(), "duplicates in get_all_words");
        assert_eq!(all.into_iter().collect::<BTreeSet<_>>(), expected);
        assert!(node_ids_match_indices(&dawg));
    });
}

#[test]
fn build_one_way_matches_word_set() {
    random_cases(12345, |words, probes| {
        let dawg = TWDawgBuilder::build_one_way(words.iter().map(String::as_str));
        let expected = upper_set(words);

        for probe in words.iter().chain(probes) {
            let is_word = expected.contains(&probe.to_ascii_uppercase());
            assert_eq!(dawg.contains(probe), is_word, "contains({probe}), words: {words:?}");
        }
        let all: Vec<String> = dawg.get_all_words();
        assert_eq!(all.len(), expected.len(), "duplicates in get_all_words");
        assert_eq!(all.into_iter().collect::<BTreeSet<_>>(), expected);
        assert!(node_ids_match_indices(&dawg));
    });
}

#[test]
fn invalid_words_are_skipped_and_never_contained() {
    let words = ["dog", "", "do-g", "café", "c4t", "Cat"];
    for dawg in [TWDawgBuilder::build(words), TWDawgBuilder::build_one_way(words)] {
        assert!(dawg.contains("dog"));
        assert!(dawg.contains("CAT"));
        for w in ["", "do-g", "café", "c4t", "[", "dog["] {
            assert!(!dawg.contains(w), "contains({w:?})");
        }
        let mut all = dawg.get_all_words();
        all.sort();
        assert_eq!(all, ["CAT", "DOG"]);
    }
}
