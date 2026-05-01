use super::{App, ChatMessage, MAX_CONTEXT_TURNS, MAX_STORED_TURNS, ModelEvent, SPINNER_FRAMES};
use crate::llm::ConversationTurn;

impl App {
    /// Apply one streamed model event to the visible conversation.
    pub fn handle_model_event(&mut self, event: ModelEvent) {
        match event {
            ModelEvent::Token(token) => {
                if let Some(message) = self.active_message_mut() {
                    message.answer.push_str(&token);
                }
            }
            ModelEvent::Finished => {
                if let Some(message) = self.active_message_mut() {
                    message.in_progress = false;
                }

                let model_name = self
                    .active_model_name
                    .take()
                    .unwrap_or_else(|| "the selected model".to_string());
                self.waiting_for_model = false;
                self.status = format!("Last answer came from {model_name}.");
            }
            ModelEvent::Failed(error) => {
                if let Some(message) = self.active_message_mut() {
                    if !message.answer.trim().is_empty() {
                        message.answer.push_str("\n\n");
                    }
                    message.answer.push_str(&error);
                    message.in_progress = false;
                    message.failed = true;
                }

                self.active_model_name = None;
                self.waiting_for_model = false;
                self.status = "Model request failed.".to_string();
            }
        }
    }

    /// Advance lightweight UI activity while a request is in progress.
    pub fn tick(&mut self) {
        if !self.waiting_for_model {
            return;
        }

        self.activity_tick = self.activity_tick.wrapping_add(1);
        let spinner = SPINNER_FRAMES[self.activity_tick % SPINNER_FRAMES.len()];
        let model_name = self
            .active_model_name
            .as_deref()
            .unwrap_or("the selected model");
        self.status = format!("{spinner} Streaming from {model_name}...");
    }

    /// Return the model currently in use, or the most recent model if idle.
    pub fn current_model_label(&self) -> String {
        if let Some(model_name) = &self.active_model_name {
            return model_name.clone();
        }

        if let Some(pinned) = &self.pinned_model {
            return format!("{} (pinned)", pinned.display_label());
        }

        self.history
            .iter()
            .rev()
            .find(|message| message.include_in_context)
            .map(|message| message.model_name.clone())
            .unwrap_or_else(|| "none".to_string())
    }

    /// Current rules status for the status panel.
    pub fn rules_status_line(&self) -> String {
        self.rules.status_line()
    }

    /// Take the next external action requested by a local command.
    pub fn take_external_action(&mut self) -> Option<crate::command::ExternalAction> {
        self.pending_external_action.take()
    }

    pub(crate) fn conversation_context(&self) -> Vec<ConversationTurn> {
        let mut turns = self
            .history
            .iter()
            .rev()
            .filter(|message| {
                message.include_in_context
                    && !message.in_progress
                    && !message.failed
                    && !message.answer.trim().is_empty()
            })
            .take(MAX_CONTEXT_TURNS)
            .map(|message| ConversationTurn {
                user: message.prompt.clone(),
                assistant: message.answer.clone(),
            })
            .collect::<Vec<_>>();

        turns.reverse();
        turns
    }

    pub(crate) fn trim_history(&mut self) {
        let overflow = self.history.len().saturating_sub(MAX_STORED_TURNS);
        if overflow > 0 {
            self.history.drain(0..overflow);
        }
    }

    fn active_message_mut(&mut self) -> Option<&mut ChatMessage> {
        self.history
            .iter_mut()
            .rev()
            .find(|message| message.in_progress)
    }

    /// Add a local command result to the visible history without sending it later.
    pub fn append_local_message(&mut self, command: &str, answer: String) {
        self.history.push(ChatMessage {
            prompt: command.to_string(),
            model_name: "ollama-me".to_string(),
            route_reason: "Local command. Not sent to any model.".to_string(),
            answer,
            in_progress: false,
            failed: false,
            include_in_context: false,
        });
        self.trim_history();
    }
}
