/// UI module for jeditor terminal interface.
///
/// This module provides the main UI structure for rendering the terminal interface,
/// including layout management and widget composition.
pub mod layout;
pub mod status_line;
pub mod tree_view;

use anyhow::Result;
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders};
use ratatui::Terminal;

use crate::editor::state::EditorState;
use crate::theme::Theme;

/// Main UI structure that manages the terminal interface rendering.
///
/// The UI is composed of three main areas:
/// - Main view area (top): Displays the JSON tree structure
/// - Status line (middle): Shows current mode, file info, and cursor position
/// - Message area (bottom): Displays messages and prompts to the user
///
/// # Example
///
/// ```no_run
/// use jeditor::ui::UI;
/// use jeditor::theme::get_builtin_theme;
/// use jeditor::editor::state::EditorState;
/// use jeditor::document::tree::JsonTree;
/// use jeditor::document::node::{JsonNode, JsonValue};
/// use ratatui::backend::CrosstermBackend;
/// use ratatui::Terminal;
/// use std::io;
///
/// let theme = get_builtin_theme("default-dark").unwrap();
/// let ui = UI::new(theme);
/// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
/// let state = EditorState::new(tree);
/// let backend = CrosstermBackend::new(io::stdout());
/// let mut terminal = Terminal::new(backend).unwrap();
/// // ui.render(&mut terminal, &state).unwrap();
/// ```
pub struct UI {
    theme: Theme,
}

impl UI {
    /// Creates a new UI instance with the specified theme.
    ///
    /// # Arguments
    ///
    /// * `theme` - The color theme to use for rendering
    ///
    /// # Example
    ///
    /// ```
    /// use jeditor::ui::UI;
    /// use jeditor::theme::get_builtin_theme;
    ///
    /// let theme = get_builtin_theme("default-dark").unwrap();
    /// let ui = UI::new(theme);
    /// ```
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Renders the UI to the terminal.
    ///
    /// This method draws the complete UI layout including the main view area,
    /// status line, and message area. Currently renders a minimal layout with
    /// empty blocks as placeholder widgets.
    ///
    /// # Arguments
    ///
    /// * `terminal` - The ratatui terminal instance to render to
    /// * `state` - The current editor state containing document and cursor information
    ///
    /// # Errors
    ///
    /// Returns an error if terminal drawing fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jeditor::ui::UI;
    /// use jeditor::theme::get_builtin_theme;
    /// use jeditor::editor::state::EditorState;
    /// use jeditor::document::tree::JsonTree;
    /// use jeditor::document::node::{JsonNode, JsonValue};
    /// use ratatui::backend::CrosstermBackend;
    /// use ratatui::Terminal;
    /// use std::io;
    ///
    /// let theme = get_builtin_theme("default-dark").unwrap();
    /// let ui = UI::new(theme);
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let state = EditorState::new(tree);
    /// let backend = CrosstermBackend::new(io::stdout());
    /// let mut terminal = Terminal::new(backend).unwrap();
    /// ui.render(&mut terminal, &state).unwrap();
    /// ```
    pub fn render<B: Backend>(
        &self,
        terminal: &mut Terminal<B>,
        _state: &EditorState,
    ) -> Result<()> {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),    // Main view area
                    Constraint::Length(1), // Status line
                    Constraint::Length(1), // Message area
                ])
                .split(f.area());

            // For now, render empty blocks as placeholders
            // These will be replaced with actual widgets in future tasks
            let block = Block::default().borders(Borders::NONE);
            f.render_widget(block, chunks[0]);
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::get_builtin_theme;

    #[test]
    fn test_ui_creation() {
        let theme = get_builtin_theme("default-dark").unwrap();
        let _ui = UI::new(theme);
        // Verify UI can be created without panicking
    }

    #[test]
    fn test_ui_with_light_theme() {
        let theme = get_builtin_theme("default-light").unwrap();
        let _ui = UI::new(theme);
        // Verify UI can be created with light theme
    }
}
