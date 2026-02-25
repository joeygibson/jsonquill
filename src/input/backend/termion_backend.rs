//! Termion backend for terminal input events.

use super::event::{BackendEvent, BackendKey, BackendMouse};
use anyhow::{Context, Result};
use std::fs::File;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::event::{Event, Key, MouseButton, MouseEvent};
use termion::input::{Events, TermRead};

/// Reads terminal events using the termion backend.
///
/// Uses a background thread to read events from termion's blocking iterator,
/// forwarding them through a channel so `poll_event` can use a timeout.
pub struct EventReader {
    rx: mpsc::Receiver<BackendEvent>,
}

impl Default for EventReader {
    fn default() -> Self {
        Self::new()
    }
}

impl EventReader {
    /// Creates a new EventReader that reads from stdin.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            read_events_from(io::stdin().events(), tx);
        });
        Self { rx }
    }

    /// Creates a new EventReader that reads from /dev/tty.
    /// Use this when stdin has been consumed for piped data.
    pub fn new_with_tty() -> Result<Self> {
        let tty_file = File::options()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .context("Failed to open /dev/tty for keyboard input")?;

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            read_events_from(tty_file.events(), tx);
        });
        Ok(Self { rx })
    }

    /// Polls for a terminal event with a timeout.
    ///
    /// Returns `Some(BackendEvent)` if an event occurred, `None` if timeout elapsed.
    pub fn poll_event(&mut self) -> Result<Option<BackendEvent>> {
        match self.rx.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => Ok(Some(event)),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(mpsc::RecvTimeoutError::Disconnected) => Ok(None),
        }
    }
}

/// Reads events from a termion event iterator and sends them through the channel.
fn read_events_from<R: io::Read>(events: Events<R>, tx: mpsc::Sender<BackendEvent>) {
    for event in events.flatten() {
        if let Some(backend_event) = translate_event(event) {
            if tx.send(backend_event).is_err() {
                break;
            }
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
