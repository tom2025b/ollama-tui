use std::path::PathBuf;

use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

use super::session::{CommandContext, ExternalAction};

// Handle the /codex slash command.
//
// Suspends the TUI and launches the `codex` CLI in the project root, so it
// starts with full awareness of the codebase. Resumes the TUI when the user
// exits Codex.
pub fn handle_codex_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    if context.waiting_for_model() {
        context.append_local_message(
            command.raw(),
            "Wait for the current response to finish before opening Codex.".to_string(),
        );
        context.set_status("Cannot open Codex while a model is answering.".to_string());
        return;
    }

    let working_dir = context.project_root().unwrap_or_else(|| PathBuf::from("."));

    context.queue_external_action(ExternalAction::CodexCli { working_dir });
    context.set_status("Opening Codex\u{2026}".to_string());
}

// Called by external.rs after the codex process exits.
pub fn complete_codex_session(context: &mut dyn CommandContext, result: Result<(), String>) {
    match result {
        Ok(()) => {
            context.append_local_message("/codex", "Codex session ended.".to_string());
            context.set_status("Returned to TUI.".to_string());
        }
        Err(error) => {
            context.append_local_message("/codex", format!("Codex failed.\n{error}"));
            context.set_status("Codex failed.".to_string());
        }
    }
}
