use super::super::handlers::CommandContext;
use super::super::parser::ParsedCommand;

/// Stable identifier for command behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandId {
    Backend,
    Bookmark,
    Clear,
    Context,
    Export,
    Help,
    History,
    Memory,
    Model,
    Quit,
    Resize,
    Rules,
    Summary,
    Theme,
    Tokens,
    Voice,
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
    #[allow(dead_code)]
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
    pub(super) fn registered_command(&self) -> RegisteredCommand {
        RegisteredCommand {
            id: self.id,
            executor: self.executor,
        }
    }

    pub(super) fn help(&self) -> CommandHelp {
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
