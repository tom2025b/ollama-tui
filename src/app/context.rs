use std::path::PathBuf;

use super::App;
use crate::command::handlers::{CommandContext, ExternalAction, HistoryEntry};
use crate::llm::LanguageModel;
use crate::rules::RulesTarget;

impl CommandContext for App {
    fn waiting_for_model(&self) -> bool {
        self.waiting_for_model
    }

    fn clear_conversation(&mut self) {
        self.history.clear();
        self.active_model_name = None;
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
        self.pending_external_action = Some(action);
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

    fn history_entries(&self) -> Vec<HistoryEntry<'_>> {
        self.history
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

    fn open_help_overlay(&mut self) {
        self.show_help = true;
        self.status = "Help is open. Press q, Esc, ?, or Ctrl-C to close it.".to_string();
    }

    fn set_status(&mut self, status: String) {
        self.status = status;
    }

    fn quit(&mut self) {
        App::quit(self);
    }
}
