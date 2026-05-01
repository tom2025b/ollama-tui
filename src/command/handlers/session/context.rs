use std::path::PathBuf;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Setting {
    Theme,
    Layout,
    Voice,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingEdit<'a> {
    Theme(Option<&'a str>),
    Layout(Option<&'a str>),
    VoiceEnabled(bool),
    VoiceSpeed(&'a str),
    VoiceMode(&'a str),
}

/// Execution boundary used by command handlers.
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
    fn context_turn_limit(&self) -> usize;
    fn history_entries(&self) -> Vec<HistoryEntry<'_>>;
    fn include_latest_history_entry(&mut self, include: bool) -> Option<String>;
    fn clear_context_memory(&mut self) -> usize;
    fn setting_report(&self, setting: Setting) -> String;
    fn set_setting(&mut self, setting: SettingEdit<'_>) -> Result<String, String>;
    fn open_help_overlay(&mut self);
    fn set_status(&mut self, status: String);
    fn quit(&mut self);
}
