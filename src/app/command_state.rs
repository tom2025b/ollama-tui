use std::collections::VecDeque;

use crate::command::{CommandDispatcher, CommandRegistry, ExternalAction};

/// Slash-command state, including suggestions and queued command side effects.
pub(super) struct CommandState {
    pub(super) command_dispatcher: CommandDispatcher,
    external_actions: VecDeque<ExternalAction>,
    pub(super) suggestion_index: usize,
    pub(super) suggestions_dismissed: bool,
}

impl CommandState {
    pub(super) fn new() -> Self {
        Self {
            command_dispatcher: CommandDispatcher::new(CommandRegistry::default()),
            external_actions: VecDeque::new(),
            suggestion_index: 0,
            suggestions_dismissed: false,
        }
    }

    pub(super) fn reset_suggestions(&mut self) {
        self.suggestion_index = 0;
        self.suggestions_dismissed = false;
    }

    pub(super) fn dismiss_suggestions(&mut self) {
        self.suggestions_dismissed = true;
    }

    pub(super) fn queue_external_action(&mut self, action: ExternalAction) {
        self.external_actions.push_back(action);
    }

    pub(super) fn take_external_action(&mut self) -> Option<ExternalAction> {
        self.external_actions.pop_front()
    }
}
