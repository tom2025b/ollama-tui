use super::super::{App, ChatMessage};

impl App {
    /// Add a local command result to the visible history without sending it later.
    pub fn append_local_message(&mut self, command: &str, answer: String) {
        self.session.history.push(ChatMessage {
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
