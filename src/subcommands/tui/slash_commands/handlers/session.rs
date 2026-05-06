use std::path::PathBuf;

use crate::prompt_rules::RulesTarget;
use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

/// External work requested by a command handler.
#[derive(Clone, Debug)]
pub enum ExternalAction {
    /// Suspend the TUI and hand control to the `claude` CLI.
    ClaudeCode {
        /// Directory to launch Claude Code in.
        working_dir: PathBuf,
    },

    /// Suspend the TUI and hand control to the `codex` CLI.
    CodexCli {
        /// Directory to launch Codex in.
        working_dir: PathBuf,
    },

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

pub fn complete_claude_session(app: &mut App, result: Result<(), String>) {
    match result {
        Ok(()) => app.ui.status = "Returned from Claude Code.".to_string(),
        Err(error) => {
            app.append_local_message("Claude Code", format!("Claude Code failed.\n{error}"));
            app.ui.status = "Claude Code failed.".to_string();
        }
    }
}

pub fn complete_codex_session(app: &mut App, result: Result<(), String>) {
    match result {
        Ok(()) => app.ui.status = "Returned from Codex.".to_string(),
        Err(error) => {
            app.append_local_message("Codex", format!("Codex failed.\n{error}"));
            app.ui.status = "Codex failed.".to_string();
        }
    }
}

pub fn unknown_command(app: &mut App, command: &ParsedCommand, available_commands: &str) {
    app.append_local_message(
        command.raw(),
        format!("Unknown command. Available commands: {available_commands}."),
    );
    app.ui.status = "Unknown command.".to_string();
}
