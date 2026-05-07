mod conversation;
mod input;
mod models;
mod navigation;
mod prompt;
mod settings;
mod state;
mod types;

pub use state::App;
pub use types::{ChatMessage, ModelEvent, PendingRequest};

pub(crate) use types::SPINNER_FRAMES;

#[cfg(test)]
mod tests;
