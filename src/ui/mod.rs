pub mod edit_prompt;
pub mod help_overlay;
/// UI module for jsonquill terminal interface.
///
/// This module provides the main UI structure for rendering the terminal interface,
/// including layout management and widget composition.
pub mod layout;
pub mod message_area;
pub mod status_line;
pub mod theme_picker;
pub mod tree_view;

use anyhow::Result;
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout};
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
/// use jsonquill::ui::UI;
/// use jsonquill::theme::get_builtin_theme;
/// use jsonquill::editor::state::EditorState;
/// use jsonquill::document::tree::JsonTree;
/// use jsonquill::document::node::{JsonNode, JsonValue};
/// use ratatui::backend::TermionBackend;
/// use ratatui::Terminal;
/// use std::io;
/// use termion::raw::IntoRawMode;
///
/// let theme = get_builtin_theme("default-dark").unwrap();
/// let ui = UI::new(theme);
/// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
/// let state = EditorState::new_with_default_theme(tree);
/// let backend = TermionBackend::new(io::stdout().into_raw_mode().unwrap());
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
    /// use jsonquill::ui::UI;
    /// use jsonquill::theme::get_builtin_theme;
    ///
    /// let theme = get_builtin_theme("default-dark").unwrap();
    /// let ui = UI::new(theme);
    /// ```
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Returns the current theme name.
    pub fn theme_name(&self) -> &str {
        &self.theme.name
    }

    /// Changes the current theme.
    ///
    /// Returns true if the theme was successfully changed, false if the theme name is invalid.
    pub fn set_theme(&mut self, theme_name: &str) -> bool {
        use crate::theme::get_builtin_theme;

        if let Some(new_theme) = get_builtin_theme(theme_name) {
            self.theme = new_theme;
            true
        } else {
            false
        }
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
    /// use jsonquill::ui::UI;
    /// use jsonquill::theme::get_builtin_theme;
    /// use jsonquill::editor::state::EditorState;
    /// use jsonquill::document::tree::JsonTree;
    /// use jsonquill::document::node::{JsonNode, JsonValue};
    /// use ratatui::backend::TermionBackend;
    /// use ratatui::Terminal;
    /// use std::io;
    /// use termion::raw::IntoRawMode;
    ///
    /// let theme = get_builtin_theme("default-dark").unwrap();
    /// let ui = UI::new(theme);
    /// let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
    /// let mut state = EditorState::new_with_default_theme(tree);
    /// let backend = TermionBackend::new(io::stdout().into_raw_mode().unwrap());
    /// let mut terminal = Terminal::new(backend).unwrap();
    /// ui.render(&mut terminal, &mut state).unwrap();
    /// ```
    pub fn render<B: Backend>(
        &self,
        terminal: &mut Terminal<B>,
        state: &mut EditorState,
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

            // Adjust scroll to ensure cursor is visible
            let viewport_height = chunks[0].height as usize;
            state.adjust_scroll_to_cursor(viewport_height);

            // Render tree view
            tree_view::render_tree_view(
                f,
                chunks[0],
                state.tree_view(),
                state.cursor(),
                &self.theme.colors,
                state.show_line_numbers(),
                state.relative_line_numbers(),
                state.scroll_offset(),
                state.visual_selection(),
            );

            // Status line
            status_line::render_status_line(f, chunks[1], state, &self.theme.colors);

            // Render key prompt if in AwaitingKey stage
            use crate::editor::state::AddModeStage;
            if matches!(state.add_mode_stage(), AddModeStage::AwaitingKey) {
                // Render key prompt with cursor
                edit_prompt::render_edit_prompt(
                    f,
                    chunks[2],
                    state.add_key_buffer(),
                    state.add_key_cursor_position(),
                    state.cursor_visible(),
                    &self.theme.colors,
                    "Key: ",
                );
            } else if let Some(buffer) = state.edit_buffer() {
                // Render edit prompt if in insert mode with active buffer
                // If we're in AwaitingValue stage with a key, show the key as the prompt
                let prompt = if matches!(state.add_mode_stage(), AddModeStage::AwaitingValue)
                    && !state.add_key_buffer().is_empty()
                {
                    format!("{}: ", state.add_key_buffer())
                } else {
                    "Edit: ".to_string()
                };

                edit_prompt::render_edit_prompt(
                    f,
                    chunks[2],
                    buffer,
                    state.edit_cursor_position(),
                    state.cursor_visible(),
                    &self.theme.colors,
                    &prompt,
                );
            } else {
                // Message area
                message_area::render_message_area(f, chunks[2], state, &self.theme.colors);
            }

            // Help overlay (rendered on top if visible)
            if state.show_help() {
                help_overlay::render_help_overlay(f, &self.theme.colors, state.help_scroll());
            }

            // Theme picker overlay (rendered on top if visible)
            if state.show_theme_picker() {
                if let Some(picker_state) = state.theme_picker_state() {
                    theme_picker::render_theme_picker(f, picker_state, &self.theme.colors);
                }
            }
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

    #[test]
    fn test_render_executes() {
        use crate::document::node::{JsonNode, JsonValue};
        use crate::document::tree::JsonTree;
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let theme = get_builtin_theme("default-dark").unwrap();
        let ui = UI::new(theme);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new_with_default_theme(tree);
        let result = ui.render(&mut terminal, &mut state);

        assert!(result.is_ok());
    }

    #[test]
    fn test_render_with_status_line() {
        use crate::document::node::{JsonNode, JsonValue};
        use crate::document::tree::JsonTree;
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let theme = get_builtin_theme("default-dark").unwrap();
        let ui = UI::new(theme);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let tree = JsonTree::new(JsonNode::new(JsonValue::Null));
        let mut state = EditorState::new_with_default_theme(tree);
        state.set_filename("test.json".to_string());
        state.mark_dirty();

        let result = ui.render(&mut terminal, &mut state);
        assert!(result.is_ok());

        // Verify the terminal was drawn to
        let buffer = terminal.backend().buffer();
        assert!(buffer.area().width > 0);
    }
}
