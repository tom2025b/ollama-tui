use crate::rules::RulesState;

use super::command_state::CommandState;
use super::routing_state::RoutingState;
use super::session_state::SessionState;
use super::ui_state::UiState;

/// Top-level application coordinator.
///
/// Domain-specific state lives in smaller structs. `App` keeps the event loop
/// simple by grouping those structs with process-level lifecycle state.
pub struct App {
    pub(super) routing: RoutingState,
    pub(super) commands: CommandState,
    pub(crate) session: SessionState,
    pub(crate) ui: UiState,

    /// Set to true when the user wants to leave the app.
    pub(crate) should_quit: bool,

    pub(super) rules: RulesState,
    pub(super) system_prompt: Option<String>,
}

impl App {
    /// Build a fresh app with the default router.
    pub fn new() -> Self {
        Self {
            routing: RoutingState::new(),
            commands: CommandState::new(),
            session: SessionState::new(),
            ui: UiState::new(),
            should_quit: false,
            rules: RulesState::load(),
            system_prompt: None,
        }
    }
}
