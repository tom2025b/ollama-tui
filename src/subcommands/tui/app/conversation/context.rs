use super::super::{App, MAX_CONTEXT_TURNS, MAX_STORED_TURNS};
use crate::llm::ConversationTurn;

const MAX_PERSISTENT_CONTEXT_TURNS: usize = 3;

impl App {
    pub(crate) fn conversation_context(&self) -> Vec<ConversationTurn> {
        let mut context = self.persistent_context_turns();
        let remaining_session_turns = MAX_CONTEXT_TURNS.saturating_sub(context.len());
        let session_turns = self.session_context_turns(&context, remaining_session_turns);

        context.extend(session_turns);
        context
    }

    pub(crate) fn trim_history(&mut self) {
        let overflow = self.session.history.len().saturating_sub(MAX_STORED_TURNS);
        if overflow > 0 {
            self.session.history.drain(0..overflow);
        }
    }

    pub fn remember_latest_history_entry(&mut self) -> Result<Option<String>, String> {
        let Some(message) = self
            .session
            .history
            .iter_mut()
            .rev()
            .find(|message| message.has_completed_model_answer())
        else {
            return Ok(None);
        };

        message.remember_for_context();
        self.memory
            .remember_turn(&message.prompt, &message.answer)
            .map_err(|error| format!("Could not save memory: {error}"))?;
        Ok(Some(message.prompt.clone()))
    }

    pub fn forget_latest_history_entry(&mut self) -> Result<Option<String>, String> {
        let Some(message) = self
            .session
            .history
            .iter_mut()
            .rev()
            .find(|message| message.has_completed_model_answer())
        else {
            return Ok(None);
        };

        message.forget_context();
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
        for message in self
            .session
            .history
            .iter_mut()
            .filter(|message| message.is_ready_for_context())
        {
            message.forget_context();
            cleared += 1;
        }
        cleared
    }

    pub fn clear_persistent_memory(&mut self) -> Result<usize, String> {
        self.memory
            .clear()
            .map_err(|error| format!("Could not clear memory: {error}"))
    }

    fn persistent_context_turns(&self) -> Vec<ConversationTurn> {
        let mut turns = Vec::new();
        for turn in self.memory.turns().into_iter().rev() {
            if turns.len() == MAX_PERSISTENT_CONTEXT_TURNS {
                break;
            }
            push_unique_turn(&mut turns, turn);
        }
        turns.reverse();
        turns
    }

    fn session_context_turns(
        &self,
        persistent_turns: &[ConversationTurn],
        limit: usize,
    ) -> Vec<ConversationTurn> {
        if limit == 0 {
            return Vec::new();
        }

        let mut turns = Vec::new();
        for turn in self
            .session
            .history
            .iter()
            .rev()
            .filter_map(|message| message.context_turn())
        {
            if turns.len() == limit {
                break;
            }
            if !persistent_turns.contains(&turn) {
                push_unique_turn(&mut turns, turn);
            }
        }
        turns.reverse();
        turns
    }
}

fn push_unique_turn(turns: &mut Vec<ConversationTurn>, turn: ConversationTurn) {
    if !turns.contains(&turn) {
        turns.push(turn);
    }
}
