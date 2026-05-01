use super::App;
use crate::command::{CommandHelp, CommandSuggestion};

impl App {
    /// Add one typed character to the input buffer.
    pub fn push_input_char(&mut self, character: char) {
        self.input.push(character);
        self.suggestion_index = 0;
        self.suggestions_dismissed = false;
    }

    /// Remove the most recent typed character.
    pub fn backspace(&mut self) {
        self.input.pop();
        // Editing the input revives the popup if it was previously dismissed,
        // and resets the highlight so the user does not land on a stale row.
        self.suggestion_index = 0;
        self.suggestions_dismissed = false;
    }

    /// Clear the current input without submitting it.
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.status = "Input cleared.".to_string();
        self.suggestion_index = 0;
        self.suggestions_dismissed = false;
    }

    /// Slash-command suggestions that match the current input.
    pub fn command_suggestions(&self) -> Vec<CommandSuggestion> {
        if self.suggestions_dismissed {
            return Vec::new();
        }

        self.command_dispatcher.registry().suggestions(&self.input)
    }

    /// Command help rows from the command registry.
    pub fn command_help_entries(&self) -> Vec<CommandHelp> {
        self.command_dispatcher.registry().help_entries()
    }

    /// Currently highlighted suggestion index, clamped to the live match list.
    pub fn suggestion_index(&self) -> usize {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() {
            0
        } else {
            self.suggestion_index.min(suggestions.len() - 1)
        }
    }

    /// Move the popup highlight to the previous item, wrapping to the bottom.
    pub fn select_previous_suggestion(&mut self) {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() {
            return;
        }
        let current = self.suggestion_index.min(suggestions.len() - 1);
        self.suggestion_index = if current == 0 {
            suggestions.len() - 1
        } else {
            current - 1
        };
    }

    /// Move the popup highlight to the next item, wrapping to the top.
    pub fn select_next_suggestion(&mut self) {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() {
            return;
        }
        let current = self.suggestion_index.min(suggestions.len() - 1);
        self.suggestion_index = (current + 1) % suggestions.len();
    }

    /// Replace the input with the highlighted suggestion plus a trailing space.
    pub fn accept_suggestion(&mut self) -> bool {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() {
            return false;
        }
        let index = self.suggestion_index.min(suggestions.len() - 1);
        let selected = suggestions[index].name;

        self.input.clear();
        self.input.push_str(selected);
        self.input.push(' ');
        self.suggestion_index = 0;
        self.suggestions_dismissed = true;
        true
    }

    /// Hide the suggestion popup until the input is edited again.
    pub fn dismiss_suggestions(&mut self) {
        self.suggestions_dismissed = true;
    }
}
