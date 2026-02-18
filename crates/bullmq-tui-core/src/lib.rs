//! Shared TUI logic for BullMQ Monitor
//!
//! This crate implements the Elm Architecture (TEA) for the BullMQ TUI:
//! - **Model**: Application state
//! - **Message**: Actions/events
//! - **Update**: State transitions
//! - **View**: Rendering with Ratatui
//!
//! This separation allows the same logic to be used across:
//! - Terminal (Crossterm backend)
//! - Web (Ratzilla/WASM backend)
//! - Desktop (Tauri with embedded TUI)

pub mod effects;
pub mod message;
pub mod model;
pub mod update;
pub mod view;

// Re-exports for convenience
pub use effects::{Effect, UpdateResult};
pub use message::Message;
pub use model::{
    ConfirmAction, ConnectionStatus, JobDetailState, JobListState, Model, QueueListState,
    SearchState, StatusMessage, View,
};
pub use update::update;
pub use view::render;
