use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

use jeditor::document::node::{JsonNode, JsonValue};
use jeditor::document::tree::JsonTree;
use jeditor::editor::state::EditorState;
use jeditor::input::InputHandler;
use jeditor::theme::get_builtin_theme;
use jeditor::ui::UI;

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

    // Create sample document with nested structure
    let user_obj = vec![
        ("name".to_string(), JsonNode::new(JsonValue::String("Alice".to_string()))),
        ("email".to_string(), JsonNode::new(JsonValue::String("alice@example.com".to_string()))),
    ];

    let obj = vec![
        ("user".to_string(), JsonNode::new(JsonValue::Object(user_obj))),
        ("count".to_string(), JsonNode::new(JsonValue::Number(42.0))),
        ("active".to_string(), JsonNode::new(JsonValue::Boolean(true))),
    ];

    let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));
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
