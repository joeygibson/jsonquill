use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, IsTerminal};
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

    // Load file or create empty document BEFORE terminal setup
    // (stdin might be used for JSON data, so we need to read it before taking over the terminal)
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

    // Setup terminal AFTER loading data (so stdin is available for JSON if needed)
    // Check if stdin is a terminal - if not, JSON was piped/redirected
    if !io::stdin().is_terminal() {
        anyhow::bail!(
            "stdin is not a terminal (piped or redirected input detected).\n\n\
             Piping or redirecting JSON via stdin is not supported.\n\
             The TUI requires stdin for keyboard input, which conflicts with data input.\n\n\
             Please pass the file path directly instead:\n   \
                   jeditor foo.json"
        );
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Load config
    use jeditor::config::Config;
    let config = Config::load();

    // Initialize components
    // CLI theme overrides config theme
    let theme_name = if !cli.theme.is_empty() {
        &cli.theme
    } else {
        &config.theme
    };

    let theme = get_builtin_theme(theme_name).unwrap_or_else(|| {
        eprintln!("Warning: Theme '{}' not found, using default-dark", theme_name);
        get_builtin_theme("default-dark").unwrap()
    });
    let mut ui = UI::new(theme);
    let input_handler = InputHandler::new();

    let mut state = EditorState::new(tree);
    if let Some(name) = filename {
        state.set_filename(name);
    }

    // Apply config settings
    state.set_current_theme(theme_name.to_string());
    state.set_show_line_numbers(config.show_line_numbers);

    // Main event loop
    let result = run_event_loop(&mut terminal, &mut ui, &input_handler, &mut state);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    ui: &mut UI,
    input_handler: &InputHandler,
    state: &mut EditorState,
) -> Result<()> {
    loop {
        // Check for pending theme changes
        if let Some(theme_name) = state.take_pending_theme() {
            ui.set_theme(&theme_name);
        }

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
