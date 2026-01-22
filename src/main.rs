use anyhow::{Context, Result};
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
            (tree, None, false)
        }
    };

    // Setup terminal
    // When stdin was piped for data, we need to redirect stdin to /dev/tty for keyboard input
    #[cfg(unix)]
    {
        if _stdin_was_piped {
            use std::fs::File;
            use std::os::unix::io::AsRawFd;

            let tty = File::options()
                .read(true)
                .write(true)
                .open("/dev/tty")
                .context(
                    "Failed to open /dev/tty for keyboard input.\n\
                     When JSON is piped via stdin, jeditor requires a controlling terminal (/dev/tty) for keyboard input.\n\
                     This error typically occurs when running in:\n\
                     - Non-interactive shells or scripts\n\
                     - CI/CD environments without a PTY\n\
                     - Environments where the controlling terminal has been detached\n\n\
                     Try running from an interactive terminal session."
                )?;

            // Redirect stdin (fd 0) to point to the TTY
            // This allows crossterm to read keyboard events from the TTY
            let tty_fd = tty.as_raw_fd();
            unsafe {
                if libc::dup2(tty_fd, 0) == -1 {
                    anyhow::bail!("Failed to redirect stdin to /dev/tty");
                }
            }
            // Keep tty alive until after dup2 completes
            drop(tty);
        }
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
