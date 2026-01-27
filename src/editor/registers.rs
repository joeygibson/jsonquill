use crate::document::node::JsonNode;
use std::collections::HashMap;

/// Content stored in a register (nodes + optional keys for object members)
#[derive(Debug, Clone, PartialEq)]
pub struct RegisterContent {
    pub nodes: Vec<JsonNode>,
    pub keys: Vec<Option<String>>,
}

impl RegisterContent {
    pub fn new(nodes: Vec<JsonNode>, keys: Vec<Option<String>>) -> Self {
        Self { nodes, keys }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// Manages all registers (unnamed, named a-z, numbered 0-9)
#[derive(Debug, Clone)]
pub struct RegisterSet {
    unnamed: RegisterContent,
    named: HashMap<char, RegisterContent>,
    numbered: [RegisterContent; 10],
}

impl RegisterSet {
    pub fn new() -> Self {
        Self {
            unnamed: RegisterContent::new(vec![], vec![]),
            named: HashMap::new(),
            numbered: [
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
                RegisterContent::new(vec![], vec![]),
            ],
        }
    }

    pub fn get_unnamed(&self) -> &RegisterContent {
        &self.unnamed
    }

    pub fn set_unnamed(&mut self, content: RegisterContent) {
        self.unnamed = content;
    }

    pub fn get_named(&self, register: char) -> Option<&RegisterContent> {
        self.named.get(&register.to_ascii_lowercase())
    }

    pub fn set_named(&mut self, register: char, content: RegisterContent) {
        self.named.insert(register.to_ascii_lowercase(), content);
    }

    /// Get content from any register (named a-z or numbered 0-9).
    pub fn get(&self, register: char) -> Option<&RegisterContent> {
        if register.is_ascii_digit() {
            register.to_digit(10).map(|d| &self.numbered[d as usize])
        } else {
            self.get_named(register)
        }
    }

    /// Gets content from numbered register (0-9).
    ///
    /// # Panics
    /// Panics in debug builds if index >= 10
    pub fn get_numbered(&self, index: usize) -> &RegisterContent {
        debug_assert!(index < 10, "numbered register index must be 0-9");
        &self.numbered[index]
    }

    /// Sets content for numbered register (0-9).
    ///
    /// # Panics
    /// Panics in debug builds if index >= 10
    pub fn set_numbered(&mut self, index: usize, content: RegisterContent) {
        debug_assert!(index < 10, "numbered register index must be 0-9");
        self.numbered[index] = content;
    }

    /// Appends content to a named register. If the register doesn't exist,
    /// creates it with the provided content.
    pub fn append_named(&mut self, register: char, content: RegisterContent) {
        let key = register.to_ascii_lowercase();
        if let Some(existing) = self.named.get_mut(&key) {
            existing.nodes.extend(content.nodes);
            existing.keys.extend(content.keys);
        } else {
            self.named.insert(key, content);
        }
    }

    /// Pushes new content to delete history. Shifts all history registers:
    /// "9 is lost, "8→"9, ..., "2→"3, "1→"2, and new content goes to "1.
    pub fn push_delete_history(&mut self, content: RegisterContent) {
        // Shift history: "9 lost, "8→"9, ..., "2→"3, "1→"2
        for i in (1..9).rev() {
            self.numbered[i + 1] = self.numbered[i].clone();
        }
        // New delete goes to "1
        self.numbered[1] = content;
    }

    /// Updates the yank register "0 with the latest yank content.
    pub fn update_yank_register(&mut self, content: RegisterContent) {
        // Update "0 with latest yank
        self.numbered[0] = content;
    }
}

impl Default for RegisterSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::node::{JsonNode, JsonValue};

    #[test]
    fn test_register_content_new() {
        let node = JsonNode::new(JsonValue::String("test".to_string()));
        let content = RegisterContent::new(vec![node.clone()], vec![None]);

        assert_eq!(content.nodes.len(), 1);
        assert_eq!(content.keys.len(), 1);
    }

    #[test]
    fn test_register_set_new() {
        let regs = RegisterSet::new();
        assert!(regs.get_unnamed().is_empty());
        assert_eq!(regs.get_named('a'), None);
        assert!(regs.get_numbered(0).is_empty());
    }

    #[test]
    fn test_register_set_named() {
        let mut regs = RegisterSet::new();
        let node = JsonNode::new(JsonValue::Number(42.0));
        let content = RegisterContent::new(vec![node.clone()], vec![None]);

        regs.set_named('a', content.clone());

        let retrieved = regs.get_named('a').unwrap();
        assert_eq!(retrieved.nodes.len(), 1);
    }

    #[test]
    fn test_register_set_unnamed() {
        let mut regs = RegisterSet::new();
        let node = JsonNode::new(JsonValue::Boolean(true));
        let content = RegisterContent::new(vec![node.clone()], vec![None]);

        regs.set_unnamed(content.clone());

        let retrieved = regs.get_unnamed();
        assert_eq!(retrieved.nodes.len(), 1);
    }

    #[test]
    fn test_register_set_numbered() {
        let mut regs = RegisterSet::new();
        let node = JsonNode::new(JsonValue::Null);
        let content = RegisterContent::new(vec![node.clone()], vec![None]);

        regs.set_numbered(5, content.clone());

        let retrieved = regs.get_numbered(5);
        assert_eq!(retrieved.nodes.len(), 1);
    }

    #[test]
    fn test_register_numbered_valid_range() {
        let mut regs = RegisterSet::new();
        let node = JsonNode::new(JsonValue::String("test".to_string()));
        let content = RegisterContent::new(vec![node.clone()], vec![None]);

        // Test all valid indices 0-9
        for i in 0..10 {
            regs.set_numbered(i, content.clone());
            let retrieved = regs.get_numbered(i);
            assert_eq!(retrieved.nodes.len(), 1);
        }
    }

    #[test]
    fn test_register_append_named() {
        let mut regs = RegisterSet::new();
        let node1 = JsonNode::new(JsonValue::Number(1.0));
        let node2 = JsonNode::new(JsonValue::Number(2.0));

        regs.set_named('a', RegisterContent::new(vec![node1.clone()], vec![None]));
        regs.append_named('a', RegisterContent::new(vec![node2.clone()], vec![None]));

        let retrieved = regs.get_named('a').unwrap();
        assert_eq!(retrieved.nodes.len(), 2);
    }

    #[test]
    fn test_register_push_delete_history() {
        let mut regs = RegisterSet::new();
        let node1 = JsonNode::new(JsonValue::Number(1.0));
        let node2 = JsonNode::new(JsonValue::Number(2.0));
        let node3 = JsonNode::new(JsonValue::Number(3.0));

        regs.push_delete_history(RegisterContent::new(vec![node1.clone()], vec![None]));
        regs.push_delete_history(RegisterContent::new(vec![node2.clone()], vec![None]));
        regs.push_delete_history(RegisterContent::new(vec![node3.clone()], vec![None]));

        // "1 should have most recent (node3)
        assert_eq!(regs.get_numbered(1).nodes.len(), 1);
        // "2 should have node2
        assert_eq!(regs.get_numbered(2).nodes.len(), 1);
    }

    #[test]
    fn test_register_update_yank_register() {
        let mut regs = RegisterSet::new();
        let node = JsonNode::new(JsonValue::Boolean(true));
        let content = RegisterContent::new(vec![node.clone()], vec![None]);

        regs.update_yank_register(content.clone());

        // "0 should have the yanked content
        assert_eq!(regs.get_numbered(0).nodes.len(), 1);
    }

    #[test]
    fn test_register_get_unified() {
        let mut regs = RegisterSet::new();
        let node1 = JsonNode::new(JsonValue::Number(1.0));
        let node2 = JsonNode::new(JsonValue::Number(2.0));

        // Set a named register
        regs.set_named('a', RegisterContent::new(vec![node1.clone()], vec![None]));

        // Set a numbered register
        regs.set_numbered(5, RegisterContent::new(vec![node2.clone()], vec![None]));

        // Test get() for named register
        let retrieved_a = regs.get('a').unwrap();
        assert_eq!(retrieved_a.nodes.len(), 1);

        // Test get() for numbered register
        let retrieved_5 = regs.get('5').unwrap();
        assert_eq!(retrieved_5.nodes.len(), 1);

        // Test get() for non-existent named register
        assert!(regs.get('z').is_none());
    }
}
