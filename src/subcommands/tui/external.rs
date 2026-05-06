use std::process::Command;

use anyhow::Result;

use crate::subcommands::tui::{
    app::App,
    slash_commands::{ExternalAction, handlers},
    terminal::{AppTerminal, resume_terminal, suspend_terminal},
};

/// Run an external command that cannot happen while the terminal is in TUI mode.
pub fn run_external_action(
    terminal: &mut AppTerminal,
    app: &mut App,
    action: ExternalAction,
) -> Result<()> {
    match action {
        ExternalAction::ClaudeCode {
            working_dir,
            prompt,
        } => {
            suspend_terminal(terminal)?;
            // Pass the prompt as a positional argument so Claude Code opens
            // with context. Interactive behaviour depends on the installed CLI
            // version; if unsupported, Claude Code simply opens normally.
            let result = Command::new("claude")
                .arg(&prompt)
                .current_dir(&working_dir)
                .status();
            resume_terminal(terminal)?;

            let result = match result {
                Ok(status) if status.success() => Ok(()),
                Ok(status) => Err(format!("claude exited with status {status}")),
                Err(error) => Err(format!("failed to launch claude: {error}")),
            };

            handlers::session::complete_claude_session(app, result);
        }
        ExternalAction::CodexCli {
            working_dir,
            prompt,
        } => {
            suspend_terminal(terminal)?;
            // Codex CLI accepts an initial prompt as a positional argument.
            let result = Command::new("codex")
                .arg(&prompt)
                .current_dir(&working_dir)
                .status();
            resume_terminal(terminal)?;

            let result = match result {
                Ok(status) if status.success() => Ok(()),
                Ok(status) => Err(format!("codex exited with status {status}")),
                Err(error) => Err(format!("failed to launch codex: {error}")),
            };

            handlers::session::complete_codex_session(app, result);
        }
        ExternalAction::EditRules { target, path } => {
            let editor = app.editor_command().to_os_string();
            let editor_name = editor.to_string_lossy().into_owned();

            suspend_terminal(terminal)?;
            let editor_result = Command::new(&editor).arg(&path).status();
            resume_terminal(terminal)?;

            let editor_result = match editor_result {
                Ok(status) if status.success() => Ok(()),
                Ok(status) => Err(format!("{editor_name} exited with status {status}")),
                Err(error) => Err(format!("failed to launch {editor_name}: {error}")),
            };

            handlers::complete_rules_edit(app, target, path, editor_result);
        }
    }

    Ok(())
}
