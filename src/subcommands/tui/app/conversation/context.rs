use super::super::{App, MAX_CONTEXT_TURNS, MAX_STORED_TURNS};
use crate::llm::ConversationTurn;

const MAX_PERSISTENT_CONTEXT_TURNS: usize = 3;

impl App {
    pub(crate) fn conversation_context(&self) -> Vec<ConversationTurn> {
        let mut persistent = self
            .memory
            .turns()
            .into_iter()
            .rev()
            .take(MAX_PERSISTENT_CONTEXT_TURNS)
            .collect::<Vec<_>>();
        persistent.reverse();

        let remaining_session_turns = MAX_CONTEXT_TURNS.saturating_sub(persistent.len());
        let mut session_turns = self
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
            .take(remaining_session_turns)
            .map(|message| ConversationTurn {
                user: message.prompt.clone(),
                assistant: message.answer.clone(),
            })
            .collect::<Vec<_>>();

        session_turns.reverse();
        persistent.extend(session_turns);
        persistent
    }

    pub(crate) fn trim_history(&mut self) {
        let overflow = self.session.history.len().saturating_sub(MAX_STORED_TURNS);
        if overflow > 0 {
            self.session.history.drain(0..overflow);
        }
    }

    pub fn remember_latest_history_entry(&mut self) -> Result<Option<String>, String> {
        let Some(message) = self.session.history.iter_mut().rev().find(|message| {
            !message.is_local_message
                && !message.in_progress
                && !message.failed
                && !message.answer.trim().is_empty()
        }) else {
            return Ok(None);
        };

        message.include_in_context = true;
        self.memory
            .remember_turn(&message.prompt, &message.answer)
            .map_err(|error| format!("Could not save memory: {error}"))?;
        Ok(Some(message.prompt.clone()))
    }

    pub fn forget_latest_history_entry(&mut self) -> Result<Option<String>, String> {
        let Some(message) = self.session.history.iter_mut().rev().find(|message| {
            !message.is_local_message
                && !message.in_progress
                && !message.failed
                && !message.answer.trim().is_empty()
        }) else {
            return Ok(None);
        };

        message.include_in_context = false;
        self.memory
            .forget_latest_turn(&message.prompt)
            .map_err(|error| format!("Could not update memory: {error}"))?;
        Ok(Some(message.prompt.clone()))
    }

    pub fn pin_memory_note(&mut self, note: &str) -> Result<(), String> {
        self.memory
            .remember_note(note)
            .map_err(|error| format!("Could not save memory: {error}"))
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

    pub fn clear_persistent_memory(&mut self) -> Result<usize, String> {
        self.memory
            .clear()
            .map_err(|error| format!("Could not clear memory: {error}"))
    }
}
