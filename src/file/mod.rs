//! File I/O operations for JSON documents.
//!
//! This module provides functionality to load JSON files from disk or stdin,
//! and save JSON trees back to files with atomic write operations and optional backups.

pub mod loader;
pub mod saver;
