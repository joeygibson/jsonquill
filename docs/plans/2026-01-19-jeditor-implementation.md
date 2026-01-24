# jsonquill Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a terminal-based structural JSON editor in Rust with vim-style keybindings, syntax highlighting, and JSONL support.

**Architecture:** Multi-layered design with parser (serde_json + metadata), document model (tree structure), editor state (mode/cursor/history), UI (ratatui), input handler (modal), and file manager (lazy loading).

**Tech Stack:** Rust, ratatui, serde_json, crossterm, clap, toml, anyhow

---

## Phase 1: Project Foundation & Data Structures

### Task 1: Project Setup

**Files:**
- Modify: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`

**Step 1: Update Cargo.toml with dependencies**

```toml
[package]
name = "jsonquill"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.29"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
crossterm = "0.28"
clap = { version = "4.5", features = ["derive"] }
toml = "0.8"
anyhow = "1.0"
arboard = "3.4"  # for clipboard support

[dev-dependencies]
tempfile = "3.13"
```

**Step 2: Create basic main.rs**

```rust
use anyhow::Result;

fn main() -> Result<()> {
    println!("jsonquill v0.1.0");
    Ok(())
}
```

**Step 3: Create lib.rs for module organization**

```rust
pub mod document;
pub mod editor;
pub mod ui;
pub mod input;
pub mod file;
pub mod config;
pub mod theme;
```

**Step 4: Build to verify dependencies**

Run: `cargo build`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add Cargo.toml src/
git commit -m "feat: initialize jsonquill project with dependencies

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: JSON Document Model - Node Types

**Files:**
- Create: `src/document/mod.rs`
- Create: `src/document/node.rs`
- Create: `tests/document_tests.rs`

**Step 1: Write test for JSON node types**

```rust
// tests/document_tests.rs
use jsonquill::document::node::{JsonNode, JsonValue};

#[test]
fn test_create_string_node() {
    let node = JsonNode::new(JsonValue::String("hello".to_string()));
    assert!(matches!(node.value(), JsonValue::String(_)));
}

#[test]
fn test_create_number_node() {
    let node = JsonNode::new(JsonValue::Number(42.0));
    assert!(matches!(node.value(), JsonValue::Number(_)));
}

#[test]
fn test_create_boolean_node() {
    let node = JsonNode::new(JsonValue::Boolean(true));
    assert!(matches!(node.value(), JsonValue::Boolean(true)));
}

#[test]
fn test_create_null_node() {
    let node = JsonNode::new(JsonValue::Null);
    assert!(matches!(node.value(), JsonValue::Null));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_create_string_node`
Expected: FAIL with "module not found"

**Step 3: Implement JsonNode and JsonValue types**

```rust
// src/document/mod.rs
pub mod node;

// src/document/node.rs
use serde_json::Value as SerdeValue;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Object(Vec<(String, JsonNode)>),
    Array(Vec<JsonNode>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone)]
pub struct JsonNode {
    value: JsonValue,
    metadata: NodeMetadata,
}

#[derive(Debug, Clone)]
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
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_create_`
Expected: All 4 tests PASS

**Step 5: Commit**

```bash
git add src/document/ tests/document_tests.rs
git commit -m "feat: add JsonNode and JsonValue types

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 3: JSON Document Model - Tree Structure

**Files:**
- Modify: `src/document/node.rs`
- Create: `src/document/tree.rs`
- Modify: `tests/document_tests.rs`

**Step 1: Write test for tree operations**

```rust
// tests/document_tests.rs (add to file)
use jsonquill::document::tree::JsonTree;
use jsonquill::document::node::{JsonNode, JsonValue};

#[test]
fn test_create_empty_object_tree() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    assert!(tree.root().value().is_object());
}

#[test]
fn test_tree_get_child() {
    let mut obj = vec![];
    obj.push(("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))));

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));
    let path = vec![0]; // First child
    let child = tree.get_node(&path);
    assert!(child.is_some());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_create_empty_object_tree`
Expected: FAIL with "module tree not found"

**Step 3: Add is_object helper to JsonValue**

```rust
// src/document/node.rs (add impl block)
impl JsonValue {
    pub fn is_object(&self) -> bool {
        matches!(self, JsonValue::Object(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, JsonValue::Array(_))
    }

    pub fn is_container(&self) -> bool {
        self.is_object() || self.is_array()
    }
}
```

**Step 4: Implement JsonTree**

```rust
// src/document/mod.rs (add)
pub mod tree;

// src/document/tree.rs
use super::node::{JsonNode, JsonValue};

#[derive(Debug, Clone)]
pub struct JsonTree {
    root: JsonNode,
}

impl JsonTree {
    pub fn new(root: JsonNode) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &JsonNode {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut JsonNode {
        &mut self.root
    }

    /// Get node at path (indices into object keys or array elements)
    pub fn get_node(&self, path: &[usize]) -> Option<&JsonNode> {
        let mut current = &self.root;

        for &index in path {
            match current.value() {
                JsonValue::Object(entries) => {
                    current = &entries.get(index)?.1;
                }
                JsonValue::Array(elements) => {
                    current = elements.get(index)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }

    pub fn get_node_mut(&mut self, path: &[usize]) -> Option<&mut JsonNode> {
        let mut current = &mut self.root;

        for &index in path {
            match current.value_mut() {
                JsonValue::Object(entries) => {
                    current = &mut entries.get_mut(index)?.1;
                }
                JsonValue::Array(elements) => {
                    current = elements.get_mut(index)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test test_tree`
