use std::path::PathBuf;

use crate::llm::LanguageModel;
use crate::prompt_rules::RulesTarget;

/// External work requested by a command handler.
#[derive(Clone, Debug)]
pub enum ExternalAction {
    /// Open a rules file in the configured editor, then reload rules when it exits.
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Setting {
    Theme,
    Layout,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingEdit<'a> {
    Theme(Option<&'a str>),
    Layout(Option<&'a str>),
}

/// Shared output surface used by slash commands.
pub trait CommandOutput {
    fn append_local_message(&mut self, command: &str, answer: String);
    fn set_status(&mut self, status: String);
}

/// Read-only model activity state.
pub trait ModelActivity {
    fn waiting_for_model(&self) -> bool;
}

/// Conversation lifecycle operations.
pub trait ConversationControl {
    fn clear_conversation(&mut self);
}

/// Model picker operations.
pub trait ModelPicker {
    fn open_models_picker(&mut self);
}

/// Read-only model catalog access.
pub trait ModelCatalog {
    fn models(&self) -> &[LanguageModel];
}

/// Rules state and edit operations.
pub trait RulesContext {
    fn default_rules_target(&self) -> RulesTarget;
    fn project_root(&self) -> Option<PathBuf>;
    fn prepare_rules_edit(&mut self, target: RulesTarget) -> Result<PathBuf, String>;
    fn queue_external_action(&mut self, action: ExternalAction);
    fn rules_report(&self) -> String;
    fn rules_enabled(&self) -> bool;
    fn set_rules_enabled(&mut self, enabled: bool);
    fn reload_rules(&mut self, enabled: bool);
    fn rules_status_line(&self) -> String;
}

/// Read-only conversation history access.
pub trait HistoryView {
    fn context_turn_limit(&self) -> usize;
    fn history_entries(&self) -> Vec<HistoryEntry<'_>>;
}

/// Conversation history export operations.
pub trait HistoryExport {
    fn save_history_report(
        &self,
        report: &str,
        requested_path: Option<&str>,
    ) -> Result<PathBuf, String>;
}

/// Conversation context-memory operations.
pub trait ContextMemory {
    fn include_latest_history_entry(&mut self, include: bool) -> Option<String>;
    fn clear_context_memory(&mut self) -> usize;
}

/// TUI setting operations.
pub trait SettingsContext {
    fn setting_report(&self, setting: Setting) -> String;
    fn set_setting(&mut self, setting: SettingEdit<'_>) -> Result<String, String>;
}

/// Help overlay operations.
pub trait HelpOverlay {
    fn open_help_overlay(&mut self);
}

/// App lifecycle operations.
pub trait AppLifecycle {
    fn quit(&mut self);
}

/// Staged model prompt operations.
pub trait PromptStaging {
    fn stage_prompt_for_model(&mut self, prompt: String);
}

/// Full execution boundary used only by the command registry/dispatcher.
pub trait CommandContext:
    CommandOutput
    + ModelActivity
    + ConversationControl
    + ModelPicker
    + ModelCatalog
    + RulesContext
    + HistoryView
    + HistoryExport
    + ContextMemory
    + SettingsContext
    + HelpOverlay
    + AppLifecycle
    + PromptStaging
{
}

impl<T> CommandContext for T where
    T: CommandOutput
        + ModelActivity
        + ConversationControl
        + ModelPicker
        + ModelCatalog
        + RulesContext
        + HistoryView
        + HistoryExport
        + ContextMemory
        + SettingsContext
        + HelpOverlay
        + AppLifecycle
        + PromptStaging
{
}
