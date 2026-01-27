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
}
