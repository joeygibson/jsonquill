//! Input handling for keyboard events and vim-style keybindings.

pub mod handler;
pub mod keys;

pub use handler::InputHandler;
pub use keys::InputEvent;
