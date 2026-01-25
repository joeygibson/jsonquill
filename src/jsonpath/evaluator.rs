use super::ast::PathSegment;
use crate::document::node::{JsonNode, JsonValue};

pub struct Evaluator<'a> {
    root: &'a JsonNode,
}

impl<'a> Evaluator<'a> {
    pub fn new(root: &'a JsonNode) -> Self {
        Evaluator { root }
    }

    pub fn evaluate(&self, segments: &[PathSegment]) -> Vec<&'a JsonNode> {
        if segments.is_empty() {
            return vec![];
        }

        // Start with root
        let mut current: Vec<&JsonNode> = vec![self.root];

        // Process each segment
        for segment in segments {
            let mut next = Vec::new();
            for node in &current {
                next.extend(self.evaluate_segment(node, segment));
            }
            current = next;
        }

        current
    }

    fn evaluate_segment(&self, node: &'a JsonNode, segment: &PathSegment) -> Vec<&'a JsonNode> {
        match segment {
            PathSegment::Root => vec![self.root],
            PathSegment::Current => vec![node],
            PathSegment::Child(name) => self.find_child(node, name),
            PathSegment::Index(idx) => self.get_array_element(node, *idx),
            PathSegment::Wildcard => self.get_all_children(node),
            PathSegment::RecursiveDescent(prop) => self.recursive_descent(node, prop.as_deref()),
            PathSegment::Slice(start, end) => self.get_slice(node, *start, *end),
            PathSegment::MultiProperty(props) => {
                let mut results = Vec::new();
                for prop in props {
                    results.extend(self.find_child(node, prop));
                }
                results
            }
        }
    }

    fn find_child(&self, node: &'a JsonNode, name: &str) -> Vec<&'a JsonNode> {
        if let JsonValue::Object(props) = node.value() {
            for (key, child) in props {
                if key == name {
                    return vec![child];
                }
            }
        }
        vec![]
    }

    fn get_array_element(&self, node: &'a JsonNode, idx: isize) -> Vec<&'a JsonNode> {
        if let JsonValue::Array(items) = node.value() {
            let len = items.len() as isize;
            let normalized_idx = if idx < 0 { len + idx } else { idx };

            if normalized_idx >= 0 && (normalized_idx as usize) < items.len() {
                return vec![&items[normalized_idx as usize]];
            }
        }
        vec![]
    }

    fn get_all_children(&self, node: &'a JsonNode) -> Vec<&'a JsonNode> {
        match node.value() {
            JsonValue::Object(props) => props.iter().map(|(_, child)| child).collect(),
            JsonValue::Array(items) => items.iter().collect(),
            JsonValue::JsonlRoot(lines) => lines.iter().collect(),
            _ => vec![],
        }
    }

    fn get_slice(
        &self,
        node: &'a JsonNode,
        start: Option<isize>,
        end: Option<isize>,
    ) -> Vec<&'a JsonNode> {
        if let JsonValue::Array(items) = node.value() {
            let len = items.len() as isize;

            // Normalize start
            let start_idx = match start {
                Some(s) if s < 0 => (len + s).max(0) as usize,
                Some(s) => s.min(len) as usize,
                None => 0,
            };

            // Normalize end
            let end_idx = match end {
                Some(e) if e < 0 => (len + e).max(0) as usize,
                Some(e) => e.min(len) as usize,
                None => len as usize,
            };

            if start_idx <= end_idx {
                return items[start_idx..end_idx].iter().collect();
            }
        }
        vec![]
    }

    fn recursive_descent(&self, node: &'a JsonNode, prop: Option<&str>) -> Vec<&'a JsonNode> {
        let mut results = Vec::new();

        // Helper to recursively walk the tree
        fn walk<'a>(node: &'a JsonNode, prop: Option<&str>, results: &mut Vec<&'a JsonNode>) {
            // If property name specified, only match that property
            if let Some(name) = prop {
                if let JsonValue::Object(props) = node.value() {
                    for (key, child) in props {
                        if key == name {
                            results.push(child);
                        }
                        walk(child, prop, results);
                    }
                } else if let JsonValue::Array(items) = node.value() {
                    for item in items {
                        walk(item, prop, results);
                    }
                }
            } else {
                // No property name - match all nodes
                match node.value() {
                    JsonValue::Object(props) => {
                        for (_, child) in props {
                            results.push(child);
                            walk(child, prop, results);
                        }
                    }
                    JsonValue::Array(items) => {
                        for item in items {
                            results.push(item);
                            walk(item, prop, results);
                        }
                    }
                    _ => {}
                }
            }
        }

        walk(node, prop, &mut results);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_tree() -> JsonNode {
        let items = vec![
            JsonNode::new(JsonValue::String("a".to_string())),
            JsonNode::new(JsonValue::String("b".to_string())),
            JsonNode::new(JsonValue::String("c".to_string())),
        ];

        let obj = vec![
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String("test".to_string())),
            ),
            ("age".to_string(), JsonNode::new(JsonValue::Number(42.0))),
            ("items".to_string(), JsonNode::new(JsonValue::Array(items))),
        ];

        JsonNode::new(JsonValue::Object(obj))
    }

    #[test]
    fn test_evaluate_root() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let results = evaluator.evaluate(&[PathSegment::Root]);
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].value(), JsonValue::Object(_)));
    }

    #[test]
    fn test_evaluate_child() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let results =
            evaluator.evaluate(&[PathSegment::Root, PathSegment::Child("name".to_string())]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value(), &JsonValue::String("test".to_string()));
    }

    #[test]
    fn test_evaluate_array_index() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let results = evaluator.evaluate(&[
            PathSegment::Root,
            PathSegment::Child("items".to_string()),
            PathSegment::Index(1),
        ]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value(), &JsonValue::String("b".to_string()));
    }

    #[test]
    fn test_evaluate_wildcard() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let results = evaluator.evaluate(&[PathSegment::Root, PathSegment::Wildcard]);
        assert_eq!(results.len(), 3); // name, age, items
    }

    #[test]
    fn test_evaluate_recursive_descent() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let results = evaluator.evaluate(&[PathSegment::Root, PathSegment::RecursiveDescent(None)]);
        assert!(results.len() > 3); // Should find nodes at all levels
    }

    #[test]
    fn test_evaluate_recursive_descent_with_name() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let results = evaluator.evaluate(&[
            PathSegment::Root,
            PathSegment::RecursiveDescent(Some("name".to_string())),
        ]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value(), &JsonValue::String("test".to_string()));
    }

    #[test]
    fn test_evaluate_complex_path() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let results = evaluator.evaluate(&[
            PathSegment::Root,
            PathSegment::Child("items".to_string()),
            PathSegment::Index(0),
        ]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value(), &JsonValue::String("a".to_string()));
    }

    #[test]
    fn test_evaluate_negative_index() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let results = evaluator.evaluate(&[
            PathSegment::Root,
            PathSegment::Child("items".to_string()),
            PathSegment::Index(-1),
        ]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value(), &JsonValue::String("c".to_string()));
    }

    #[test]
    fn test_evaluate_slice() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let results = evaluator.evaluate(&[
            PathSegment::Root,
            PathSegment::Child("items".to_string()),
            PathSegment::Slice(Some(0), Some(2)),
        ]);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].value(), &JsonValue::String("a".to_string()));
        assert_eq!(results[1].value(), &JsonValue::String("b".to_string()));
    }

    #[test]
    fn test_evaluate_no_match() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let results = evaluator.evaluate(&[
            PathSegment::Root,
            PathSegment::Child("nonexistent".to_string()),
        ]);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_evaluate_multi_property() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        let results = evaluator.evaluate(&[
            PathSegment::Root,
            PathSegment::MultiProperty(vec!["name".to_string(), "age".to_string()]),
        ]);
        assert_eq!(results.len(), 2); // Should find both name and age
                                      // Verify we got the right values
        assert_eq!(results[0].value(), &JsonValue::String("test".to_string()));
        assert_eq!(results[1].value(), &JsonValue::Number(42.0));
    }

    #[test]
    fn test_evaluate_current() {
        let tree = make_test_tree();
        let evaluator = Evaluator::new(&tree);
        // Current returns the same node
        let results = evaluator.evaluate_segment(&tree, &PathSegment::Current);
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].value(), JsonValue::Object(_)));
    }
}
