//! Core library for the `ai-suite` workspace.
//!
//! This crate exposes the shared runtime, provider, routing, and streaming
//! primitives used by both the terminal and GUI frontends.

/// Process bootstrap and top-level command execution.
pub mod bootstrap;
/// Centralized error types and user-facing error rendering helpers.
pub mod errors;
/// Public extension interfaces for tool execution.
pub mod extensions;
/// High-level streaming APIs used by the frontends.
pub mod stream;
/// Public tool definitions and registries.
pub mod tools;

mod cli;
mod llm;
mod prompt_rules;
mod providers;
mod routing;
mod runtime;
mod storage;
mod subcommands;

pub use bootstrap::run;
pub use errors::{
    Error, Result, debug_mode_enabled, friendly_error, init_debug_mode_from_env, toggle_debug_mode,
};
pub use llm::ConversationTurn;
pub use stream::{
    ModelInfo, available_models, route_prompt, stream_prompt, stream_prompt_with_model,
};
