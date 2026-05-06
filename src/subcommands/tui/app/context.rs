use std::path::PathBuf;

use super::App;
use crate::llm::LanguageModel;
use crate::rules::RulesTarget;
use crate::subcommands::tui::slash_commands::handlers::CommandContext;
use crate::subcommands::tui::slash_commands::handlers::{
    ExternalAction, HistoryEntry, Setting, SettingEdit,
};

impl CommandContext for App {
    fn waiting_for_model(&self) -> bool {
        self.session.waiting_for_model
    }

    fn clear_conversation(&mut self) {
        self.session.history.clear();
        self.session.active_model_name = None;
    }

    fn open_models_picker(&mut self) {
        App::open_models_picker(self);
    }

    fn append_local_message(&mut self, command: &str, answer: String) {
        App::append_local_message(self, command, answer);
    }

    fn models(&self) -> &[LanguageModel] {
        App::models(self)
    }

    fn default_rules_target(&self) -> RulesTarget {
        if self.rules.project_root().is_some() {
            RulesTarget::Project
        } else {
            RulesTarget::Global
        }
    }

    fn project_root(&self) -> Option<PathBuf> {
        self.rules.project_root().map(PathBuf::from)
    }

    fn prepare_rules_edit(&mut self, target: RulesTarget) -> Result<PathBuf, String> {
        self.rules
            .prepare_edit(target)
            .map_err(|error| error.to_string())
    }

    fn queue_external_action(&mut self, action: ExternalAction) {
        self.commands.queue_external_action(action);
    }

    fn rules_report(&self) -> String {
        self.rules.report()
    }

    fn rules_enabled(&self) -> bool {
        self.rules.enabled()
    }

    fn set_rules_enabled(&mut self, enabled: bool) {
        self.rules.set_enabled(enabled);
    }

    fn reload_rules(&mut self, enabled: bool) {
        self.rules = crate::rules::RulesState::load().with_enabled(enabled);
    }

    fn rules_status_line(&self) -> String {
        self.rules.status_line()
    }

    fn context_turn_limit(&self) -> usize {
        crate::subcommands::tui::app::MAX_CONTEXT_TURNS
    }

    fn history_entries(&self) -> Vec<HistoryEntry<'_>> {
        self.session
            .history
            .iter()
            .map(|message| HistoryEntry {
                prompt: &message.prompt,
                model_name: &message.model_name,
                route_reason: &message.route_reason,
                answer: &message.answer,
                in_progress: message.in_progress,
                failed: message.failed,
                include_in_context: message.include_in_context,
            })
            .collect()
    }

    fn include_latest_history_entry(&mut self, include: bool) -> Option<String> {
        App::include_latest_history_entry(self, include)
    }

    fn clear_context_memory(&mut self) -> usize {
        App::clear_context_memory(self)
    }

    fn setting_report(&self, setting: Setting) -> String {
        App::setting_report(self, setting)
    }

    fn set_setting(&mut self, setting: SettingEdit<'_>) -> Result<String, String> {
        App::set_setting(self, setting)
    }

    fn open_help_overlay(&mut self) {
        self.ui.show_help = true;
        self.ui.status = "Help is open. Press q, Esc, ?, or Ctrl-C to close it.".to_string();
    }

    fn set_status(&mut self, status: String) {
        self.ui.status = status;
    }

    fn quit(&mut self) {
        App::quit(self);
    }

    fn stage_prompt_for_model(&mut self, prompt: String) {
        self.commands.stage_prompt(prompt);
    }
}

impl App {
    /// Drain a prompt staged by the most recent slash command, if any.
    pub(crate) fn take_staged_command_prompt(&mut self) -> Option<String> {
        self.commands.take_staged_prompt()
    }
}
