use std::process::Command;

use crate::Result;
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
        ExternalAction::EditRules { target, path } => {
            let editor_result = run_editor(terminal, app, &path)?;
            handlers::complete_rules_edit(app, target, path, editor_result);
        }
        ExternalAction::EditConfig { path } => {
            let editor_result = run_editor(terminal, app, &path)?;
            handlers::complete_config_edit(app, path, editor_result);
        }
    }

    Ok(())
}

/// Suspend the TUI, launch `$EDITOR <path>`, then resume the TUI. Returns the
/// editor's exit result formatted as a friendly string for handlers to display.
fn run_editor(
    terminal: &mut AppTerminal,
    app: &App,
    path: &std::path::Path,
) -> Result<std::result::Result<(), String>> {
    let editor = app.editor_command().to_os_string();
    let editor_name = editor.to_string_lossy().into_owned();

    suspend_terminal(terminal)?;
    let status = Command::new(&editor).arg(path).status();
    resume_terminal(terminal)?;

    Ok(match status {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("{editor_name} exited with status {status}")),
        Err(error) => Err(format!("failed to launch {editor_name}: {error}")),
    })
}
