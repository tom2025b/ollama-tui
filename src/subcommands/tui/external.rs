use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::subcommands::tui::{
    app::App,
    slash_commands::{ExternalAction, handlers},
    terminal::{AppTerminal, resume_terminal, stop_terminal},
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
            let result = run_outside_tui(terminal, || {
                launch_with_prompt("claude", &prompt, &working_dir)
            })?;
            handlers::session::complete_claude_session(app, result);
        }
        ExternalAction::CodexCli {
            working_dir,
            prompt,
        } => {
            let result = run_outside_tui(terminal, || {
                launch_with_prompt("codex", &prompt, &working_dir)
            })?;
            handlers::session::complete_codex_session(app, result);
        }
        ExternalAction::EditRules { target, path } => {
            let editor = app.editor_command().to_os_string();
            let editor_name = editor.to_string_lossy().into_owned();

            let editor_result = run_outside_tui(terminal, || {
                let status = Command::new(&editor).arg(&path).status();
                describe_status(&editor_name, status)
            })?;

            handlers::complete_rules_edit(app, target, path, editor_result);
        }
    }

    Ok(())
}

/// Drop the TUI, run a foreground command, and reattach.
fn run_outside_tui<F, T>(terminal: &mut AppTerminal, run: F) -> Result<T>
where
    F: FnOnce() -> T,
{
    stop_terminal(terminal)?;
    let result = run();
    resume_terminal(terminal)?;
    Ok(result)
}

/// Launch a terminal app with the user's prompt as its first argument.
///
/// Both `claude` and `codex` accept an initial prompt as a positional argument
/// so the CLI opens with context. If the installed CLI ignores it, the app
/// simply opens normally.
fn launch_with_prompt(binary: &str, prompt: &str, working_dir: &Path) -> Result<(), String> {
    let status = Command::new(binary)
        .arg(prompt)
        .current_dir(working_dir)
        .status();
    describe_status(binary, status)
}

fn describe_status(
    binary: &str,
    status: std::io::Result<std::process::ExitStatus>,
) -> Result<(), String> {
    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("{binary} exited with status {status}")),
        Err(error) => Err(format!("failed to launch {binary}: {error}")),
    }
}
