mod definitions;
mod types;

use super::parser::{ParsedCommand, suggestion_prefix};
use definitions::COMMAND_GROUPS;
use types::CommandDefinition;
pub use types::{CommandHelp, CommandId, CommandSuggestion, RegisteredCommand};

/// Registry for all slash-command names, aliases, help text, and handlers.
#[derive(Clone, Copy, Debug)]
pub struct CommandRegistry {
    groups: &'static [&'static [CommandDefinition]],
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self {
            groups: COMMAND_GROUPS,
        }
    }
}

impl CommandRegistry {
    fn definitions(&self) -> impl Iterator<Item = &'static CommandDefinition> + '_ {
        self.groups.iter().flat_map(|group| group.iter())
    }

    /// Resolve a parsed command to its registered executable definition.
    pub fn resolve(&self, parsed: &ParsedCommand) -> Option<RegisteredCommand> {
        self.definitions().find_map(|definition| {
            definition
                .names
                .iter()
                .find(|name| name.name == parsed.name())
                .map(|_| definition.registered_command())
        })
    }

    /// Slash-command suggestions that match the current input prefix.
    pub fn suggestions(&self, input: &str) -> Vec<CommandSuggestion> {
        let Some(prefix) = suggestion_prefix(input) else {
            return Vec::new();
        };

        self.definitions()
            .filter(|definition| {
                definition
                    .names
                    .iter()
                    .any(|name| name.visible && name.name.starts_with(prefix))
            })
            .map(|definition| definition.suggestion())
            .collect()
    }

    /// Human-readable list used when an unknown command is entered.
    pub fn available_commands(&self) -> String {
        self.help_entries()
            .into_iter()
            .map(|entry| entry.name)
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Command help rows in display order.
    pub fn help_entries(&self) -> Vec<CommandHelp> {
        self.definitions()
            .map(|definition| definition.help())
            .collect()
    }
}

#[cfg(test)]
mod tests;
