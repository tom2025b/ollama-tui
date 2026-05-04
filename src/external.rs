use std::process::Command;

use anyhow::Result;

use crate::{
    app::App,
    command::{ExternalAction, handlers},
    terminal::{AppTerminal, resume_terminal, suspend_terminal},
};

const COST_TRACKER_DIR: &str = "/home/tom/projects/claude-cost-tracker";
const COST_TRACKER_SCRIPT: &str = "quick-cost-report.py";

/// Run an external command that cannot happen while the terminal is in TUI mode.
pub fn run_external_action(
    terminal: &mut AppTerminal,
    app: &mut App,
    action: ExternalAction,
) -> Result<()> {
    match action {
        ExternalAction::CostReport => {
            suspend_terminal(terminal)?;
            let cost_result = Command::new("python3")
                .arg(COST_TRACKER_SCRIPT)
                .current_dir(COST_TRACKER_DIR)
                .status();
            resume_terminal(terminal)?;

            let cost_result = match cost_result {
                Ok(status) if status.success() => Ok(()),
                Ok(status) => Err(format!("cost tracker exited with status {status}")),
                Err(error) => Err(format!("failed to launch cost tracker: {error}")),
            };

            handlers::session::complete_cost_report(app, cost_result);
        }
        ExternalAction::EditRules { target, path } => {
            suspend_terminal(terminal)?;
            let editor_result = Command::new("nano").arg(&path).status();
            resume_terminal(terminal)?;

            let editor_result = match editor_result {
                Ok(status) if status.success() => Ok(()),
                Ok(status) => Err(format!("nano exited with status {status}")),
                Err(error) => Err(format!("failed to launch nano: {error}")),
            };

            handlers::complete_rules_edit(app, target, path, editor_result);
        }
    }

    Ok(())
}