Expected: Both tests PASS

**Step 6: Commit**

```bash
git add src/document/ tests/document_tests.rs
git commit -m "feat: add JsonTree with path-based navigation

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 4: JSON Parser with Metadata

**Files:**
- Create: `src/document/parser.rs`
- Modify: `src/document/mod.rs`
- Modify: `tests/document_tests.rs`

**Step 1: Write test for parsing JSON with metadata**

```rust
// tests/document_tests.rs (add)
use jsonquill::document::parser::parse_json;

#[test]
fn test_parse_simple_object() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let tree = parse_json(json).unwrap();

    match tree.root().value() {
        JsonValue::Object(entries) => {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0, "name");
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_parse_nested_structure() {
    let json = r#"{"user": {"name": "Bob"}}"#;
    let tree = parse_json(json).unwrap();

    let user_node = tree.get_node(&[0]);
    assert!(user_node.is_some());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_parse_simple`
Expected: FAIL with "parser module not found"

**Step 3: Implement parser**

```rust
// src/document/mod.rs (add)
pub mod parser;

// src/document/parser.rs
use super::node::{JsonNode, JsonValue, NodeMetadata};
use super::tree::JsonTree;
use anyhow::{Result, Context};
use serde_json::Value as SerdeValue;

pub fn parse_json(json_str: &str) -> Result<JsonTree> {
    let serde_value: SerdeValue = serde_json::from_str(json_str)
        .context("Failed to parse JSON")?;

    let root = convert_serde_value(serde_value, Some(json_str.to_string()));
    Ok(JsonTree::new(root))
}

fn convert_serde_value(value: SerdeValue, original_text: Option<String>) -> JsonNode {
    let json_value = match value {
        SerdeValue::Object(map) => {
            let entries = map.into_iter()
                .map(|(k, v)| (k, convert_serde_value(v, None)))
                .collect();
            JsonValue::Object(entries)
        }
        SerdeValue::Array(arr) => {
            let elements = arr.into_iter()
                .map(|v| convert_serde_value(v, None))
                .collect();
            JsonValue::Array(elements)
        }
        SerdeValue::String(s) => JsonValue::String(s),
        SerdeValue::Number(n) => JsonValue::Number(n.as_f64().unwrap_or(0.0)),
        SerdeValue::Bool(b) => JsonValue::Boolean(b),
        SerdeValue::Null => JsonValue::Null,
    };

    JsonNode {
        value: json_value,
        metadata: NodeMetadata {
            original_text,
            modified: false,
        },
    }
}
```

**Step 4: Update JsonNode to make fields pub(crate)**

```rust
// src/document/node.rs (update)
pub struct JsonNode {
    pub(crate) value: JsonValue,
    pub(crate) metadata: NodeMetadata,
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test test_parse`
Expected: Both tests PASS

**Step 6: Commit**

```bash
git add src/document/ tests/document_tests.rs
git commit -m "feat: add JSON parser with metadata preservation

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Phase 2: Editor State & Configuration

### Task 5: Editor Mode Enum

**Files:**
- Create: `src/editor/mod.rs`
- Create: `src/editor/mode.rs`
- Create: `tests/editor_tests.rs`

**Step 1: Write test for mode transitions**

```rust
// tests/editor_tests.rs
use jsonquill::editor::mode::EditorMode;

#[test]
fn test_mode_starts_normal() {
    let mode = EditorMode::Normal;
    assert!(matches!(mode, EditorMode::Normal));
}

#[test]
fn test_mode_display() {
    assert_eq!(format!("{}", EditorMode::Normal), "NORMAL");
    assert_eq!(format!("{}", EditorMode::Insert), "INSERT");
    assert_eq!(format!("{}", EditorMode::Command), "COMMAND");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_mode_starts`
Expected: FAIL with "module not found"

**Step 3: Implement EditorMode**

```rust
// src/editor/mod.rs
pub mod mode;
pub mod state;

// src/editor/mode.rs
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
    Command,
}

impl fmt::Display for EditorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditorMode::Normal => write!(f, "NORMAL"),
            EditorMode::Insert => write!(f, "INSERT"),
            EditorMode::Command => write!(f, "COMMAND"),
        }
    }
}

impl Default for EditorMode {
    fn default() -> Self {
        EditorMode::Normal
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_mode`
Expected: Both tests PASS

**Step 5: Commit**

```bash
git add src/editor/ tests/editor_tests.rs
git commit -m "feat: add EditorMode enum for modal editing

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 6: Editor State Structure

**Files:**
- Create: `src/editor/state.rs`
- Create: `src/editor/cursor.rs`
- Modify: `src/editor/mod.rs`
- Modify: `tests/editor_tests.rs`

**Step 1: Write test for editor state**

```rust
// tests/editor_tests.rs (add)
use jsonquill::editor::state::EditorState;
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;

#[test]
fn test_editor_state_creation() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let state = EditorState::new(tree);

    assert_eq!(state.mode(), &EditorMode::Normal);
    assert!(!state.is_dirty());
}

#[test]
fn test_editor_state_set_dirty() {
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new(tree);

    state.mark_dirty();
    assert!(state.is_dirty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_editor_state_creation`
Expected: FAIL

**Step 3: Implement Cursor**

```rust
// src/editor/mod.rs (add)
pub mod cursor;

// src/editor/cursor.rs
/// Path to a node in the JSON tree (indices into objects/arrays)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cursor {
    path: Vec<usize>,
}

impl Cursor {
    pub fn new() -> Self {
        Self { path: vec![] }
    }

    pub fn path(&self) -> &[usize] {
        &self.path
    }

    pub fn push(&mut self, index: usize) {
        self.path.push(index);
    }

    pub fn pop(&mut self) -> Option<usize> {
        self.path.pop()
    }

    pub fn set_path(&mut self, path: Vec<usize>) {
        self.path = path;
    }
}
```

**Step 4: Implement EditorState**

```rust
// src/editor/state.rs
use super::mode::EditorMode;
use super::cursor::Cursor;
use crate::document::tree::JsonTree;

pub struct EditorState {
    tree: JsonTree,
    mode: EditorMode,
    cursor: Cursor,
    dirty: bool,
    filename: Option<String>,
}

impl EditorState {
    pub fn new(tree: JsonTree) -> Self {
        Self {
            tree,
            mode: EditorMode::Normal,
            cursor: Cursor::new(),
            dirty: false,
            filename: None,
        }
    }

    pub fn tree(&self) -> &JsonTree {
        &self.tree
    }

    pub fn tree_mut(&mut self) -> &mut JsonTree {
        &mut self.tree
    }

    pub fn mode(&self) -> &EditorMode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: EditorMode) {
        self.mode = mode;
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    pub fn set_filename(&mut self, filename: String) {
        self.filename = Some(filename);
    }
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test test_editor_state`
Expected: Both tests PASS

**Step 6: Commit**

```bash
git add src/editor/ tests/editor_tests.rs
git commit -m "feat: add EditorState with cursor and dirty tracking

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 7: Configuration System

**Files:**
- Create: `src/config/mod.rs`
- Create: `tests/config_tests.rs`

**Step 1: Write test for config defaults**

```rust
// tests/config_tests.rs
use jsonquill::config::Config;

#[test]
fn test_config_defaults() {
    let config = Config::default();

    assert_eq!(config.theme, "default-dark");
    assert_eq!(config.indent_size, 2);
    assert!(!config.auto_save);
    assert_eq!(config.validation_mode, "strict");
    assert!(!config.create_backup);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_config_defaults`
Expected: FAIL

**Step 3: Implement Config struct**

```rust
// src/config/mod.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_indent_size")]
    pub indent_size: usize,

    #[serde(default)]
    pub show_line_numbers: bool,

    #[serde(default)]
    pub auto_save: bool,

    #[serde(default = "default_validation_mode")]
    pub validation_mode: String,

    #[serde(default)]
    pub create_backup: bool,

    #[serde(default = "default_undo_limit")]
    pub undo_limit: usize,

    #[serde(default)]
    pub sync_unnamed_register: bool,

    #[serde(default = "default_lazy_load_threshold")]
    pub lazy_load_threshold: usize,
}

fn default_theme() -> String {
    "default-dark".to_string()
}

fn default_indent_size() -> usize {
    2
}

fn default_validation_mode() -> String {
    "strict".to_string()
}

fn default_undo_limit() -> usize {
    1000
}

fn default_lazy_load_threshold() -> usize {
    104_857_600 // 100MB
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            indent_size: default_indent_size(),
            show_line_numbers: true,
            auto_save: false,
            validation_mode: default_validation_mode(),
            create_backup: false,
            undo_limit: default_undo_limit(),
            sync_unnamed_register: true,
            lazy_load_threshold: default_lazy_load_threshold(),
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_config_defaults`
Expected: PASS

**Step 5: Commit**

```bash
git add src/config/ tests/config_tests.rs
git commit -m "feat: add Config struct with defaults

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Phase 3: Basic UI with Ratatui

### Task 8: Theme System

**Files:**
- Create: `src/theme/mod.rs`
- Create: `src/theme/colors.rs`
- Create: `tests/theme_tests.rs`

**Step 1: Write test for theme colors**

```rust
// tests/theme_tests.rs
use jsonquill::theme::{Theme, get_builtin_theme};

#[test]
fn test_default_dark_theme_exists() {
    let theme = get_builtin_theme("default-dark");
    assert!(theme.is_some());
}

#[test]
fn test_invalid_theme_returns_none() {
    let theme = get_builtin_theme("nonexistent");
    assert!(theme.is_none());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_default_dark_theme`
Expected: FAIL

**Step 3: Implement Theme struct**

```rust
// src/theme/mod.rs
pub mod colors;

use ratatui::style::Color;
use colors::ThemeColors;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
}

pub fn get_builtin_theme(name: &str) -> Option<Theme> {
    match name {
        "default-dark" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::default_dark(),
        }),
        "default-light" => Some(Theme {
            name: name.to_string(),
            colors: ThemeColors::default_light(),
        }),
        _ => None,
    }
}

// src/theme/colors.rs
use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct ThemeColors {
    // Syntax colors
    pub key: Color,
    pub string: Color,
    pub number: Color,
    pub boolean: Color,
    pub null: Color,

    // UI colors
    pub background: Color,
    pub foreground: Color,
    pub cursor: Color,
    pub status_line_bg: Color,
    pub status_line_fg: Color,

    // Semantic colors
    pub error: Color,
    pub warning: Color,
    pub info: Color,
    pub search_highlight: Color,
}

impl ThemeColors {
    pub fn default_dark() -> Self {
        Self {
            key: Color::Rgb(224, 108, 117),      // #e06c75
            string: Color::Rgb(152, 195, 121),   // #98c379
            number: Color::Rgb(209, 154, 102),   // #d19a66
            boolean: Color::Rgb(86, 182, 194),   // #56b6c2
            null: Color::Rgb(198, 120, 221),     // #c678dd

            background: Color::Rgb(40, 44, 52),  // #282c34
            foreground: Color::Rgb(171, 178, 191), // #abb2bf
            cursor: Color::Rgb(82, 139, 255),    // #528bff
            status_line_bg: Color::Rgb(33, 37, 43), // #21252b
            status_line_fg: Color::Rgb(171, 178, 191),

            error: Color::Rgb(224, 108, 117),
            warning: Color::Rgb(229, 192, 123),  // #e5c07b
            info: Color::Rgb(97, 175, 239),      // #61afef
            search_highlight: Color::Rgb(62, 68, 81), // #3e4451
        }
    }

    pub fn default_light() -> Self {
        Self {
            key: Color::Rgb(166, 38, 164),
            string: Color::Rgb(80, 161, 79),
            number: Color::Rgb(152, 104, 1),
            boolean: Color::Rgb(1, 132, 188),
            null: Color::Rgb(160, 30, 170),

            background: Color::Rgb(250, 250, 250),
            foreground: Color::Rgb(56, 58, 66),
            cursor: Color::Rgb(82, 139, 255),
            status_line_bg: Color::Rgb(238, 238, 238),
            status_line_fg: Color::Rgb(56, 58, 66),

            error: Color::Rgb(202, 18, 67),
            warning: Color::Rgb(152, 104, 1),
            info: Color::Rgb(1, 132, 188),
            search_highlight: Color::Rgb(220, 220, 220),
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_default_dark_theme`
Expected: Both tests PASS

**Step 5: Commit**

```bash
git add src/theme/ tests/theme_tests.rs
git commit -m "feat: add theme system with default-dark and default-light

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 9: Basic UI Layout

**Files:**
- Create: `src/ui/mod.rs`
- Create: `src/ui/layout.rs`
- Modify: `src/main.rs`

**Step 1: Create UI module structure**

```rust
// src/ui/mod.rs
pub mod layout;
pub mod tree_view;
pub mod status_line;

use ratatui::backend::Backend;
use ratatui::Terminal;
use crate::editor::state::EditorState;
use crate::theme::Theme;
use anyhow::Result;

pub struct UI {
    theme: Theme,
}

impl UI {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn render<B: Backend>(
        &self,
        terminal: &mut Terminal<B>,
        state: &EditorState,
    ) -> Result<()> {
        use ratatui::Frame;
        use ratatui::layout::{Constraint, Direction, Layout};

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),      // Main view
                    Constraint::Length(1),   // Status line
                    Constraint::Length(1),   // Message area
                ])
                .split(f.area());

            // For now, just render empty blocks
            use ratatui::widgets::{Block, Borders};
            let block = Block::default().borders(Borders::NONE);
            f.render_widget(block, chunks[0]);
        })?;

        Ok(())
    }
}
```

**Step 2: Update main.rs to initialize terminal**

```rust
// src/main.rs
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Clear screen
    terminal.clear()?;

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    println!("jsonquill v0.1.0 - TUI initialized successfully");
    Ok(())
}
```

**Step 3: Build to verify compilation**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Run to verify terminal setup works**

Run: `cargo run`
Expected: Screen clears and returns with success message

**Step 5: Commit**

```bash
git add src/ui/ src/main.rs
git commit -m "feat: add basic UI structure and terminal initialization

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 10: Status Line Widget

**Files:**
- Create: `src/ui/status_line.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Implement status line widget**

```rust
// src/ui/status_line.rs
use ratatui::{
    layout::Rect,
    style::{Style, Color},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::editor::state::EditorState;
use crate::theme::colors::ThemeColors;

pub fn render_status_line(
    f: &mut Frame,
    area: Rect,
    state: &EditorState,
    colors: &ThemeColors,
) {
    let mode_text = format!("{}", state.mode());
    let filename = state.filename().unwrap_or("[No Name]");
    let dirty_indicator = if state.is_dirty() { " [+]" } else { "" };

    let left = format!("{} | {}{}", mode_text, filename, dirty_indicator);

    let line = Line::from(vec![
        Span::styled(
            left,
            Style::default()
                .fg(colors.status_line_fg)
                .bg(colors.status_line_bg),
        ),
    ]);

    let status = Paragraph::new(line)
        .style(Style::default().bg(colors.status_line_bg));

    f.render_widget(status, area);
}
```

**Step 2: Update UI render to use status line**

```rust
// src/ui/mod.rs (update render method)
pub fn render<B: Backend>(
    &self,
    terminal: &mut Terminal<B>,
    state: &EditorState,
) -> Result<()> {
    use ratatui::layout::{Constraint, Direction, Layout};

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),      // Main view
                Constraint::Length(1),   // Status line
                Constraint::Length(1),   // Message area
            ])
            .split(f.area());

        // Render status line
        status_line::render_status_line(
            f,
            chunks[1],
            state,
            &self.theme.colors,
        );
    })?;

    Ok(())
}
```

**Step 3: Build to verify**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/ui/
git commit -m "feat: add status line widget with mode and filename

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Phase 4: Input Handling & Navigation

### Task 11: Input Event Handler

**Files:**
- Create: `src/input/mod.rs`
- Create: `src/input/handler.rs`
- Create: `src/input/keys.rs`

**Step 1: Create input event structure**

```rust
// src/input/mod.rs
pub mod handler;
pub mod keys;

pub use handler::InputHandler;

// src/input/keys.rs
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Quit,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    EnterInsertMode,
    EnterCommandMode,
    ExitMode,
    Unknown,
}

pub fn map_key_event(key: KeyEvent, mode: &crate::editor::mode::EditorMode) -> InputEvent {
    use crate::editor::mode::EditorMode;

    match mode {
        EditorMode::Normal => match key.code {
            KeyCode::Char('q') => InputEvent::Quit,
            KeyCode::Char('j') => InputEvent::MoveDown,
            KeyCode::Char('k') => InputEvent::MoveUp,
            KeyCode::Char('h') => InputEvent::MoveLeft,
            KeyCode::Char('l') => InputEvent::MoveRight,
            KeyCode::Char('i') => InputEvent::EnterInsertMode,
            KeyCode::Char(':') => InputEvent::EnterCommandMode,
            KeyCode::Down => InputEvent::MoveDown,
            KeyCode::Up => InputEvent::MoveUp,
            KeyCode::Left => InputEvent::MoveLeft,
            KeyCode::Right => InputEvent::MoveRight,
            _ => InputEvent::Unknown,
        },
        EditorMode::Insert => match key.code {
            KeyCode::Esc => InputEvent::ExitMode,
            _ => InputEvent::Unknown,
        },
        EditorMode::Command => match key.code {
            KeyCode::Esc => InputEvent::ExitMode,
            _ => InputEvent::Unknown,
        },
    }
}
```

**Step 2: Create input handler**

```rust
// src/input/handler.rs
use super::keys::{map_key_event, InputEvent};
use crate::editor::state::EditorState;
use crate::editor::mode::EditorMode;
use crossterm::event::{self, Event, KeyCode};
use anyhow::Result;
use std::time::Duration;

pub struct InputHandler;

impl InputHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn poll_event(&self, timeout: Duration) -> Result<Option<Event>> {
        if event::poll(timeout)? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }

    pub fn handle_event(&self, event: Event, state: &mut EditorState) -> Result<bool> {
        if let Event::Key(key) = event {
            let input_event = map_key_event(key, state.mode());

            match input_event {
                InputEvent::Quit => return Ok(true),
                InputEvent::EnterInsertMode => {
                    state.set_mode(EditorMode::Insert);
                }
                InputEvent::EnterCommandMode => {
                    state.set_mode(EditorMode::Command);
                }
                InputEvent::ExitMode => {
                    state.set_mode(EditorMode::Normal);
                }
                InputEvent::MoveDown => {
                    // TODO: implement cursor movement
                }
                InputEvent::MoveUp => {
                    // TODO: implement cursor movement
                }
                _ => {}
            }
        }

        Ok(false)
    }
}
```

**Step 3: Build to verify**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/input/
git commit -m "feat: add input event handler with basic key mapping

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 12: Main Event Loop

**Files:**
- Modify: `src/main.rs`

**Step 1: Update main.rs with event loop**

```rust
// src/main.rs
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;
use jsonquill::input::InputHandler;
use jsonquill::theme::get_builtin_theme;
use jsonquill::ui::UI;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Initialize components
    let theme = get_builtin_theme("default-dark").unwrap();
    let ui = UI::new(theme);
    let input_handler = InputHandler::new();

    // Create empty document for now
    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(vec![])));
    let mut state = EditorState::new(tree);

    // Main event loop
    let result = run_event_loop(&mut terminal, &ui, &input_handler, &mut state);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    ui: &UI,
    input_handler: &InputHandler,
    state: &mut EditorState,
) -> Result<()> {
    loop {
        // Render UI
        ui.render(terminal, state)?;

        // Handle input
        if let Some(event) = input_handler.poll_event(Duration::from_millis(100))? {
            let should_quit = input_handler.handle_event(event, state)?;
            if should_quit {
                break;
            }
        }
    }

    Ok(())
}
```

**Step 2: Run to test basic event loop**

Run: `cargo run`
Expected: Opens TUI, shows status line with "NORMAL", press 'q' to quit

**Step 3: Test mode switching**

Run: `cargo run`
Expected: Press 'i' to see "INSERT", press Esc to return to "NORMAL"

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: add main event loop with UI rendering and input

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Phase 5: Tree View Rendering

### Task 13: Tree View Data Structure

**Files:**
- Create: `src/ui/tree_view.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Create tree view line structure**

```rust
// src/ui/tree_view.rs
use crate::document::node::{JsonNode, JsonValue};
use crate::document::tree::JsonTree;

#[derive(Debug, Clone)]
pub struct TreeViewLine {
    pub path: Vec<usize>,
    pub depth: usize,
    pub key: Option<String>,
    pub value_type: ValueType,
    pub value_preview: String,
    pub expandable: bool,
    pub expanded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    Object,
    Array,
    String,
    Number,
    Boolean,
    Null,
}

impl ValueType {
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

pub struct TreeViewState {
    lines: Vec<TreeViewLine>,
    expanded_paths: std::collections::HashSet<Vec<usize>>,
}

impl TreeViewState {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            expanded_paths: std::collections::HashSet::new(),
        }
    }

    pub fn lines(&self) -> &[TreeViewLine] {
        &self.lines
    }

    pub fn toggle_expand(&mut self, path: &[usize]) {
        if self.expanded_paths.contains(path) {
            self.expanded_paths.remove(path);
        } else {
            self.expanded_paths.insert(path.to_vec());
        }
    }

    pub fn is_expanded(&self, path: &[usize]) -> bool {
        self.expanded_paths.contains(path)
    }

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
```

**Step 2: Build to verify**

Run: `cargo build`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/ui/tree_view.rs src/ui/mod.rs
git commit -m "feat: add tree view data structure with expand/collapse

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 14: Tree View Rendering

**Files:**
- Modify: `src/ui/tree_view.rs`
- Modify: `src/editor/state.rs`

**Step 1: Add tree view state to editor state**

```rust
// src/editor/state.rs (add to imports and struct)
use crate::ui::tree_view::TreeViewState;

pub struct EditorState {
    tree: JsonTree,
    mode: EditorMode,
    cursor: Cursor,
    dirty: bool,
    filename: Option<String>,
    tree_view: TreeViewState,  // Add this
}

impl EditorState {
    pub fn new(tree: JsonTree) -> Self {
        let mut tree_view = TreeViewState::new();
        tree_view.rebuild(&tree);

        Self {
            tree,
            mode: EditorMode::Normal,
            cursor: Cursor::new(),
            dirty: false,
            filename: None,
            tree_view,
        }
    }

    pub fn tree_view(&self) -> &TreeViewState {
        &self.tree_view
    }

    pub fn tree_view_mut(&mut self) -> &mut TreeViewState {
        &mut self.tree_view
    }
}
```

**Step 2: Add tree view rendering function**

```rust
// src/ui/tree_view.rs (add to end of file)
use ratatui::{
    layout::Rect,
    style::{Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::theme::colors::ThemeColors;
use crate::editor::cursor::Cursor;

pub fn render_tree_view(
    f: &mut Frame,
    area: Rect,
    tree_view: &TreeViewState,
    cursor: &Cursor,
    colors: &ThemeColors,
) {
    let mut lines_to_render = Vec::new();

    for (idx, line) in tree_view.lines().iter().enumerate() {
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
```

**Step 3: Update UI render to show tree view**

```rust
// src/ui/mod.rs (update render method)
pub fn render<B: Backend>(
    &self,
    terminal: &mut Terminal<B>,
    state: &EditorState,
) -> Result<()> {
    use ratatui::layout::{Constraint, Direction, Layout};

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),      // Main view
                Constraint::Length(1),   // Status line
                Constraint::Length(1),   // Message area
            ])
            .split(f.area());

        // Render tree view
        tree_view::render_tree_view(
            f,
            chunks[0],
            state.tree_view(),
            state.cursor(),
            &self.theme.colors,
        );

        // Render status line
        status_line::render_status_line(
            f,
            chunks[1],
            state,
            &self.theme.colors,
        );
    })?;

    Ok(())
}
```

**Step 4: Test with sample data - update main.rs**

```rust
// src/main.rs (update tree creation)
// Create sample document
let mut obj = vec![];
obj.push(("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))));
obj.push(("age".to_string(), JsonNode::new(JsonValue::Number(30.0))));
obj.push(("active".to_string(), JsonNode::new(JsonValue::Boolean(true))));

let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));
let mut state = EditorState::new(tree);
```

**Step 5: Run to test tree rendering**

Run: `cargo run`
Expected: Shows JSON tree with "name", "age", "active" fields, cursor on first line

**Step 6: Commit**

```bash
git add src/ui/ src/editor/ src/main.rs
git commit -m "feat: add tree view rendering with syntax highlighting

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Phase 6: Navigation & Cursor Movement

### Task 15: Cursor Navigation

**Files:**
- Modify: `src/input/handler.rs`
- Modify: `src/editor/state.rs`

**Step 1: Add navigation methods to EditorState**

```rust
// src/editor/state.rs (add methods)
impl EditorState {
    pub fn move_cursor_down(&mut self) {
        let lines = self.tree_view.lines();
        if lines.is_empty() {
            return;
        }

        let current_path = self.cursor.path();

        // Find current line index
        if let Some(current_idx) = lines.iter().position(|l| l.path == current_path) {
            if current_idx + 1 < lines.len() {
                let next_path = lines[current_idx + 1].path.clone();
                self.cursor.set_path(next_path);
            }
        } else if !lines.is_empty() {
            // If cursor not found, go to first line
            self.cursor.set_path(lines[0].path.clone());
        }
    }

    pub fn move_cursor_up(&mut self) {
        let lines = self.tree_view.lines();
        if lines.is_empty() {
            return;
        }

        let current_path = self.cursor.path();

        if let Some(current_idx) = lines.iter().position(|l| l.path == current_path) {
            if current_idx > 0 {
                let prev_path = lines[current_idx - 1].path.clone();
                self.cursor.set_path(prev_path);
            }
        } else if !lines.is_empty() {
            self.cursor.set_path(lines[0].path.clone());
        }
    }

    pub fn toggle_expand_at_cursor(&mut self) {
        let current_path = self.cursor.path().to_vec();
        self.tree_view.toggle_expand(&current_path);
        self.tree_view.rebuild(&self.tree);
    }
}
```

**Step 2: Update input handler to call navigation methods**

```rust
// src/input/handler.rs (update handle_event)
pub fn handle_event(&self, event: Event, state: &mut EditorState) -> Result<bool> {
    if let Event::Key(key) = event {
        let input_event = map_key_event(key, state.mode());

        match input_event {
            InputEvent::Quit => return Ok(true),
            InputEvent::EnterInsertMode => {
                state.set_mode(EditorMode::Insert);
            }
            InputEvent::EnterCommandMode => {
                state.set_mode(EditorMode::Command);
            }
            InputEvent::ExitMode => {
                state.set_mode(EditorMode::Normal);
            }
            InputEvent::MoveDown => {
                state.move_cursor_down();
            }
            InputEvent::MoveUp => {
                state.move_cursor_up();
            }
            InputEvent::MoveRight => {
                state.toggle_expand_at_cursor();
            }
            InputEvent::MoveLeft => {
                state.toggle_expand_at_cursor();
            }
            _ => {}
        }
    }

    Ok(false)
}
```

**Step 3: Update main.rs with nested test data**

```rust
// src/main.rs (update tree creation)
let mut user_obj = vec![];
user_obj.push(("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))));
user_obj.push(("email".to_string(), JsonNode::new(JsonValue::String("alice@example.com".to_string()))));

let mut obj = vec![];
obj.push(("user".to_string(), JsonNode::new(JsonValue::Object(user_obj))));
obj.push(("count".to_string(), JsonNode::new(JsonValue::Number(42.0))));
obj.push(("active".to_string(), JsonNode::new(JsonValue::Boolean(true))));

let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));
let mut state = EditorState::new(tree);
```

**Step 4: Run to test navigation**

Run: `cargo run`
Expected: j/k moves cursor up/down, l expands nodes, h collapses

**Step 5: Commit**

```bash
git add src/input/ src/editor/ src/main.rs
git commit -m "feat: add cursor navigation and expand/collapse

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Phase 7: File I/O

### Task 16: File Loading

**Files:**
- Create: `src/file/mod.rs`
- Create: `src/file/loader.rs`
- Create: `tests/file_tests.rs`

**Step 1: Write test for file loading**

```rust
// tests/file_tests.rs
use jsonquill::file::loader::load_json_file;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_load_json_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"{{"name": "test"}}"#).unwrap();

    let tree = load_json_file(temp_file.path()).unwrap();
    assert!(tree.root().value().is_object());
}

#[test]
fn test_load_invalid_json() {
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, r#"{{invalid json}}"#).unwrap();

    let result = load_json_file(temp_file.path());
    assert!(result.is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_load_json_file`
Expected: FAIL

**Step 3: Implement file loader**

```rust
// src/file/mod.rs
pub mod loader;
pub mod saver;

// src/file/loader.rs
use crate::document::tree::JsonTree;
use crate::document::parser::parse_json;
use anyhow::{Result, Context};
use std::path::Path;
use std::fs;

pub fn load_json_file<P: AsRef<Path>>(path: P) -> Result<JsonTree> {
    let content = fs::read_to_string(path.as_ref())
        .context("Failed to read file")?;

    parse_json(&content)
        .context("Failed to parse JSON")
}

pub fn load_json_from_stdin() -> Result<JsonTree> {
    use std::io::{self, Read};

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)
        .context("Failed to read from stdin")?;

    parse_json(&buffer)
        .context("Failed to parse JSON from stdin")
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_load`
Expected: Both tests PASS

**Step 5: Commit**

```bash
git add src/file/ tests/file_tests.rs
git commit -m "feat: add file loading for JSON files

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 17: File Saving

**Files:**
- Create: `src/file/saver.rs`
- Modify: `tests/file_tests.rs`

**Step 1: Write test for file saving**

```rust
// tests/file_tests.rs (add)
use jsonquill::file::saver::save_json_file;
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;

#[test]
fn test_save_json_file() {
    let mut obj = vec![];
    obj.push(("name".to_string(), JsonNode::new(JsonValue::String("test".to_string()))));

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));
    let temp_file = NamedTempFile::new().unwrap();

    save_json_file(temp_file.path(), &tree, 2, false).unwrap();

    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("\"name\""));
    assert!(content.contains("\"test\""));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_save_json_file`
Expected: FAIL

**Step 3: Implement file saver**

```rust
// src/file/saver.rs
use crate::document::tree::JsonTree;
use crate::document::node::{JsonNode, JsonValue};
use anyhow::{Result, Context};
use std::path::Path;
use std::fs;

pub fn save_json_file<P: AsRef<Path>>(
    path: P,
    tree: &JsonTree,
    indent: usize,
    create_backup: bool,
) -> Result<()> {
    let path = path.as_ref();

    // Create backup if requested and file exists
    if create_backup && path.exists() {
        let backup_path = path.with_extension("jsonquill.bak");
        fs::copy(path, backup_path)
            .context("Failed to create backup")?;
    }

    // Serialize to JSON
    let json_str = serialize_node(tree.root(), indent, 0);

    // Write to temp file first (atomic save)
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, json_str)
        .context("Failed to write temp file")?;

    // Rename temp to target (atomic operation)
    fs::rename(&temp_path, path)
        .context("Failed to rename temp file")?;

    Ok(())
}

fn serialize_node(node: &JsonNode, indent_size: usize, current_depth: usize) -> String {
    let indent = " ".repeat(indent_size * current_depth);
    let next_indent = " ".repeat(indent_size * (current_depth + 1));

    match node.value() {
        JsonValue::Object(entries) => {
            if entries.is_empty() {
                return "{}".to_string();
            }

            let mut result = "{\n".to_string();
            for (i, (key, value)) in entries.iter().enumerate() {
                result.push_str(&next_indent);
                result.push_str(&format!("\"{}\": ", key));
                result.push_str(&serialize_node(value, indent_size, current_depth + 1));
                if i < entries.len() - 1 {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&indent);
            result.push('}');
            result
        }
        JsonValue::Array(elements) => {
            if elements.is_empty() {
                return "[]".to_string();
            }

            let mut result = "[\n".to_string();
            for (i, element) in elements.iter().enumerate() {
                result.push_str(&next_indent);
                result.push_str(&serialize_node(element, indent_size, current_depth + 1));
                if i < elements.len() - 1 {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&indent);
            result.push(']');
            result
        }
        JsonValue::String(s) => format!("\"{}\"", s),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Boolean(b) => b.to_string(),
        JsonValue::Null => "null".to_string(),
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_save_json_file`
Expected: PASS

**Step 5: Commit**

```bash
git add src/file/ tests/file_tests.rs
git commit -m "feat: add file saving with atomic write

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 18: CLI Argument Parsing

**Files:**
- Create: `src/cli.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`

**Step 1: Implement CLI args**

```rust
// src/lib.rs (add)
pub mod cli;

// src/cli.rs
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "jsonquill")]
#[command(version = "0.1.0")]
#[command(about = "A terminal-based JSON editor", long_about = None)]
pub struct Cli {
    /// JSON file to open
    pub file: Option<PathBuf>,

    /// Force file type (json or jsonl)
    #[arg(short, long)]
    pub mode: Option<String>,

    /// Theme to use
    #[arg(short, long)]
    pub theme: Option<String>,

    /// Open in read-only mode
    #[arg(short, long)]
    pub readonly: bool,

    /// Force strict validation
    #[arg(short, long)]
    pub strict: bool,

    /// Force lenient validation
    #[arg(short, long)]
    pub lenient: bool,
}
```

**Step 2: Update main.rs to use CLI args**

```rust
// src/main.rs
use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

use jsonquill::cli::Cli;
use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;
use jsonquill::input::InputHandler;
use jsonquill::theme::get_builtin_theme;
use jsonquill::ui::UI;
use jsonquill::file::loader::{load_json_file, load_json_from_stdin};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Initialize components
    let theme_name = cli.theme.as_deref().unwrap_or("default-dark");
    let theme = get_builtin_theme(theme_name).unwrap_or_else(|| get_builtin_theme("default-dark").unwrap());
    let ui = UI::new(theme);
    let input_handler = InputHandler::new();

    // Load document
    let tree = if let Some(file_path) = cli.file {
        load_json_file(&file_path)?
    } else if atty::isnt(atty::Stream::Stdin) {
        load_json_from_stdin()?
    } else {
        // Empty document
        JsonTree::new(JsonNode::new(JsonValue::Object(vec![])))
    };

    let mut state = EditorState::new(tree);

    // Main event loop
    let result = run_event_loop(&mut terminal, &ui, &input_handler, &mut state);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    ui: &UI,
    input_handler: &InputHandler,
    state: &mut EditorState,
) -> Result<()> {
    loop {
        ui.render(terminal, state)?;

        if let Some(event) = input_handler.poll_event(Duration::from_millis(100))? {
            let should_quit = input_handler.handle_event(event, state)?;
            if should_quit {
                break;
            }
        }
    }

    Ok(())
}
```

**Step 3: Add atty dependency to Cargo.toml**

```toml
# Cargo.toml (add to dependencies)
atty = "0.2"
```

**Step 4: Test CLI with file argument**

Create test file:
```bash
echo '{"test": "value"}' > /tmp/test.json
```

Run: `cargo run -- /tmp/test.json`
Expected: Opens with test.json loaded

**Step 5: Commit**

```bash
git add src/cli.rs src/lib.rs src/main.rs Cargo.toml
git commit -m "feat: add CLI argument parsing and file loading

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Summary and Next Steps

This implementation plan covers the foundational architecture of jsonquill:

**Completed in this plan:**
- ✅ Project structure and dependencies
- ✅ JSON document model with tree structure
- ✅ Parser with metadata preservation
- ✅ Editor state and modes
- ✅ Configuration system
- ✅ Theme system with default themes
- ✅ Basic UI with ratatui
- ✅ Tree view rendering with syntax highlighting
- ✅ Input handling and navigation
- ✅ File I/O (load/save)
- ✅ CLI argument parsing

**Still TODO (follow-up plans needed):**
- Command mode implementation (`:w`, `:q`, etc.)
- Insert mode with type-aware editing
- Undo/redo system
- Yank/paste with registers
- Search functionality (text and path-based)
- JSONL support
- Additional themes (monokai, solarized, etc.)
- Help overlay
- Config file loading/saving
- Clipboard integration
- Large file lazy loading
- Error handling and validation modes
- More keybindings (gg, G, {}, etc.)

**Testing the current implementation:**

```bash
# Run with sample file
echo '{"user": {"name": "Alice", "email": "alice@example.com"}, "count": 42}' > sample.json
cargo run -- sample.json

# Navigation:
# - j/k or arrows: move cursor
# - l: expand node
# - h: collapse node
# - i: enter insert mode (shows in status)
# - Esc: return to normal mode
# - q: quit
```
