use crate::document::node::JsonNode;

/// Content stored in a register (nodes + optional keys for object members)
#[derive(Debug, Clone)]
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
}
