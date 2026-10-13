use std::{
    fmt,
    hash::{Hash, Hasher},
};

pub const ALPHABET_SIZE: usize = 26;

// Represents a node in the TWDawg.
#[derive(Clone)]
pub struct TWDNode {
    pub id: usize,
    pub is_terminal: bool,
    // Extra for reverse
    pub edges: [Option<usize>; ALPHABET_SIZE + 1],
}

impl TWDNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            is_terminal: false,
            edges: [None; ALPHABET_SIZE + 1],
        }
    }

    pub fn get(&self, c: u8) -> Option<usize> {
        assert!(b'A' <= c && c <= b'[', "Invalid char index");
        self.edges[(c - b'A') as usize]
    }

    pub fn set(&mut self, c: u8, v: usize) {
        assert!(b'A' <= c && c <= b'[', "Invalid char index");
        self.edges[(c - b'A') as usize] = Some(v);
    }

    pub fn contains_edge(&self, c: u8) -> bool {
        (b'A'..=b'[').contains(&c) && self.get(c).is_some()
    }
}

// Ignore `id` for equality checks
impl PartialEq for TWDNode {
    fn eq(&self, other: &Self) -> bool {
        self.is_terminal == other.is_terminal && self.edges == other.edges
    }
}

impl Eq for TWDNode {}

impl Hash for TWDNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.is_terminal.hash(state);
        // faster hashing instead of hashmap
        self.edges.iter().for_each(|v| v.hash(state));
    }
}

impl fmt::Debug for TWDNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut edge_map = Vec::new();
        for (i, &edge) in self.edges.iter().take(ALPHABET_SIZE).enumerate() {
            if let Some(target) = edge {
                let label = (b'A' + i as u8) as char;
                edge_map.push(format!("{} -> {}", label, target));
            }
        }

        f.debug_struct("TWDNode")
            .field("id", &self.id)
            .field("is_terminal", &self.is_terminal)
            .field("edges", &edge_map)
            .field("rev_id", &self.edges[ALPHABET_SIZE])
            .finish()
    }
}
