//! Backend-agnostic event types for terminal input.

/// A terminal input event from the backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendEvent {
    /// A keyboard event.
    Key(BackendKey),
    /// A mouse event.
    Mouse(BackendMouse),
    /// Terminal was resized to (width, height).
    Resize(u16, u16),
}

/// A keyboard key from the backend.
///
/// Enter is represented as `Char('\n')` and Tab as `Char('\t')` to match
/// termion's representation, minimizing changes in the input handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendKey {
    /// A regular character (includes '\n' for Enter, '\t' for Tab).
    Char(char),
    /// A Ctrl+character combination.
    Ctrl(char),
    /// The Escape key.
    Esc,
    /// The Backspace key.
    Backspace,
    /// Left arrow key.
    Left,
    /// Right arrow key.
    Right,
    /// Up arrow key.
    Up,
    /// Down arrow key.
    Down,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page Up key.
    PageUp,
    /// Page Down key.
    PageDown,
    /// Function key (F1, F2, etc.).
    F(u8),
}

/// A mouse event from the backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendMouse {
    /// Mouse wheel scrolled up.
    WheelUp,
    /// Mouse wheel scrolled down.
    WheelDown,
}
