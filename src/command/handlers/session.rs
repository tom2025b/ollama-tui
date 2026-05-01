use std::path::PathBuf;

use crate::command::parser::ParsedCommand;
use crate::llm::LanguageModel;
use crate::rules::RulesTarget;

/// External work requested by a command handler.
#[derive(Clone, Debug)]
pub enum ExternalAction {
    /// Open a rules file in nano, then reload rules when nano exits.
    EditRules {
        /// Which rules file is being edited.
        target: RulesTarget,

        /// File path to open.
        path: PathBuf,
    },
}

/// Read-only command view of one visible history entry.
#[derive(Clone, Copy, Debug)]
pub struct HistoryEntry<'a> {
    pub prompt: &'a str,
    pub model_name: &'a str,
    pub route_reason: &'a str,
    pub answer: &'a str,
    pub in_progress: bool,
    pub failed: bool,
    pub include_in_context: bool,
}

/// Execution boundary used by command handlers.
///
/// `App` implements this trait today. Keeping handlers behind a narrow context
/// trait lets command execution move out of the app module incrementally
/// without forcing command code to depend on the full TUI state shape.
pub trait CommandContext {
    fn waiting_for_model(&self) -> bool;
    fn clear_conversation(&mut self);
    fn open_models_picker(&mut self);
    fn append_local_message(&mut self, command: &str, answer: String);
    fn models(&self) -> &[LanguageModel];
    fn default_rules_target(&self) -> RulesTarget;
    fn project_root(&self) -> Option<PathBuf>;
    fn prepare_rules_edit(&mut self, target: RulesTarget) -> Result<PathBuf, String>;
    fn queue_external_action(&mut self, action: ExternalAction);
    fn rules_report(&self) -> String;
    fn rules_enabled(&self) -> bool;
    fn set_rules_enabled(&mut self, enabled: bool);
    fn reload_rules(&mut self, enabled: bool);
    fn rules_status_line(&self) -> String;
    fn history_entries(&self) -> Vec<HistoryEntry<'_>>;
    fn open_help_overlay(&mut self);
    fn set_status(&mut self, status: String);
    fn quit(&mut self);
}

pub fn clear_conversation_command(context: &mut dyn CommandContext, _command: &ParsedCommand) {
    if context.waiting_for_model() {
        context.set_status("Cannot clear while a model is answering.".to_string());
        return;
    }

    context.clear_conversation();
    context.set_status("Conversation cleared.".to_string());
}

pub fn open_models_command(context: &mut dyn CommandContext, _command: &ParsedCommand) {
    context.open_models_picker();
}

pub fn open_help_command(context: &mut dyn CommandContext, _command: &ParsedCommand) {
    context.open_help_overlay();
}

pub fn quit_command(context: &mut dyn CommandContext, _command: &ParsedCommand) {
    context.quit();
}

pub fn unknown_command(
    context: &mut dyn CommandContext,
    command: &ParsedCommand,
    available_commands: &str,
) {
    context.append_local_message(
        command.raw(),
        format!("Unknown command. Available commands: {available_commands}."),
    );
    context.set_status("Unknown command.".to_string());
}
