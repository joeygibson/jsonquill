//! Backend abstraction for terminal input.
//!
//! This module provides a unified event type (`BackendEvent`) and platform-specific
//! `EventReader` implementations that translate raw backend events into `BackendEvent`.

mod event;

#[cfg(feature = "backend-termion")]
mod termion_backend;

#[cfg(feature = "backend-crossterm")]
mod crossterm_backend;

pub use event::{BackendEvent, BackendKey, BackendMouse};

#[cfg(feature = "backend-termion")]
pub use termion_backend::EventReader;

#[cfg(feature = "backend-crossterm")]
pub use crossterm_backend::EventReader;
