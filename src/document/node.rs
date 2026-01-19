#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Object(Vec<(String, JsonNode)>),
    Array(Vec<JsonNode>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonNode {
    value: JsonValue,
    metadata: NodeMetadata,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeMetadata {
    /// Original formatting (whitespace, indentation)
    pub original_text: Option<String>,
    /// Whether this node has been modified
    pub modified: bool,
}

impl JsonNode {
    pub fn new(value: JsonValue) -> Self {
        Self {
            value,
            metadata: NodeMetadata {
                original_text: None,
                modified: true,
            },
        }
    }

    pub fn value(&self) -> &JsonValue {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut JsonValue {
        self.metadata.modified = true;
        &mut self.value
    }

    pub fn is_modified(&self) -> bool {
        self.metadata.modified
    }
}
