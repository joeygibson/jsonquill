//! Crossterm backend for terminal input events.

use super::event::{BackendEvent, BackendKey, BackendMouse};
use anyhow::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind,
};
use std::time::Duration;

/// Reads terminal events using the crossterm backend.
pub struct EventReader;

impl Default for EventReader {
    fn default() -> Self {
        Self::new()
    }
}

impl EventReader {
    /// Creates a new EventReader.
    ///
    /// Crossterm manages its own input source (including CONIN$ on Windows),
    /// so no special setup is needed.
    pub fn new() -> Self {
        Self
    }

    /// Creates a new EventReader for when stdin was piped.
    ///
    /// Crossterm handles this case automatically on all platforms.
    pub fn new_with_tty() -> Result<Self> {
        Ok(Self)
    }

    /// Polls for a terminal event with a short timeout.
    ///
    /// Returns `Some(BackendEvent)` if an event occurred, `None` if timeout elapsed.
    pub fn poll_event(&mut self) -> Result<Option<BackendEvent>> {
        if event::poll(Duration::from_millis(100))? {
            let raw = event::read()?;
            Ok(translate_event(raw))
        } else {
            Ok(None)
        }
    }
}

/// Translates a crossterm `Event` into a `BackendEvent`.
fn translate_event(event: Event) -> Option<BackendEvent> {
    match event {
        Event::Key(key_event) => translate_key(key_event).map(BackendEvent::Key),
        Event::Mouse(mouse_event) => match mouse_event.kind {
            MouseEventKind::ScrollUp => Some(BackendEvent::Mouse(BackendMouse::WheelUp)),
            MouseEventKind::ScrollDown => Some(BackendEvent::Mouse(BackendMouse::WheelDown)),
            _ => None,
        },
        Event::Resize(w, h) => Some(BackendEvent::Resize(w, h)),
        _ => None,
    }
}

/// Translates a crossterm `KeyEvent` into a `BackendKey`.
fn translate_key(key: KeyEvent) -> Option<BackendKey> {
    // Only handle key press events — Windows sends Press, Release, and Repeat
    if key.kind != KeyEventKind::Press {
        return None;
    }

    // Handle Ctrl+key combinations first
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if let KeyCode::Char(c) = key.code {
            return Some(BackendKey::Ctrl(c));
        }
    }

    match key.code {
        // Map Enter → Char('\n') and Tab → Char('\t') to match termion's representation
        KeyCode::Enter => Some(BackendKey::Char('\n')),
        KeyCode::Tab => Some(BackendKey::Char('\t')),
        KeyCode::Char(c) => Some(BackendKey::Char(c)),
        KeyCode::Esc => Some(BackendKey::Esc),
        KeyCode::Backspace => Some(BackendKey::Backspace),
        KeyCode::Left => Some(BackendKey::Left),
        KeyCode::Right => Some(BackendKey::Right),
        KeyCode::Up => Some(BackendKey::Up),
        KeyCode::Down => Some(BackendKey::Down),
        KeyCode::Home => Some(BackendKey::Home),
        KeyCode::End => Some(BackendKey::End),
        KeyCode::PageUp => Some(BackendKey::PageUp),
        KeyCode::PageDown => Some(BackendKey::PageDown),
        KeyCode::F(n) => Some(BackendKey::F(n)),
        _ => None,
    }
}
