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
    /// use jsonquill::document::node::JsonValue;
    /// use jsonquill::ui::tree_view::ValueType;
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
            JsonValue::JsonlRoot(_) => ValueType::Array, // Treat JSONL root like array for display
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
/// use jsonquill::document::node::{JsonNode, JsonValue};
/// use jsonquill::document::tree::JsonTree;
/// use jsonquill::ui::tree_view::TreeViewState;
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

    /// Expands a specific node and all its descendants.
    ///
    /// This is used when expanding JSONL lines to show the entire tree within the line.
    pub fn expand_node_and_descendants(&mut self, tree: &JsonTree, path: &[usize]) {
        // First expand the node itself
        self.expanded_paths.insert(path.to_vec());

        // Then expand all descendants
        if let Some(node) = tree.get_node(path) {
            self.expand_all_recursive(node, path);
        }
    }

    /// Collapses a specific node and all its descendants.
    ///
    /// Removes the expansion state for the node and all paths underneath it.
    pub fn collapse_node_and_descendants(&mut self, tree: &JsonTree, path: &[usize]) {
        // First collapse the node itself
        self.expanded_paths.remove(path);

        // Then collapse all descendants
        if let Some(node) = tree.get_node(path) {
            self.collapse_all_recursive(node, path);
        }
    }

    /// Rebuilds the list of visible lines from the JSON tree.
    ///
    /// This should be called after the tree changes or expand/collapse state changes.
    pub fn rebuild(&mut self, tree: &JsonTree) {
        self.lines.clear();

        // Handle JSONL root specially - render as flat list
        match tree.root().value() {
            JsonValue::JsonlRoot(lines) => {
                self.render_jsonl_root(lines);
            }
            _ => {
                self.build_lines(tree.root(), &[], 0);
            }
        }
    }

    /// Renders JSONL root as a flat list of collapsed lines.
    ///
    /// Each line in the JSONL document is shown at depth 0, collapsed by default.
    /// Users can expand individual lines to see their contents.
    fn render_jsonl_root(&mut self, lines: &[JsonNode]) {
        for (idx, node) in lines.iter().enumerate() {
            let path = vec![idx];
            let is_expanded = self.is_expanded(&path);

            // Show collapsed preview for the line itself
            let preview = format_collapsed_preview(node, 60);
            self.lines.push(TreeViewLine {
                path: path.clone(),
                depth: 0,
                key: None,
                value_type: ValueType::from_json_value(node.value()),
                value_preview: preview,
                expandable: true,
                expanded: is_expanded,
            });

            // If expanded, render the contents of the line
            if is_expanded {
                self.build_lines(node, &path, 1);
            }
        }
    }

    /// Expands all container nodes (objects and arrays) in the tree.
    ///
    /// This is typically called when initially loading a file to show
    /// the full structure. After calling this, call `rebuild()` to
    /// regenerate the visible lines.
    pub fn expand_all(&mut self, tree: &JsonTree) {
        self.expand_all_recursive(tree.root(), &[]);
    }

    fn expand_all_recursive(&mut self, node: &JsonNode, path: &[usize]) {
        match node.value() {
            JsonValue::Object(entries) => {
                for (i, (_, child)) in entries.iter().enumerate() {
                    let child_path: Vec<usize> =
                        path.iter().copied().chain(std::iter::once(i)).collect();
                    if child.value().is_container() {
                        self.expanded_paths.insert(child_path.clone());
                        self.expand_all_recursive(child, &child_path);
                    }
                }
            }
            JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
                for (i, child) in elements.iter().enumerate() {
                    let child_path: Vec<usize> =
                        path.iter().copied().chain(std::iter::once(i)).collect();
                    if child.value().is_container() {
                        self.expanded_paths.insert(child_path.clone());
                        self.expand_all_recursive(child, &child_path);
                    }
                }
            }
            _ => {}
        }
    }

    fn collapse_all_recursive(&mut self, node: &JsonNode, path: &[usize]) {
        match node.value() {
            JsonValue::Object(entries) => {
                for (i, (_, child)) in entries.iter().enumerate() {
                    let child_path: Vec<usize> =
                        path.iter().copied().chain(std::iter::once(i)).collect();
                    if child.value().is_container() {
                        self.expanded_paths.remove(&child_path);
                        self.collapse_all_recursive(child, &child_path);
                    }
                }
            }
            JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
                for (i, child) in elements.iter().enumerate() {
                    let child_path: Vec<usize> =
                        path.iter().copied().chain(std::iter::once(i)).collect();
                    if child.value().is_container() {
                        self.expanded_paths.remove(&child_path);
                        self.collapse_all_recursive(child, &child_path);
                    }
                }
            }
            _ => {}
        }
    }

    fn build_lines(&mut self, node: &JsonNode, path: &[usize], depth: usize) {
        match node.value() {
            JsonValue::Object(entries) => {
                for (i, (key, child)) in entries.iter().enumerate() {
                    let child_path: Vec<usize> =
                        path.iter().copied().chain(std::iter::once(i)).collect();
                    let expanded = self.is_expanded(&child_path);

                    // Always use collapsed preview for containers
                    let value_preview = if child.value().is_container() {
                        format_collapsed_preview(child, 60)
                    } else {
                        self.get_value_preview(child.value())
                    };

                    self.lines.push(TreeViewLine {
                        path: child_path.clone(),
                        depth,
                        key: Some(key.clone()),
                        value_type: ValueType::from_json_value(child.value()),
                        value_preview,
                        expandable: child.value().is_container(),
                        expanded,
                    });

                    if expanded && child.value().is_container() {
                        self.build_lines(child, &child_path, depth + 1);
                    }
                }
            }
            JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
                for (i, child) in elements.iter().enumerate() {
                    let child_path: Vec<usize> =
                        path.iter().copied().chain(std::iter::once(i)).collect();
                    let expanded = self.is_expanded(&child_path);

                    // Always use collapsed preview for containers
                    let value_preview = if child.value().is_container() {
                        format_collapsed_preview(child, 60)
                    } else {
                        self.get_value_preview(child.value())
                    };

                    self.lines.push(TreeViewLine {
                        path: child_path.clone(),
                        depth,
                        key: Some(format!("[{}]", i)),
                        value_type: ValueType::from_json_value(child.value()),
                        value_preview,
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
            JsonValue::Array(elements) | JsonValue::JsonlRoot(elements) => {
                format!("[ {} items ]", elements.len())
            }
            JsonValue::String(s) => format!("\"{}\"", s),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::Boolean(b) => b.to_string(),
            JsonValue::Null => "null".to_string(),
        }
    }

    /// Updates expanded paths after inserting a node at the given path.
    ///
    /// When a node is inserted, all subsequent siblings and their descendants
    /// need their indices shifted by 1. This method updates the expanded_paths
    /// set to reflect the new indices after insertion.
    ///
    /// # Arguments
    /// * `insertion_path` - The path where the new node was inserted
    pub fn update_paths_after_insertion(&mut self, insertion_path: &[usize]) {
        if insertion_path.is_empty() {
            return;
        }

        let parent_path = &insertion_path[..insertion_path.len() - 1];
        let insertion_idx = insertion_path[insertion_path.len() - 1];

        // Collect paths that need updating
        let paths_to_update: Vec<Vec<usize>> = self
            .expanded_paths
            .iter()
            .filter(|path| {
                // Check if this path is affected by the insertion
                if path.len() < parent_path.len() + 1 {
                    return false;
                }

                // Check if path has the same parent
                if &path[..parent_path.len()] != parent_path {
                    return false;
                }

                // Check if the index at the insertion level is >= insertion_idx
                path[parent_path.len()] >= insertion_idx
            })
            .cloned()
            .collect();

        // IMPORTANT: Remove ALL old paths first, then insert ALL new paths
        // This prevents collisions where a new path matches an old path that hasn't been updated yet
        // Example: when inserting at [0, 1], both [0, 3] and [0, 4] shift to [0, 4] and [0, 5]
        // If we remove+insert one at a time, [0, 3]→[0, 4] would collide with existing [0, 4]

        // First pass: remove all old paths
        for old_path in &paths_to_update {
            self.expanded_paths.remove(old_path);
        }

        // Second pass: insert all new paths
        for old_path in paths_to_update {
            let mut new_path = old_path.clone();
            new_path[parent_path.len()] += 1;
            self.expanded_paths.insert(new_path);
        }
    }

    /// Updates expanded_paths after a node deletion.
    ///
    /// When a node is deleted, all subsequent siblings and their descendants
    /// need their indices shifted down by 1. This method updates the expanded_paths
    /// set to reflect the new indices after deletion. Also removes the deleted path
    /// and all its descendants from expanded_paths.
    ///
    /// # Arguments
    /// * `deletion_path` - The path of the node that was deleted
    pub fn update_paths_after_deletion(&mut self, deletion_path: &[usize]) {
        if deletion_path.is_empty() {
            return;
        }

        let parent_path = &deletion_path[..deletion_path.len() - 1];
        let deletion_idx = deletion_path[deletion_path.len() - 1];

        // Collect paths to remove (deleted node and its descendants)
        let paths_to_remove: Vec<Vec<usize>> = self
            .expanded_paths
            .iter()
            .filter(|path| {
                // Remove if this is the deleted node or a descendant
                path.len() >= deletion_path.len() && &path[..deletion_path.len()] == deletion_path
            })
            .cloned()
            .collect();

        // Collect paths that need index shifting (subsequent siblings and descendants)
        let paths_to_update: Vec<Vec<usize>> = self
            .expanded_paths
            .iter()
            .filter(|path| {
                // Check if this path is affected by the deletion
                if path.len() < parent_path.len() + 1 {
                    return false;
                }

                // Check if path has the same parent
                if &path[..parent_path.len()] != parent_path {
                    return false;
                }

                // Check if the index at the deletion level is > deletion_idx
                // (nodes after the deleted node need to shift down)
                path[parent_path.len()] > deletion_idx
            })
            .cloned()
            .collect();

        // First pass: remove deleted node and its descendants
        for path in &paths_to_remove {
            self.expanded_paths.remove(path);
        }

        // Second pass: remove all old paths that need shifting
        for old_path in &paths_to_update {
            self.expanded_paths.remove(old_path);
        }

        // Third pass: insert all shifted paths
        for old_path in paths_to_update {
            let mut new_path = old_path.clone();
            new_path[parent_path.len()] -= 1;
            self.expanded_paths.insert(new_path);
        }
    }
}

impl Default for TreeViewState {
    fn default() -> Self {
        Self::new()
    }
}

use crate::editor::cursor::Cursor;
use crate::theme::colors::ThemeColors;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

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
/// use jsonquill::ui::tree_view::{render_tree_view, TreeViewState};
/// use jsonquill::editor::cursor::Cursor;
/// use jsonquill::theme::colors::ThemeColors;
/// use jsonquill::document::node::{JsonNode, JsonValue};
/// use jsonquill::document::tree::JsonTree;
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
///     render_tree_view(f, f.area(), &tree_view, &cursor, &colors, true, false, 0, &[]);
/// }).unwrap();
/// ```
#[allow(clippy::too_many_arguments)]
pub fn render_tree_view(
    f: &mut Frame,
    area: Rect,
    tree_view: &TreeViewState,
    cursor: &Cursor,
    colors: &ThemeColors,
    show_line_numbers: bool,
    relative_line_numbers: bool,
    scroll_offset: usize,
    visual_selection: &[Vec<usize>],
) {
    let mut lines_to_render = Vec::new();
    let max_line_num_width = if show_line_numbers {
        tree_view.lines().len().to_string().len()
    } else {
        0
    };

    // Find the cursor line number for relative numbering
    let cursor_line_num = tree_view
        .lines()
        .iter()
        .position(|l| l.path == cursor.path())
        .unwrap_or(0);

    let viewport_height = area.height as usize;

    for (line_num, line) in tree_view
        .lines()
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(viewport_height)
    {
        let is_cursor = cursor.path() == line.path.as_slice();
        let is_selected = visual_selection
            .iter()
            .any(|selected_path| selected_path == &line.path);

        let mut spans = Vec::new();

        // Line number
        if show_line_numbers {
            let display_num = if relative_line_numbers {
                if is_cursor {
                    // Show absolute line number on cursor line
                    line_num + 1
                } else {
                    // Show relative distance from cursor
                    line_num.abs_diff(cursor_line_num)
                }
            } else {
                // Absolute line numbers
                line_num + 1
            };

            let line_num_str = format!("{:>width$} ", display_num, width = max_line_num_width);
            spans.push(Span::styled(
                line_num_str,
                Style::default()
                    .fg(colors.foreground)
                    .add_modifier(Modifier::DIM),
            ));
        }

        // Indentation
        spans.push(Span::raw("  ".repeat(line.depth)));

        // Expand/collapse indicator or cursor indicator for scalars
        if line.expandable {
            let indicator = if line.expanded { "▼ " } else { "▶ " };
            spans.push(Span::raw(indicator));
        } else if is_cursor {
            spans.push(Span::raw("▶ "));
        } else {
            spans.push(Span::raw("  "));
        }

        // Key (if object property) - highlight only the key when cursor is on this line
        if let Some(key) = &line.key {
            let key_style = if is_cursor {
                // White text on cursor background for readability
                Style::default()
                    .fg(Color::White)
                    .bg(colors.cursor)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.key)
            };
            spans.push(Span::styled(format!("{}: ", key), key_style));
        }

        // Value - highlight when cursor is on this line
        let value_style = if is_cursor {
            // All values on cursor line get same highlight as keys for consistent visibility
            Style::default()
                .fg(Color::White)
                .bg(colors.cursor)
                .add_modifier(Modifier::BOLD)
        } else {
            let value_color = if line.expandable {
                // All containers use preview color (they show collapsed preview format)
                colors.preview
            } else {
                // Scalars use their type-specific colors
                match line.value_type {
                    ValueType::String => colors.string,
                    ValueType::Number => colors.number,
                    ValueType::Boolean => colors.boolean,
                    ValueType::Null => colors.null,
                    ValueType::Object | ValueType::Array => colors.foreground,
                }
            };
            Style::default().fg(value_color)
        };

        spans.push(Span::styled(&line.value_preview, value_style));

        // Apply visual selection background if this line is selected
        let final_line = if is_selected {
            Line::from(
                spans
                    .into_iter()
                    .map(|span| {
                        Span::styled(span.content, span.style.bg(colors.visual_selection_bg))
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            Line::from(spans)
        };

        lines_to_render.push(final_line);
    }

    let paragraph = Paragraph::new(lines_to_render)
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().bg(colors.background).fg(colors.foreground));

    f.render_widget(paragraph, area);
}

/// Formats a number as an integer if it has no fractional part, otherwise as a float.
fn format_number(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

/// Formats a collapsed preview of a JSON node similar to jless.
///
/// Format: (N) {key1: val1, key2: val2, ...} for objects
///         (N) [elem1, elem2, ...] for arrays
///
/// Truncates at max_chars with "..." if needed.
pub fn format_collapsed_preview(node: &JsonNode, max_chars: usize) -> String {
    match node.value() {
        JsonValue::Object(fields) => format_collapsed_object(fields, max_chars),
        JsonValue::Array(elements) => format_collapsed_array(elements, max_chars),
        JsonValue::JsonlRoot(lines) => {
            // Shouldn't happen, but treat like array
            format_collapsed_array(lines, max_chars)
        }
        JsonValue::String(s) => format!("\"{}\"", s),
        JsonValue::Number(n) => format_number(*n),
        JsonValue::Boolean(b) => format!("{}", b),
        JsonValue::Null => "null".to_string(),
    }
}

fn format_collapsed_object(fields: &[(String, JsonNode)], max_chars: usize) -> String {
    if fields.is_empty() {
        return "{…}".to_string();
    }

    let count = fields.len();
    let mut preview = format!("({}) {{", count);
    let mut truncated = false;

    for (i, (key, value)) in fields.iter().enumerate() {
        // Check if we need to truncate (leave room for "..." and "}")
        if preview.len() + key.len() + 10 > max_chars {
            preview.push_str("...");
            truncated = true;
            break;
        }

        // Add key
        preview.push_str(key);
        preview.push_str(": ");

        // Add value
        let value_str = match value.value() {
            JsonValue::Object(_) => "{…}".to_string(),
            JsonValue::Array(_) | JsonValue::JsonlRoot(_) => "[…]".to_string(),
            JsonValue::String(s) => {
                let quoted = format!("\"{}\"", s);
                if preview.len() + quoted.len() > max_chars {
                    // Use char-based truncation to avoid UTF-8 boundary panics
                    let truncated: String = s.chars().take(10).collect();
                    format!("\"{}...\"", truncated)
                } else {
                    quoted
                }
            }
            JsonValue::Number(n) => format_number(*n),
            JsonValue::Boolean(b) => format!("{}", b),
            JsonValue::Null => "null".to_string(),
        };

        preview.push_str(&value_str);

        // Add comma if not last
        if i < fields.len() - 1 {
            preview.push_str(", ");
        }
    }

    // Close brace if we didn't truncate
    if !truncated {
        preview.push('}');
    }

    preview
}

fn format_collapsed_array(elements: &[JsonNode], max_chars: usize) -> String {
    if elements.is_empty() {
        return "[…]".to_string();
    }

    let count = elements.len();
    let mut preview = format!("({}) [", count);
    let mut truncated = false;

    for (i, element) in elements.iter().enumerate() {
        // Check if we need to truncate (leave room for "..." and "]")
        if preview.len() + 10 > max_chars {
            preview.push_str("...");
            truncated = true;
            break;
        }

        let value_str = match element.value() {
            JsonValue::Object(_) => "{…}".to_string(),
            JsonValue::Array(_) | JsonValue::JsonlRoot(_) => "[…]".to_string(),
            JsonValue::String(s) => {
                let quoted = format!("\"{}\"", s);
                // Check length to avoid exceeding max_chars with long strings
                if preview.len() + quoted.len() > max_chars {
                    // Use char-based truncation to avoid UTF-8 boundary panics
                    let truncated_str: String = s.chars().take(10).collect();
                    format!("\"{}...\"", truncated_str)
                } else {
                    quoted
                }
            }
            JsonValue::Number(n) => format_number(*n),
            JsonValue::Boolean(b) => format!("{}", b),
            JsonValue::Null => "null".to_string(),
        };

        preview.push_str(&value_str);

        if i < elements.len() - 1 {
            preview.push_str(", ");
        }
    }

    if !truncated {
        preview.push(']');
    }

    preview
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_from_json() {
        assert_eq!(
            ValueType::from_json_value(&JsonValue::Object(vec![])),
            ValueType::Object
        );
        assert_eq!(
            ValueType::from_json_value(&JsonValue::Array(vec![])),
            ValueType::Array
        );
        assert_eq!(
            ValueType::from_json_value(&JsonValue::String("x".to_string())),
            ValueType::String
        );
        assert_eq!(
            ValueType::from_json_value(&JsonValue::Number(42.0)),
            ValueType::Number
        );
        assert_eq!(
            ValueType::from_json_value(&JsonValue::Boolean(true)),
            ValueType::Boolean
        );
        assert_eq!(
            ValueType::from_json_value(&JsonValue::Null),
            ValueType::Null
        );
    }

    #[test]
    fn test_tree_view_state_creation() {
        let state = TreeViewState::new();
        assert_eq!(state.lines().len(), 0);
    }

    #[test]
    fn test_rebuild_with_flat_object() {
        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String("Alice".to_string())),
            ),
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
        assert_eq!(state.lines()[0].key, Some("[0]".to_string()));
        assert_eq!(state.lines()[0].value_preview, "1");
        assert_eq!(state.lines()[1].key, Some("[1]".to_string()));
        assert_eq!(state.lines()[1].value_preview, "2");
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
        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
            "user".to_string(),
            JsonNode::new(JsonValue::Object(vec![(
                "name".to_string(),
                JsonNode::new(JsonValue::String("Bob".to_string())),
            )])),
        )])));

        let mut state = TreeViewState::new();
        state.rebuild(&tree);

        // Should only show the "user" field, not its children (not expanded)
        assert_eq!(state.lines().len(), 1);
        assert_eq!(state.lines()[0].key, Some("user".to_string()));
        assert!(state.lines()[0].expandable);
    }

    #[test]
    fn test_nested_object_expanded() {
        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
            "user".to_string(),
            JsonNode::new(JsonValue::Object(vec![(
                "name".to_string(),
                JsonNode::new(JsonValue::String("Bob".to_string())),
            )])),
        )])));

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

        // Scalars still use simple format
        assert_eq!(
            state.get_value_preview(&JsonValue::String("test".to_string())),
            "\"test\""
        );
        assert_eq!(
            state.get_value_preview(&JsonValue::Number(std::f64::consts::PI)),
            std::f64::consts::PI.to_string()
        );
        assert_eq!(state.get_value_preview(&JsonValue::Boolean(true)), "true");
        assert_eq!(state.get_value_preview(&JsonValue::Null), "null");

        // Containers are now handled by format_collapsed_preview in build_lines
        // so get_value_preview is only used for expanded containers (which show nothing)
        // or for scalars
    }

    #[test]
    fn test_key_display_without_quotes() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String("Alice".to_string())),
            ),
            ("age".to_string(), JsonNode::new(JsonValue::Number(30.0))),
        ])));

        let mut state = TreeViewState::new();
        state.rebuild(&tree);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let colors = ThemeColors::default_dark();
        let cursor = Cursor::new();

        terminal
            .draw(|f| {
                render_tree_view(f, f.area(), &state, &cursor, &colors, false, false, 0, &[]);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let content = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        // Keys should be displayed without quotes: "name: " not "\"name\": "
        assert!(
            content.contains("name: "),
            "Expected 'name: ' without quotes in rendered output"
        );
        assert!(
            content.contains("age: "),
            "Expected 'age: ' without quotes in rendered output"
        );
        assert!(
            !content.contains("\"name\""),
            "Keys should not have quotes in rendered output"
        );
        assert!(
            !content.contains("\"age\""),
            "Keys should not have quotes in rendered output"
        );
    }

    #[test]
    fn test_cursor_highlights_entire_line() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
            "name".to_string(),
            JsonNode::new(JsonValue::String("Alice".to_string())),
        )])));

        let mut state = TreeViewState::new();
        state.rebuild(&tree);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let colors = ThemeColors::default_dark();
        let mut cursor = Cursor::new();
        cursor.set_path(vec![0]); // Select first item

        terminal
            .draw(|f| {
                render_tree_view(f, f.area(), &state, &cursor, &colors, false, false, 0, &[]);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();

        // Check cells on the first line
        // Expected layout: "▶ name: \"Alice\""
        // Both "name:" and "\"Alice\"" should be highlighted with cursor background
        let mut found_key_highlight = false;
        let mut found_key_white_text = false;
        let mut found_value_highlight = false;

        for (i, cell) in buffer.content().iter().enumerate() {
            let symbol = cell.symbol();
            // Look for the 'n' in 'name'
            if symbol == "n" && i > 0 {
                // This should be part of the key and should have cursor background
                if cell.bg == colors.cursor {
                    found_key_highlight = true;
                }
                // Key text should be white for visibility
                if cell.fg == Color::White {
                    found_key_white_text = true;
                }
            }
            // Look for the 'A' in 'Alice'
            if symbol == "A" {
                // This should be part of the value and SHOULD have cursor background
                if cell.bg == colors.cursor {
                    found_value_highlight = true;
                }
            }
        }

        assert!(
            found_key_highlight,
            "Key 'name:' should be highlighted with cursor background"
        );
        assert!(
            found_key_white_text,
            "Key 'name:' text should be white for visibility"
        );
        assert!(
            found_value_highlight,
            "Value '\"Alice\"' should be highlighted with cursor background"
        );
    }

    #[test]
    fn test_scalar_value_shows_triangle_indicator() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![(
            "name".to_string(),
            JsonNode::new(JsonValue::String("Alice".to_string())),
        )])));

        let mut state = TreeViewState::new();
        state.rebuild(&tree);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let colors = ThemeColors::default_dark();
        let mut cursor = Cursor::new();
        cursor.set_path(vec![0]); // Select first item

        terminal
            .draw(|f| {
                render_tree_view(f, f.area(), &state, &cursor, &colors, false, false, 0, &[]);
            })
            .unwrap();

        let buffer = terminal.backend().buffer().clone();
        let content = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        // Should contain a right-pointing triangle (▶) on the left side (before the key) for selected scalar
        // Expected layout: "▶ name: \"Alice\""
        // The triangle should appear before the key, not after the value
        let first_line = content.lines().next().unwrap_or("");
        assert!(
            first_line.contains("▶"),
            "Expected triangle indicator on left side for selected scalar value"
        );

        // Verify triangle comes before the key
        if let Some(triangle_pos) = first_line.find("▶") {
            if let Some(key_pos) = first_line.find("name") {
                assert!(
                    triangle_pos < key_pos,
                    "Triangle should appear before the key on the left side"
                );
            }
        }
    }

    #[test]
    fn test_format_collapsed_preview_simple_object() {
        use crate::document::node::{JsonNode, JsonValue};

        let obj = JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(1.0))),
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String("Alice".to_string())),
            ),
        ]));

        let preview = format_collapsed_preview(&obj, 100);
        assert_eq!(preview, "(2) {id: 1, name: \"Alice\"}");
    }

    #[test]
    fn test_format_collapsed_preview_nested_object() {
        use crate::document::node::{JsonNode, JsonValue};

        let obj = JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(1.0))),
            (
                "user".to_string(),
                JsonNode::new(JsonValue::Object(vec![(
                    "name".to_string(),
                    JsonNode::new(JsonValue::String("Alice".to_string())),
                )])),
            ),
        ]));

        let preview = format_collapsed_preview(&obj, 100);
        assert_eq!(preview, "(2) {id: 1, user: {…}}");
    }

    #[test]
    fn test_format_collapsed_preview_array() {
        use crate::document::node::{JsonNode, JsonValue};

        let arr = JsonNode::new(JsonValue::Array(vec![
            JsonNode::new(JsonValue::Number(1.0)),
            JsonNode::new(JsonValue::Number(2.0)),
            JsonNode::new(JsonValue::Number(3.0)),
        ]));

        let preview = format_collapsed_preview(&arr, 100);
        assert_eq!(preview, "(3) [1, 2, 3]");
    }

    #[test]
    fn test_format_collapsed_preview_truncation() {
        use crate::document::node::{JsonNode, JsonValue};

        let obj = JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(1.0))),
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String("Alice".to_string())),
            ),
            (
                "email".to_string(),
                JsonNode::new(JsonValue::String("alice@example.com".to_string())),
            ),
            (
                "active".to_string(),
                JsonNode::new(JsonValue::Boolean(true)),
            ),
        ]));

        let preview = format_collapsed_preview(&obj, 40);
        assert!(preview.len() <= 43); // Allow a bit of overflow for "..."
        assert!(preview.contains("..."));
    }

    #[test]
    fn test_format_collapsed_preview_utf8_truncation() {
        use crate::document::node::{JsonNode, JsonValue};

        // Test with multi-byte UTF-8 characters (emoji, Chinese, etc.)
        let obj = JsonNode::new(JsonValue::Object(vec![
            (
                "emoji".to_string(),
                JsonNode::new(JsonValue::String("🌟✨🎉🎊🎈".to_string())),
            ),
            (
                "chinese".to_string(),
                JsonNode::new(JsonValue::String(
                    "你好世界这是一个很长的字符串".to_string(),
                )),
            ),
        ]));

        // This should not panic even with multi-byte characters
        let preview = format_collapsed_preview(&obj, 40);
        assert!(preview.contains("..."));

        // Test array with UTF-8 strings
        let arr = JsonNode::new(JsonValue::Array(vec![
            JsonNode::new(JsonValue::String("🌟✨🎉🎊🎈🎁🎀🎂".to_string())),
            JsonNode::new(JsonValue::String(
                "你好世界这是一个很长的字符串".to_string(),
            )),
        ]));

        // This should also not panic
        let preview = format_collapsed_preview(&arr, 40);
        assert!(preview.contains("..."));
    }

    #[test]
    fn test_render_jsonl_root() {
        use crate::document::node::{JsonNode, JsonValue};
        use crate::document::tree::JsonTree;

        let lines = vec![
            JsonNode::new(JsonValue::Object(vec![(
                "id".to_string(),
                JsonNode::new(JsonValue::Number(1.0)),
            )])),
            JsonNode::new(JsonValue::Object(vec![(
                "id".to_string(),
                JsonNode::new(JsonValue::Number(2.0)),
            )])),
        ];

        let tree = JsonTree::new(JsonNode::new(JsonValue::JsonlRoot(lines)));
        let mut state = TreeViewState::new();
        state.rebuild(&tree);

        // Should have 2 lines, both collapsed
        assert_eq!(state.lines().len(), 2);
        assert!(state.lines()[0].value_preview.contains("id: 1"));
        assert_eq!(state.lines()[0].depth, 0);
        assert!(state.lines()[0].expandable);
        assert!(!state.lines()[0].expanded);
    }

    #[test]
    fn test_expand_jsonl_line() {
        use crate::document::node::{JsonNode, JsonValue};
        use crate::document::tree::JsonTree;

        let lines = vec![JsonNode::new(JsonValue::Object(vec![
            ("id".to_string(), JsonNode::new(JsonValue::Number(1.0))),
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String("Alice".to_string())),
            ),
        ]))];

        let tree = JsonTree::new(JsonNode::new(JsonValue::JsonlRoot(lines)));
        let mut state = TreeViewState::new();
        state.rebuild(&tree);

        // Initially collapsed
        assert_eq!(state.lines().len(), 1);

        // Expand first line
        state.toggle_expand(&[0]);
        state.rebuild(&tree);

        // Should now show 3 lines: object + 2 fields
        assert!(state.lines().len() > 1);
        assert_eq!(state.lines()[0].depth, 0); // The JSONL line itself
        assert!(state.lines()[0].expanded);
    }

    #[test]
    fn test_update_paths_after_deletion() {
        use crate::document::node::{JsonNode, JsonValue};
        use crate::document::tree::JsonTree;

        // Create array with 4 objects
        let _tree = JsonTree::new(JsonNode::new(JsonValue::Array(vec![
            JsonNode::new(JsonValue::Object(vec![(
                "name".to_string(),
                JsonNode::new(JsonValue::String("Alice".to_string())),
            )])),
            JsonNode::new(JsonValue::Object(vec![(
                "name".to_string(),
                JsonNode::new(JsonValue::String("Bob".to_string())),
            )])),
            JsonNode::new(JsonValue::Object(vec![(
                "name".to_string(),
                JsonNode::new(JsonValue::String("Charlie".to_string())),
            )])),
            JsonNode::new(JsonValue::Object(vec![(
                "name".to_string(),
                JsonNode::new(JsonValue::String("Dave".to_string())),
            )])),
        ])));

        let mut state = TreeViewState::new();

        // Expand objects at [0, 1], [0, 2], and [0, 3]
        state.toggle_expand(&[0, 1]);
        state.toggle_expand(&[0, 2]);
        state.toggle_expand(&[0, 3]);

        assert!(state.is_expanded(&[0, 1]));
        assert!(state.is_expanded(&[0, 2]));
        assert!(state.is_expanded(&[0, 3]));

        // Delete node at [0, 1] (Bob)
        state.update_paths_after_deletion(&[0, 1]);

        // [0, 2] (Charlie) should now be at [0, 1] and still expanded
        assert!(state.is_expanded(&[0, 1]));

        // [0, 3] should now be at [0, 2] (Dave shifted down)
        assert!(state.is_expanded(&[0, 2]));

        // Old paths should not exist
        assert!(!state.is_expanded(&[0, 3]));
    }

    #[test]
    fn test_deletion_preserves_sibling_expansion() {
        use crate::document::node::{JsonNode, JsonValue};
        use crate::document::tree::JsonTree;

        // Create nested structure where we'll delete one object but others should stay expanded
        let mut tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![
            (
                "item1".to_string(),
                JsonNode::new(JsonValue::Object(vec![(
                    "nested".to_string(),
                    JsonNode::new(JsonValue::Number(1.0)),
                )])),
            ),
            (
                "item2".to_string(),
                JsonNode::new(JsonValue::Object(vec![(
                    "nested".to_string(),
                    JsonNode::new(JsonValue::Number(2.0)),
                )])),
            ),
            (
                "item3".to_string(),
                JsonNode::new(JsonValue::Object(vec![(
                    "nested".to_string(),
                    JsonNode::new(JsonValue::Number(3.0)),
                )])),
            ),
        ])));

        let mut state = TreeViewState::new();

        // Expand root and all items
        state.toggle_expand(&[]);
        state.toggle_expand(&[0]);
        state.toggle_expand(&[1]);
        state.toggle_expand(&[2]);
        state.rebuild(&tree);

        // Verify all are expanded
        assert!(state.is_expanded(&[]));
        assert!(state.is_expanded(&[0]));
        assert!(state.is_expanded(&[1]));
        assert!(state.is_expanded(&[2]));

        // Delete item2 at index [1]
        tree.delete_node(&[1]).unwrap();
        state.update_paths_after_deletion(&[1]);
        state.rebuild(&tree);

        // Root should still be expanded
        assert!(state.is_expanded(&[]));

        // item1 at [0] should still be expanded (unaffected)
        assert!(state.is_expanded(&[0]));

        // item3 was at [2], now should be at [1] and still expanded
        assert!(state.is_expanded(&[1]));

        // Old path [2] should not be expanded
        assert!(!state.is_expanded(&[2]));
    }
}
