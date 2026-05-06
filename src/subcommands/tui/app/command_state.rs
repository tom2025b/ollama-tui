use std::collections::VecDeque;

use crate::subcommands::tui::slash_commands::{CommandDispatcher, CommandRegistry, ExternalAction};

/// Slash-command state, including suggestions and queued command side effects.
pub(super) struct CommandState {
    pub(super) command_dispatcher: CommandDispatcher,
    external_actions: VecDeque<ExternalAction>,
    pub(super) suggestion_index: usize,
    pub(super) suggestions_dismissed: bool,

    /// Prompt that a slash command produced for the next model turn, if any.
    /// Drained by `submit_prompt` so commands like /fix actually reach a model.
    staged_prompt: Option<String>,
}

impl CommandState {
    pub(super) fn new() -> Self {
        Self {
            command_dispatcher: CommandDispatcher::new(CommandRegistry::default()),
            external_actions: VecDeque::new(),
            suggestion_index: 0,
            suggestions_dismissed: false,
            staged_prompt: None,
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

    pub(super) fn stage_prompt(&mut self, prompt: String) {
        self.staged_prompt = Some(prompt);
    }

    pub(super) fn take_staged_prompt(&mut self) -> Option<String> {
        self.staged_prompt.take()
    }
}
