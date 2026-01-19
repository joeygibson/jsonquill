use anyhow::Result;
use clap::Parser;
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
use jeditor::file::loader::{load_json_file, load_json_from_stdin};
use jeditor::input::InputHandler;
use jeditor::theme::get_builtin_theme;
use jeditor::ui::UI;

/// A terminal-based JSON editor with vim-style keybindings
#[derive(Parser)]
#[command(name = "jeditor")]
#[command(version)]
#[command(about = "A terminal-based JSON editor with vim-style keybindings", long_about = None)]
struct Cli {
    /// JSON file to edit (omit for empty document, use - for stdin)
    file: Option<String>,

    /// Theme name (default: default-dark)
    #[arg(short, long, default_value = "default-dark")]
    theme: String,
}

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
    let theme = get_builtin_theme(&cli.theme).unwrap_or_else(|| {
        eprintln!("Warning: Theme '{}' not found, using default-dark", cli.theme);
        get_builtin_theme("default-dark").unwrap()
    });
    let ui = UI::new(theme);
    let input_handler = InputHandler::new();

    // Load file or create empty document
    let (tree, filename) = if let Some(file_path) = cli.file {
        if file_path == "-" {
            // Load from stdin
            let tree = load_json_from_stdin()?;
            (tree, None)
        } else {
            // Load from file
            let tree = load_json_file(&file_path)?;
            (tree, Some(file_path))
        }
    } else {
        // Create sample document with nested structure
        let user_obj = vec![
            (
                "name".to_string(),
                JsonNode::new(JsonValue::String("Alice".to_string())),
            ),
            (
                "email".to_string(),
                JsonNode::new(JsonValue::String("alice@example.com".to_string())),
            ),
        ];

        let obj = vec![
            (
                "user".to_string(),
                JsonNode::new(JsonValue::Object(user_obj)),
            ),
            (
                "count".to_string(),
                JsonNode::new(JsonValue::Number(42.0)),
            ),
            (
                "active".to_string(),
                JsonNode::new(JsonValue::Boolean(true)),
            ),
        ];

        let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));
        (tree, None)
    };

    let mut state = EditorState::new(tree);
    if let Some(name) = filename {
        state.set_filename(name);
    }

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
