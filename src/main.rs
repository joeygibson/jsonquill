use anyhow::{Context, Result};
use clap::Parser;
use ratatui::Terminal;
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::time::Duration;

use jsonquill::document::node::{JsonNode, JsonValue};
use jsonquill::document::tree::JsonTree;
use jsonquill::editor::state::EditorState;
use jsonquill::file::loader::{load_json_file, load_json_from_stdin};
use jsonquill::input::InputHandler;
use jsonquill::theme::get_builtin_theme;
use jsonquill::ui::UI;

/// JSONQuill - A terminal-based JSON editor with vim-style keybindings
#[derive(Parser)]
#[command(name = "jsonquill")]
#[command(version)]
#[command(about = "JSONQuill - A terminal-based JSON editor with vim-style keybindings", long_about = None)]
struct Cli {
    /// JSON file to edit (omit to read from stdin if piped, or create empty document if interactive)
    file: Option<String>,

    /// Theme name (default: default-dark)
    #[arg(short, long, default_value = "default-dark")]
    theme: String,
}

// --- Termion backend setup ---

#[cfg(feature = "backend-termion")]
fn setup_panic_hook() {
    use std::panic;

    let default_panic = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        use std::io::Write;

        let _ = write!(io::stderr(), "{}", termion::screen::ToMainScreen);
        let _ = write!(io::stderr(), "{}", termion::cursor::Show);
        let _ = io::stderr().flush();

        default_panic(panic_info);
    }));
}

#[cfg(feature = "backend-termion")]
#[allow(clippy::type_complexity)]
fn setup_terminal() -> Result<
    Terminal<
        ratatui::backend::TermionBackend<
            termion::screen::AlternateScreen<
                termion::input::MouseTerminal<termion::raw::RawTerminal<io::Stdout>>,
            >,
        >,
    >,
> {
    use ratatui::backend::TermionBackend;
    use termion::input::MouseTerminal;
    use termion::raw::IntoRawMode;
    use termion::screen::IntoAlternateScreen;

    let stdout = io::stdout()
        .into_raw_mode()
        .context("Failed to enable raw mode")?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = stdout
        .into_alternate_screen()
        .context("Failed to enter alternate screen")?;

    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

#[cfg(feature = "backend-termion")]
fn cleanup_terminal<B: std::io::Write>(
    terminal: &mut Terminal<ratatui::backend::TermionBackend<B>>,
) -> Result<()> {
    write!(terminal.backend_mut(), "{}", termion::cursor::Show)?;
    terminal.backend_mut().flush()?;
    Ok(())
}

// --- Crossterm backend setup ---

#[cfg(feature = "backend-crossterm")]
fn setup_panic_hook() {
    use std::panic;

    let default_panic = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        use crossterm::execute;
        use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};

        let _ = disable_raw_mode();
        let _ = execute!(io::stderr(), LeaveAlternateScreen);
        let _ = io::stderr().flush();

        default_panic(panic_info);
    }));
}

#[cfg(feature = "backend-crossterm")]
fn setup_terminal() -> Result<Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>> {
    use crossterm::event::EnableMouseCapture;
    use crossterm::execute;
    use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
    use ratatui::backend::CrosstermBackend;

    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

#[cfg(feature = "backend-crossterm")]
fn cleanup_terminal(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
) -> Result<()> {
    use crossterm::event::DisableMouseCapture;
    use crossterm::execute;
    use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

fn main() -> Result<()> {
    // Set up panic hook to restore terminal before showing panic info
    setup_panic_hook();

    let cli = Cli::parse();

    // Load file or create empty document BEFORE terminal setup
    // (stdin might be used for JSON data, so we need to read it before taking over the terminal)
    let (tree, filename, _stdin_was_piped) = if let Some(file_path) = cli.file {
        // Load from file, or create empty document if file doesn't exist
        if Path::new(&file_path).exists() {
            let tree = load_json_file(&file_path)?;
            (tree, Some(file_path), false)
        } else {
            let tree = JsonTree::new(JsonNode::new(JsonValue::Object(Vec::new())));
            (tree, Some(file_path), false)
        }
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
    let mut terminal = setup_terminal()?;

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

    let mut state = EditorState::new(tree, theme_name.to_string());
    if let Some(name) = filename {
        state.set_filename(name);
    }

    // Apply config settings (theme already set in constructor)
    state.set_show_line_numbers(config.show_line_numbers);
    state.set_relative_line_numbers(config.relative_line_numbers);
    state.set_enable_mouse(config.enable_mouse);
    state.set_create_backup(config.create_backup);

    // Main event loop
    let result = run_event_loop(&mut terminal, &mut ui, &mut input_handler, &mut state);

    // Cleanup
    cleanup_terminal(&mut terminal)?;

    result
}

fn run_event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    ui: &mut UI,
    input_handler: &mut InputHandler,
    state: &mut EditorState,
) -> Result<()>
where
    B::Error: Send + Sync + 'static,
{
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
