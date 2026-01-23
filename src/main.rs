use anyhow::{Context, Result};
use clap::Parser;
use ratatui::{backend::TermionBackend, Terminal};
use std::io::{self, IsTerminal, Write};
use std::time::Duration;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;

use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;
use jsonquill::file::loader::{load_json_file, load_json_from_stdin};
use jsonquill::input::InputHandler;
use jsonquill::theme::get_builtin_theme;
use jsonquill::ui::UI;

/// JSON Quill - A terminal-based JSON editor with vim-style keybindings
#[derive(Parser)]
#[command(name = "jsonquill")]
#[command(version)]
#[command(about = "JSON Quill - A terminal-based JSON editor with vim-style keybindings", long_about = None)]
struct Cli {
    /// JSON file to edit (omit to read from stdin if piped, or create empty document if interactive)
    file: Option<String>,

    /// Theme name (default: default-dark)
    #[arg(short, long, default_value = "default-dark")]
    theme: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load file or create empty document BEFORE terminal setup
    // (stdin might be used for JSON data, so we need to read it before taking over the terminal)
    let (tree, filename, _stdin_was_piped) = if let Some(file_path) = cli.file {
        // Load from file
        let tree = load_json_file(&file_path)?;
        (tree, Some(file_path), false)
    } else {
        // No filename provided - check if stdin has piped data
        if !io::stdin().is_terminal() {
            // Stdin is piped - read JSON from it
            let tree = load_json_from_stdin()?;
            (tree, None, true)
        } else {
            // Interactive mode - create sample document with nested structure
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
                ("count".to_string(), JsonNode::new(JsonValue::Number(42.0))),
                (
                    "active".to_string(),
                    JsonNode::new(JsonValue::Boolean(true)),
                ),
            ];

            let tree = JsonTree::new(JsonNode::new(JsonValue::Object(obj)));
            (tree, None, false)
        }
    };

    // Setup terminal
    // Termion can use /dev/tty directly when stdin is piped, no redirection needed
    let stdout = io::stdout()
        .into_raw_mode()
        .context("Failed to enable raw mode")?
        .into_alternate_screen()
        .context("Failed to enter alternate screen")?;

    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Load config
    use jsonquill::config::Config;
    let config = Config::load();

    // Initialize components
    // CLI theme overrides config theme
    let theme_name = if !cli.theme.is_empty() {
        &cli.theme
    } else {
        &config.theme
    };

    let theme = get_builtin_theme(theme_name).unwrap_or_else(|| {
        eprintln!(
            "Warning: Theme '{}' not found, using default-dark",
            theme_name
        );
        get_builtin_theme("default-dark").unwrap()
    });
    let mut ui = UI::new(theme);
    let mut input_handler = if _stdin_was_piped {
        InputHandler::new_with_tty()
            .context("Failed to open /dev/tty for keyboard input when stdin was piped")?
    } else {
        InputHandler::new()
    };

    let mut state = EditorState::new(tree);
    if let Some(name) = filename {
        state.set_filename(name);
    }

    // Apply config settings
    state.set_current_theme(theme_name.to_string());
    state.set_show_line_numbers(config.show_line_numbers);

    // Main event loop
    let result = run_event_loop(&mut terminal, &mut ui, &mut input_handler, &mut state);

    // Cleanup
    // Termion handles cleanup automatically through Drop guards
    // But we still want to show the cursor before exiting
    write!(terminal.backend_mut(), "{}", termion::cursor::Show)?;
    terminal.backend_mut().flush()?;

    result
}

fn run_event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    ui: &mut UI,
    input_handler: &mut InputHandler,
    state: &mut EditorState,
) -> Result<()> {
    loop {
        // Check for pending theme changes
        if let Some(theme_name) = state.take_pending_theme() {
            ui.set_theme(&theme_name);
        }

        // Update cursor blink state
        state.update_cursor_blink();

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
