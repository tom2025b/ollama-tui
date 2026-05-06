use super::super::parser::ParsedCommand;
use crate::subcommands::tui::app::App;

/// One visible autocomplete row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandSuggestion {
    pub name: &'static str,
    pub hint: &'static str,
    pub detail: &'static str,
}

/// One command row shown by the help overlay and unknown-command fallback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandHelp {
    pub name: &'static str,
    pub hint: &'static str,
    pub detail: &'static str,
}

pub(super) type CommandExecutor = fn(&mut App, &ParsedCommand);

#[derive(Clone, Copy, Debug)]
pub(super) struct CommandDefinition {
    pub(super) display_name: &'static str,
    pub(super) hint: &'static str,
    pub(super) detail: &'static str,
    pub(super) names: &'static [CommandName],
    pub(super) execute: CommandExecutor,
}

impl CommandDefinition {
    pub(super) fn help(&self) -> CommandHelp {
        CommandHelp {
            name: self.display_name,
            hint: self.hint,
            detail: self.detail,
        }
    }

    pub(super) fn suggestion(&self) -> CommandSuggestion {
        CommandSuggestion {
            name: self.display_name,
            hint: self.hint,
            detail: self.detail,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CommandName {
    pub(super) name: &'static str,
    pub(super) visible: bool,
}
