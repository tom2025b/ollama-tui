use super::App;
use crate::subcommands::tui::slash_commands::{CommandHelp, CommandSuggestion};

impl App {
    /// Add one typed character to the input buffer.
    pub fn push_input_char(&mut self, character: char) {
        let cursor = self.input_cursor();
        self.session.input.insert(cursor, character);
        self.session.input_cursor = cursor + character.len_utf8();
        self.commands.reset_suggestions();
    }

    /// Remove the most recent typed character.
    pub fn backspace(&mut self) {
        let cursor = self.input_cursor();
        if cursor == 0 {
            return;
        }

        let previous = previous_cursor_position(&self.session.input, cursor);
        self.session.input.drain(previous..cursor);
        self.session.input_cursor = previous;
        // Editing the input revives the popup if it was previously dismissed,
        // and resets the highlight so the user does not land on a stale row.
        self.commands.reset_suggestions();
    }

    /// Clear the current input without submitting it.
    pub fn clear_input(&mut self) {
        self.session.input.clear();
        self.session.input_cursor = 0;
        self.ui.status = "Input cleared.".to_string();
        self.commands.reset_suggestions();
    }

    /// Move the prompt cursor one character to the left.
    pub fn move_input_cursor_left(&mut self) {
        self.session.input_cursor =
            previous_cursor_position(&self.session.input, self.input_cursor());
    }

    /// Move the prompt cursor one character to the right.
    pub fn move_input_cursor_right(&mut self) {
        self.session.input_cursor = next_cursor_position(&self.session.input, self.input_cursor());
    }

    /// Current prompt cursor, clamped to a valid UTF-8 boundary.
    pub(crate) fn input_cursor(&self) -> usize {
        clamp_cursor(&self.session.input, self.session.input_cursor)
    }

    /// Slash-command suggestions that match the current input.
    pub fn command_suggestions(&self) -> Vec<CommandSuggestion> {
        if self.commands.suggestions_dismissed {
            return Vec::new();
        }

        self.commands.registry.suggestions(&self.session.input)
    }

    /// Command help rows from the command registry.
    pub fn command_help_entries(&self) -> Vec<CommandHelp> {
        self.commands.registry.help_entries()
    }

    /// Currently highlighted suggestion index, clamped to the live match list.
    pub fn suggestion_index(&self) -> usize {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() {
            0
        } else {
            self.commands.suggestion_index.min(suggestions.len() - 1)
        }
    }

    /// Move the popup highlight to the previous item, wrapping to the bottom.
    pub fn select_previous_suggestion(&mut self) {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() {
            return;
        }
        let current = self.commands.suggestion_index.min(suggestions.len() - 1);
        self.commands.suggestion_index = if current == 0 {
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
        let current = self.commands.suggestion_index.min(suggestions.len() - 1);
        self.commands.suggestion_index = (current + 1) % suggestions.len();
    }

    /// Replace the input with the highlighted suggestion plus a trailing space.
    pub fn accept_suggestion(&mut self) -> bool {
        let suggestions = self.command_suggestions();
        if suggestions.is_empty() {
            return false;
        }
        let index = self.commands.suggestion_index.min(suggestions.len() - 1);
        let selected = suggestions[index].name;

        self.session.input.clear();
        self.session.input.push_str(selected);
        self.session.input.push(' ');
        self.session.input_cursor = self.session.input.len();
        self.commands.suggestion_index = 0;
        self.commands.dismiss_suggestions();
        true
    }

    /// Hide the suggestion popup until the input is edited again.
    pub fn dismiss_suggestions(&mut self) {
        self.commands.dismiss_suggestions();
    }
}

fn clamp_cursor(input: &str, cursor: usize) -> usize {
    let mut cursor = cursor.min(input.len());
    while cursor > 0 && !input.is_char_boundary(cursor) {
        cursor -= 1;
    }
    cursor
}

fn previous_cursor_position(input: &str, cursor: usize) -> usize {
    let cursor = clamp_cursor(input, cursor);
    input[..cursor]
        .char_indices()
        .next_back()
        .map_or(0, |(index, _)| index)
}

fn next_cursor_position(input: &str, cursor: usize) -> usize {
    let cursor = clamp_cursor(input, cursor);
    input[cursor..]
        .chars()
        .next()
        .map_or(cursor, |character| cursor + character.len_utf8())
}
