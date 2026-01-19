//! Tree view data structures for displaying JSON as an expandable tree.
//!
//! This module provides:
//! - `TreeViewLine`: A single displayable line in the tree view
//! - `ValueType`: Classification of JSON value types
//! - `TreeViewState`: Manages the list of visible lines and expand/collapse state

use crate::document::node::{JsonNode, JsonValue};
use crate::document::tree::JsonTree;
use std::collections::HashSet;

/// Represents a single line in the tree view display.
///
/// Each line corresponds to a JSON value at a specific path in the tree,
/// with information about how to display it (depth, key, preview, etc.).
#[derive(Debug, Clone)]
pub struct TreeViewLine {
    /// Path to this node in the JSON tree (indices at each level)
    pub path: Vec<usize>,
    /// Indentation depth (0 for root level)
    pub depth: usize,
    /// Object key name (None for array elements)
    pub key: Option<String>,
    /// Type of the JSON value
    pub value_type: ValueType,
    /// Short preview of the value (e.g., "{ 3 fields }" or "\"Alice\"")
    pub value_preview: String,
    /// Whether this value can be expanded (object/array)
    pub expandable: bool,
    /// Whether this value is currently expanded
    pub expanded: bool,
}

/// Classification of JSON value types for display purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    /// JSON object
    Object,
    /// JSON array
    Array,
    /// JSON string
    String,
    /// JSON number
    Number,
    /// JSON boolean
    Boolean,
    /// JSON null
    Null,
}

impl ValueType {
    /// Determines the value type from a JsonValue.
    ///
    /// # Example
    ///
    /// ```
    /// use jeditor::document::node::JsonValue;
    /// use jeditor::ui::tree_view::ValueType;
    ///
    /// let value = JsonValue::String("hello".to_string());
    /// assert_eq!(ValueType::from_json_value(&value), ValueType::String);
    /// ```
    pub fn from_json_value(value: &JsonValue) -> Self {
        match value {
            JsonValue::Object(_) => ValueType::Object,
            JsonValue::Array(_) => ValueType::Array,
            JsonValue::String(_) => ValueType::String,
            JsonValue::Number(_) => ValueType::Number,
            JsonValue::Boolean(_) => ValueType::Boolean,
            JsonValue::Null => ValueType::Null,
        }
    }
}

/// Manages the tree view display state and line generation.
///
/// The TreeViewState maintains:
/// - A list of visible lines (regenerated when expand/collapse state changes)
/// - A set of expanded node paths
///
/// # Example
///
/// ```
/// use jeditor::document::node::{JsonNode, JsonValue};
/// use jeditor::document::tree::JsonTree;
/// use jeditor::ui::tree_view::TreeViewState;
///
/// let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
///     ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
/// ])));
///
/// let mut state = TreeViewState::new();
/// state.rebuild(&tree);
/// assert_eq!(state.lines().len(), 1);
/// ```
pub struct TreeViewState {
    lines: Vec<TreeViewLine>,
    expanded_paths: HashSet<Vec<usize>>,
}

impl TreeViewState {
    /// Creates a new empty TreeViewState.
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            expanded_paths: HashSet::new(),
        }
    }

    /// Returns the list of visible tree view lines.
    pub fn lines(&self) -> &[TreeViewLine] {
        &self.lines
    }

    /// Toggles the expand/collapse state of a node at the given path.
    ///
    /// After toggling, call `rebuild()` to regenerate the visible lines.
    pub fn toggle_expand(&mut self, path: &[usize]) {
        if self.expanded_paths.contains(path) {
            self.expanded_paths.remove(path);
        } else {
            self.expanded_paths.insert(path.to_vec());
        }
    }

    /// Checks if a node at the given path is expanded.
    pub fn is_expanded(&self, path: &[usize]) -> bool {
        self.expanded_paths.contains(path)
    }

    /// Rebuilds the list of visible lines from the JSON tree.
    ///
    /// This should be called after the tree changes or expand/collapse state changes.
    pub fn rebuild(&mut self, tree: &JsonTree) {
        self.lines.clear();
        self.build_lines(tree.root(), &[], 0);
    }

    fn build_lines(&mut self, node: &JsonNode, path: &[usize], depth: usize) {
        match node.value() {
            JsonValue::Object(entries) => {
                for (i, (key, child)) in entries.iter().enumerate() {
                    let child_path: Vec<usize> = path.iter().copied().chain(std::iter::once(i)).collect();
                    let expanded = self.is_expanded(&child_path);

                    self.lines.push(TreeViewLine {
                        path: child_path.clone(),
                        depth,
                        key: Some(key.clone()),
                        value_type: ValueType::from_json_value(child.value()),
                        value_preview: self.get_value_preview(child.value()),
                        expandable: child.value().is_container(),
                        expanded,
                    });

                    if expanded && child.value().is_container() {
                        self.build_lines(child, &child_path, depth + 1);
                    }
                }
            }
            JsonValue::Array(elements) => {
                for (i, child) in elements.iter().enumerate() {
                    let child_path: Vec<usize> = path.iter().copied().chain(std::iter::once(i)).collect();
                    let expanded = self.is_expanded(&child_path);

                    self.lines.push(TreeViewLine {
                        path: child_path.clone(),
                        depth,
                        key: None,
                        value_type: ValueType::from_json_value(child.value()),
                        value_preview: self.get_value_preview(child.value()),
                        expandable: child.value().is_container(),
                        expanded,
                    });

                    if expanded && child.value().is_container() {
                        self.build_lines(child, &child_path, depth + 1);
                    }
                }
            }
            _ => {}
        }
    }

    fn get_value_preview(&self, value: &JsonValue) -> String {
        match value {
            JsonValue::Object(entries) => format!("{{ {} fields }}", entries.len()),
            JsonValue::Array(elements) => format!("[ {} items ]", elements.len()),
            JsonValue::String(s) => format!("\"{}\"", s),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::Boolean(b) => b.to_string(),
            JsonValue::Null => "null".to_string(),
        }
    }
}

