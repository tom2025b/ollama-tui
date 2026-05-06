use std::path::PathBuf;

use crate::prompt_rules::RulesTarget;
use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

/// External work requested by a command handler.
#[derive(Clone, Debug)]
pub enum ExternalAction {
    /// Open a rules file in the configured editor, then reload rules when it exits.
    EditRules {
        /// Which rules file is being edited.
        target: RulesTarget,

        /// File path to open.
        path: PathBuf,
    },
}

pub fn open_models_command(app: &mut App, _command: &ParsedCommand) {
    app.open_models_picker();
}

pub fn open_help_command(app: &mut App, _command: &ParsedCommand) {
    app.ui.show_help = true;
    app.ui.status = "Help is open. Press q, Esc, ?, or Ctrl-C to close it.".to_string();
}

pub fn quit_command(app: &mut App, _command: &ParsedCommand) {
    app.quit();
}

pub fn unknown_command(app: &mut App, command: &ParsedCommand, available_commands: &str) {
    app.append_local_message(
        command.raw(),
        format!("Unknown command. Available commands: {available_commands}."),
    );
    app.ui.status = "Unknown command.".to_string();
}
