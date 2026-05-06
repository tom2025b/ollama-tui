use super::super::{App, ChatMessage};

impl App {
    /// Add a local command result to the visible history without sending it later.
    pub fn append_local_message(&mut self, command: &str, answer: String) {
        self.session
            .history
            .push(ChatMessage::local_output(command, answer));
        self.trim_history();
    }
}