impl Default for TreeViewState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_from_json() {
        assert_eq!(ValueType::from_json_value(&JsonValue::Object(vec![])), ValueType::Object);
        assert_eq!(ValueType::from_json_value(&JsonValue::Array(vec![])), ValueType::Array);
        assert_eq!(ValueType::from_json_value(&JsonValue::String("x".to_string())), ValueType::String);
        assert_eq!(ValueType::from_json_value(&JsonValue::Number(42.0)), ValueType::Number);
        assert_eq!(ValueType::from_json_value(&JsonValue::Boolean(true)), ValueType::Boolean);
        assert_eq!(ValueType::from_json_value(&JsonValue::Null), ValueType::Null);
    }

    #[test]
    fn test_tree_view_state_creation() {
        let state = TreeViewState::new();
        assert_eq!(state.lines().len(), 0);
    }

    #[test]
    fn test_rebuild_with_flat_object() {
        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
            ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
            ("age".to_string(), JsonNode::new(JsonValue::Number(30.0))),
        ])));

        let mut state = TreeViewState::new();
        state.rebuild(&tree);

        assert_eq!(state.lines().len(), 2);
        assert_eq!(state.lines()[0].key, Some("name".to_string()));
        assert_eq!(state.lines()[0].depth, 0);
        assert_eq!(state.lines()[1].key, Some("age".to_string()));
    }

    #[test]
    fn test_rebuild_with_array() {
        let tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
            JsonNode::new(JsonValue::Number(1.0)),
            JsonNode::new(JsonValue::Number(2.0)),
        ])));

        let mut state = TreeViewState::new();
        state.rebuild(&tree);

        assert_eq!(state.lines().len(), 2);
        assert_eq!(state.lines()[0].key, None);
        assert_eq!(state.lines()[0].value_preview, "1");
    }

    #[test]
    fn test_toggle_expand() {
        let mut state = TreeViewState::new();
        let path = vec![0];

        assert!(!state.is_expanded(&path));
        state.toggle_expand(&path);
        assert!(state.is_expanded(&path));
        state.toggle_expand(&path);
        assert!(!state.is_expanded(&path));
    }

    #[test]
    fn test_nested_object_collapsed() {
        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
            ("user".to_string(), JsonNode::new(JsonValue::Object(vec![
                ("name".to_string(), JsonNode::new(JsonValue::String("Bob".to_string()))),
            ]))),
        ])));

        let mut state = TreeViewState::new();
        state.rebuild(&tree);

        // Should only show the "user" field, not its children (not expanded)
        assert_eq!(state.lines().len(), 1);
        assert_eq!(state.lines()[0].key, Some("user".to_string()));
        assert_eq!(state.lines()[0].expandable, true);
    }

    #[test]
    fn test_nested_object_expanded() {
        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
            ("user".to_string(), JsonNode::new(JsonValue::Object(vec![
                ("name".to_string(), JsonNode::new(JsonValue::String("Bob".to_string()))),
            ]))),
        ])));

        let mut state = TreeViewState::new();
        state.toggle_expand(&[0]); // Expand "user"
        state.rebuild(&tree);

        // Should show both "user" and "user.name"
        assert_eq!(state.lines().len(), 2);
        assert_eq!(state.lines()[0].key, Some("user".to_string()));
        assert_eq!(state.lines()[0].depth, 0);
        assert_eq!(state.lines()[1].key, Some("name".to_string()));
        assert_eq!(state.lines()[1].depth, 1);
    }

    #[test]
    fn test_value_preview() {
        let state = TreeViewState::new();

        assert_eq!(state.get_value_preview(&JsonValue::Object(vec![("a".to_string(), JsonNode::new(JsonValue::Null))])), "{ 1 fields }");
        assert_eq!(state.get_value_preview(&JsonValue::Array(vec![JsonNode::new(JsonValue::Null), JsonNode::new(JsonValue::Null)])), "[ 2 items ]");
        assert_eq!(state.get_value_preview(&JsonValue::String("test".to_string())), "\"test\"");
        assert_eq!(state.get_value_preview(&JsonValue::Number(3.14)), "3.14");
        assert_eq!(state.get_value_preview(&JsonValue::Boolean(true)), "true");
        assert_eq!(state.get_value_preview(&JsonValue::Null), "null");
    }
}

