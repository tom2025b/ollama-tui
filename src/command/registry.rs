mod definitions;

use super::handlers::CommandContext;
use super::parser::{ParsedCommand, suggestion_prefix};
use definitions::COMMANDS;

/// Stable identifier for command behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandId {
    Backend,
    Clear,
    Help,
    History,
    Model,
    Quit,
    Rules,
}

/// One visible autocomplete row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandSuggestion {
    pub name: &'static str,
    pub hint: &'static str,
}

/// One command row shown by the help overlay and unknown-command fallback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandHelp {
    pub name: &'static str,
    pub detail: &'static str,
}

/// Resolved command metadata returned by the registry after parsing.
#[derive(Clone, Copy, Debug)]
pub struct RegisteredCommand {
    pub id: CommandId,
    executor: CommandExecutor,
}

impl RegisteredCommand {
    pub fn execute(self, context: &mut dyn CommandContext, command: &ParsedCommand) {
        (self.executor)(context, command);
    }
}

pub(super) type CommandExecutor = fn(&mut dyn CommandContext, &ParsedCommand);

#[derive(Clone, Copy, Debug)]
pub(super) struct CommandDefinition {
    pub(super) id: CommandId,
    pub(super) display_name: &'static str,
    pub(super) hint: &'static str,
    pub(super) detail: &'static str,
    pub(super) names: &'static [CommandName],
    pub(super) executor: CommandExecutor,
}

impl CommandDefinition {
    fn registered_command(self) -> RegisteredCommand {
        RegisteredCommand {
            id: self.id,
            executor: self.executor,
        }
    }

    fn help(self) -> CommandHelp {
        CommandHelp {
            name: self.display_name,
            detail: self.detail,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CommandName {
    pub(super) name: &'static str,
    pub(super) visible: bool,
}

/// Registry for all slash-command names, aliases, help text, and handlers.
#[derive(Clone, Copy, Debug)]
pub struct CommandRegistry {
    definitions: &'static [CommandDefinition],
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self {
            definitions: COMMANDS,
        }
    }
}

impl CommandRegistry {
    /// Resolve a parsed command to its registered executable definition.
    pub fn resolve(&self, parsed: &ParsedCommand) -> Option<RegisteredCommand> {
        self.definitions.iter().find_map(|definition| {
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

        self.definitions
            .iter()
            .flat_map(|definition| {
                definition
                    .names
                    .iter()
                    .filter(|name| name.visible && name.name.starts_with(prefix))
                    .map(|name| CommandSuggestion {
                        name: name.name,
                        hint: definition.hint,
                    })
            })
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
        self.definitions
            .iter()
            .map(|definition| definition.help())
            .collect()
    }
}

#[cfg(test)]
mod tests;
