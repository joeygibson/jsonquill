//! Termion backend for terminal input events.

use super::event::{BackendEvent, BackendKey, BackendMouse};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{self, Stdin};
use termion::event::{Event, Key, MouseButton, MouseEvent};
use termion::input::{Events, TermRead};

/// Event source for reading terminal events via termion.
enum EventSource {
    /// Reading from stdin.
    Stdin(Events<Stdin>),
    /// Reading from /dev/tty (when stdin was piped).
    Tty(Events<File>),
}

/// Reads terminal events using the termion backend.
pub struct EventReader {
    events: EventSource,
}

impl Default for EventReader {
    fn default() -> Self {
        Self::new()
    }
}

impl EventReader {
    /// Creates a new EventReader that reads from stdin.
    pub fn new() -> Self {
        Self {
            events: EventSource::Stdin(io::stdin().events()),
        }
    }

    /// Creates a new EventReader that reads from /dev/tty.
    /// Use this when stdin has been consumed for piped data.
    pub fn new_with_tty() -> Result<Self> {
        let tty_file = File::options()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .context("Failed to open /dev/tty for keyboard input")?;

        Ok(Self {
            events: EventSource::Tty(tty_file.events()),
        })
    }

    /// Polls for a terminal event.
    ///
    /// Returns `Some(BackendEvent)` if an event occurred, `None` if no event is available.
    pub fn poll_event(&mut self) -> Result<Option<BackendEvent>> {
        let raw_event = match &mut self.events {
            EventSource::Stdin(events) => events.next(),
            EventSource::Tty(events) => events.next(),
        };

        if let Some(event_result) = raw_event {
            let event = event_result?;
            Ok(translate_event(event))
        } else {
            Ok(None)
        }
    }
}

/// Translates a termion `Event` into a `BackendEvent`.
fn translate_event(event: Event) -> Option<BackendEvent> {
    match event {
        Event::Key(key) => Some(BackendEvent::Key(translate_key(key))),
        Event::Mouse(mouse) => translate_mouse(mouse).map(BackendEvent::Mouse),
        _ => None,
    }
}

/// Translates a termion `Key` into a `BackendKey`.
fn translate_key(key: Key) -> BackendKey {
    match key {
        Key::Char(c) => BackendKey::Char(c),
        Key::Ctrl(c) => BackendKey::Ctrl(c),
        Key::Esc => BackendKey::Esc,
        Key::Backspace => BackendKey::Backspace,
        Key::Left => BackendKey::Left,
        Key::Right => BackendKey::Right,
        Key::Up => BackendKey::Up,
        Key::Down => BackendKey::Down,
        Key::Home => BackendKey::Home,
        Key::End => BackendKey::End,
        Key::PageUp => BackendKey::PageUp,
        Key::PageDown => BackendKey::PageDown,
        Key::F(n) => BackendKey::F(n),
        _ => BackendKey::Esc, // fallback for unmapped keys
    }
}

/// Translates a termion `MouseEvent` into a `BackendMouse`.
fn translate_mouse(mouse: MouseEvent) -> Option<BackendMouse> {
    match mouse {
        MouseEvent::Press(MouseButton::WheelUp, _, _) => Some(BackendMouse::WheelUp),
        MouseEvent::Press(MouseButton::WheelDown, _, _) => Some(BackendMouse::WheelDown),
        _ => None,
    }
}