use ratatui::{
    layout::Rect,
    style::{Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::theme::colors::ThemeColors;
use crate::editor::cursor::Cursor;

/// Renders the tree view with syntax highlighting and cursor.
///
/// Displays JSON tree as an expandable/collapsible list with:
/// - Indentation based on depth
/// - Expand/collapse indicators (▼/▶) for containers
/// - Syntax-highlighted keys and values
/// - Cursor highlight on the current line
///
/// # Arguments
///
/// * `f` - The ratatui frame to render into
/// * `area` - The rectangular area for the tree view
/// * `tree_view` - The tree view state with visible lines
/// * `cursor` - The cursor position
/// * `colors` - Theme colors for syntax highlighting
///
/// # Example
///
/// ```no_run
/// use jeditor::ui::tree_view::{render_tree_view, TreeViewState};
/// use jeditor::editor::cursor::Cursor;
/// use jeditor::theme::colors::ThemeColors;
/// use jeditor::document::node::{JsonNode, JsonValue};
/// use jeditor::document::tree::JsonTree;
/// use ratatui::backend::TestBackend;
/// use ratatui::Terminal;
/// use ratatui::layout::Rect;
///
/// let backend = TestBackend::new(80, 24);
/// let mut terminal = Terminal::new(backend).unwrap();
/// let colors = ThemeColors::default_dark();
/// let cursor = Cursor::new();
///
/// let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
///     ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
/// ])));
/// let mut tree_view = TreeViewState::new();
/// tree_view.rebuild(&tree);
///
/// terminal.draw(|f| {
///     render_tree_view(f, f.area(), &tree_view, &cursor, &colors);
/// }).unwrap();
/// ```
pub fn render_tree_view(
    f: &mut Frame,
    area: Rect,
    tree_view: &TreeViewState,
    cursor: &Cursor,
    colors: &ThemeColors,
) {
    let mut lines_to_render = Vec::new();

    for line in tree_view.lines().iter() {
        let is_cursor = cursor.path() == line.path.as_slice();

        let mut spans = Vec::new();

        // Indentation
        spans.push(Span::raw("  ".repeat(line.depth)));

        // Expand/collapse indicator
        if line.expandable {
            let indicator = if line.expanded { "▼ " } else { "▶ " };
            spans.push(Span::raw(indicator));
        } else {
            spans.push(Span::raw("  "));
        }

        // Key (if object property)
        if let Some(key) = &line.key {
            spans.push(Span::styled(
                format!("\"{}\": ", key),
                Style::default().fg(colors.key),
            ));
        }

        // Value
        let value_color = match line.value_type {
            ValueType::String => colors.string,
            ValueType::Number => colors.number,
            ValueType::Boolean => colors.boolean,
            ValueType::Null => colors.null,
            ValueType::Object | ValueType::Array => colors.foreground,
        };

        spans.push(Span::styled(
            &line.value_preview,
            Style::default().fg(value_color),
        ));

        let mut style = Style::default();
        if is_cursor {
            style = style.bg(colors.cursor).add_modifier(Modifier::BOLD);
        }

        lines_to_render.push(Line::from(spans).style(style));
    }

    let paragraph = Paragraph::new(lines_to_render)
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().bg(colors.background).fg(colors.foreground));

    f.render_widget(paragraph, area);
}
