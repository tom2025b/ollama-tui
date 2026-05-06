use std::path::PathBuf;

use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

use super::session::{CommandContext, ExternalAction};

// Handle the /claude slash command.
//
// Suspends the TUI and launches the `claude` CLI (Claude Code) in the project
// root, so it starts with full awareness of the codebase. Resumes the TUI when
// the user exits Claude Code.
pub fn handle_claude_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    if context.waiting_for_model() {
        context.append_local_message(
            command.raw(),
            "Wait for the current response to finish before opening Claude Code.".to_string(),
        );
        context.set_status("Cannot open Claude Code while a model is answering.".to_string());
        return;
    }

    // Use the project root so Claude Code picks up CLAUDE.md and the file tree
    // automatically. Fall back to the process working directory if unknown.
    let working_dir = context.project_root().unwrap_or_else(|| PathBuf::from("."));

    context.queue_external_action(ExternalAction::ClaudeCode { working_dir });
    context.set_status("Opening Claude Code\u{2026}".to_string());
}

// Called by external.rs after the claude process exits.
pub fn complete_claude_session(context: &mut dyn CommandContext, result: Result<(), String>) {
    match result {
        Ok(()) => {
            context.append_local_message("/claude", "Claude Code session ended.".to_string());
            context.set_status("Returned to TUI.".to_string());
        }
        Err(error) => {
            context.append_local_message("/claude", format!("Claude Code failed.\n{error}"));
            context.set_status("Claude Code failed.".to_string());
        }
    }
}
