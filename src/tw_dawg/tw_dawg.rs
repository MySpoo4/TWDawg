use crate::utils::{RevWord, StrBuilder};

use super::{TWDNode, builder::is_valid_word, node::ALPHABET_SIZE};

#[derive(Debug)]
pub struct TWDawg {
    nodes: Vec<TWDNode>,
}

impl TWDawg {
    pub fn new(nodes: Vec<TWDNode>) -> Self {
        Self { nodes }
    }

    pub fn root(&self) -> Option<&TWDNode> {
        self.nodes.get(0)
    }

    pub fn get_node(&self, id: usize) -> &TWDNode {
        &self.nodes[id]
    }

    pub fn contains(&self, word: &str) -> bool {
        if !is_valid_word(word) {
            return false;
        }

        // Follow the path both builders store: first letter, separator, rest of the word
        RevWord::build(word, 1)
            .iter_all()
            .try_fold(self.get_node(0), |node, c| {
                node.get(c as u8).map(|id| self.get_node(id))
            })
            .is_some_and(|node| node.is_terminal)
    }

    pub fn get_all_words(&self) -> Vec<String> {
        let mut words = Vec::new();
        let mut builder = StrBuilder::new();

        // Only walk the first letter -> separator path so each word is found once
        let root = self.get_node(0);
        root.edges
            .iter()
            .take(ALPHABET_SIZE)
            .enumerate()
            .for_each(|(i, &id)| {
                let Some(rev_id) = id.and_then(|id| self.get_node(id).edges[ALPHABET_SIZE]) else {
                    return;
                };

                builder.push(char::from_u32((i + 65) as u32).unwrap());
                builder.update_rev();
                self.get_words_helper(&mut words, &mut builder, rev_id);
                builder.revert_rev();
                builder.pop();
            });

        words
    }

    fn get_words_helper(&self, words: &mut Vec<String>, builder: &mut StrBuilder, node_id: usize) {
        let node = self.get_node(node_id);

        if node.is_terminal {
            words.push(builder.build());
        }

        node.edges
            .iter()
            .take(ALPHABET_SIZE)
            .enumerate()
            .for_each(|(i, &id)| {
                if let Some(node_id) = id {
                    builder.push(char::from_u32((i + 65) as u32).unwrap());
                    self.get_words_helper(words, builder, node_id);
                    builder.pop();
                }
            });
    }
}
