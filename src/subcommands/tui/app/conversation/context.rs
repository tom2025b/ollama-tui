use super::super::App;
use crate::llm::ConversationTurn;

impl App {
    pub(crate) fn conversation_context(&self) -> Vec<ConversationTurn> {
        let limit = self.runtime.config().context().context_turns();
        let mut turns = self
            .session
            .history
            .iter()
            .rev()
            .filter(|message| {
                message.include_in_context
                    && !message.in_progress
                    && !message.failed
                    && !message.answer.trim().is_empty()
            })
            .take(limit)
            .map(|message| ConversationTurn {
                user: message.prompt.clone(),
                assistant: message.answer.clone(),
            })
            .collect::<Vec<_>>();

        turns.reverse();
        turns
    }

    pub(crate) fn trim_history(&mut self) {
        let limit = self.runtime.config().context().stored_turns();
        let overflow = self.session.history.len().saturating_sub(limit);
        if overflow > 0 {
            self.session.history.drain(0..overflow);
        }
    }

    pub fn include_latest_history_entry(&mut self, include: bool) -> Option<String> {
        let message = self.session.history.iter_mut().rev().find(|message| {
            !message.is_local_message
                && !message.in_progress
                && !message.failed
                && !message.answer.trim().is_empty()
        })?;

        message.include_in_context = include;
        Some(message.prompt.clone())
    }

    pub fn clear_context_memory(&mut self) -> usize {
        let mut cleared = 0;
        for message in self.session.history.iter_mut().filter(|message| {
            message.include_in_context && !message.is_local_message && !message.in_progress
        }) {
            message.include_in_context = false;
            cleared += 1;
        }
        cleared
    }
}
