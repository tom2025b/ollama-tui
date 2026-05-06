use crate::prompt_rules::RulesState;
use crate::runtime::Runtime;

use super::command_state::CommandState;
use super::routing_state::RoutingState;
use super::session_state::SessionState;
use super::ui_state::UiState;

/// Top-level application coordinator.
///
/// Domain-specific state lives in smaller structs. `App` keeps the event loop
/// simple by grouping those structs with process-level lifecycle state.
pub struct App {
    pub(super) runtime: Runtime,
    pub(super) routing: RoutingState,
    pub(super) commands: CommandState,
    pub(crate) session: SessionState,
    pub(crate) ui: UiState,

    /// Set to true when the user wants to leave the app.
    pub(crate) should_quit: bool,

    pub(super) rules: RulesState,
}

impl App {
    /// Build a fresh app with the default router.
    #[cfg(test)]
    pub fn new() -> Self {
        Self::with_runtime(Runtime::load())
    }

    pub(crate) fn with_runtime(runtime: Runtime) -> Self {
        let routing = RoutingState::new(runtime.config());
        let rules = RulesState::load(runtime.paths());

        Self {
            runtime,
            routing,
            commands: CommandState::new(),
            session: SessionState::new(),
            ui: UiState::new(),
            should_quit: false,
            rules,
        }
    }
}
