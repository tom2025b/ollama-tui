mod context;
mod local;

use super::{App, ChatMessage, ModelEvent, SPINNER_FRAMES};

impl App {
    /// Apply one streamed model event to the visible conversation.
    pub fn handle_model_event(&mut self, event: ModelEvent) {
        match event {
            ModelEvent::Token(token) => {
                if let Some(message) = self.active_message_mut() {
                    message.append_token(&token);
                }
            }
            ModelEvent::Finished => {
                if let Some(message) = self.active_message_mut() {
                    message.finish_streaming();
                }

                let model_name = self
                    .session
                    .active_model_name
                    .take()
                    .unwrap_or_else(|| "the selected model".to_string());
                self.session.waiting_for_model = false;
                self.ui.status = format!("Last answer came from {model_name}.");
            }
            ModelEvent::Failed(error) => {
                if let Some(message) = self.active_message_mut() {
                    message.fail_streaming(&error);
                }

                self.session.active_model_name = None;
                self.session.waiting_for_model = false;
                self.ui.status = "Model request failed.".to_string();
            }
        }
    }

    /// Advance lightweight UI activity while a request is in progress.
    pub fn tick(&mut self) {
        if self.session.waiting_for_model {
            self.session.activity_tick = self.session.activity_tick.wrapping_add(1);
            let spinner = SPINNER_FRAMES[self.session.activity_tick % SPINNER_FRAMES.len()];
            let model_name = self
                .session
                .active_model_name
                .as_deref()
                .unwrap_or("the selected model");
            self.ui.status = format!("{spinner} Streaming from {model_name}...");
        }
    }

    /// Return the model currently in use, or the most recent model if idle.
    pub fn current_model_label(&self) -> String {
        if let Some(model_name) = &self.session.active_model_name {
            return model_name.clone();
        }

        if let Some(pinned) = &self.routing.pinned_model {
            return format!("{} (pinned)", pinned.display_label());
        }

        self.session
            .history
            .iter()
            .rev()
            .find(|message| message.is_model_turn())
            .map(|message| message.model_name.clone())
            .unwrap_or_else(|| "none".to_string())
    }

    /// Current rules status for the status panel.
    pub fn rules_status_line(&self) -> String {
        self.rules.status_line()
    }

    /// Take the next external action requested by a local command.
    pub fn take_external_action(
        &mut self,
    ) -> Option<crate::subcommands::tui::slash_commands::ExternalAction> {
        self.commands.take_external_action()
    }

    fn active_message_mut(&mut self) -> Option<&mut ChatMessage> {
        self.session
            .history
            .iter_mut()
            .rev()
            .find(|message| message.in_progress)
    }
}
