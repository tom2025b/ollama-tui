use std::path::PathBuf;

use crate::prompt_rules::{RulesState, RulesTarget};
use crate::subcommands::tui::app::App;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

use super::session::ExternalAction;

pub fn rules_command(app: &mut App, command: &ParsedCommand) {
    let mut args = command.args().iter().map(String::as_str);
    let subcommand = args.next().map(|arg| arg.to_ascii_lowercase());

    match subcommand.as_deref() {
        None => {
            let target = default_rules_target(app);
            queue_rules_edit(app, command.raw(), target);
        }
        Some("global") => queue_rules_edit(app, command.raw(), RulesTarget::Global),
        Some("project") => queue_rules_edit(app, command.raw(), RulesTarget::Project),
        Some("show") | Some("status") => {
            app.append_local_message(command.raw(), app.rules.report());
            app.ui.status = "Displayed rules status.".to_string();
        }
        Some("off") | Some("disable") => {
            app.rules.set_enabled(false);
            app.append_local_message(
                command.raw(),
                "All rules are off for new prompts.".to_string(),
            );
            app.ui.status = "Rules turned off.".to_string();
        }
        Some("on") | Some("enable") => {
            reload_rules(app, true);
            app.append_local_message(command.raw(), app.rules.report());
            app.ui.status = "Rules turned on and reloaded.".to_string();
        }
        Some("toggle") => {
            toggle_rules(app, command.raw());
        }
        _ => {
            app.append_local_message(
                command.raw(),
                "Usage: /rules [global|project|show|off|on|toggle]".to_string(),
            );
            app.ui.status = "Unknown /rules command.".to_string();
        }
    }
}

fn default_rules_target(app: &App) -> RulesTarget {
    if app.rules.project_root().is_some() {
        RulesTarget::Project
    } else {
        RulesTarget::Global
    }
}

fn toggle_rules(app: &mut App, input: &str) {
    let enabled = !app.rules.enabled();
    if enabled {
        reload_rules(app, true);
    } else {
        app.rules.set_enabled(false);
    }

    app.append_local_message(
        input,
        format!(
            "Rules are now {}.\nRules: {}",
            if enabled { "on" } else { "off" },
            app.rules.status_line()
        ),
    );
    app.ui.status = if enabled {
        "Rules turned on.".to_string()
    } else {
        "Rules turned off.".to_string()
    };
}

fn queue_rules_edit(app: &mut App, input: &str, target: RulesTarget) {
    if app.session.waiting_for_model {
        app.append_local_message(
            input,
            "Wait for the current model response to finish before editing rules.".to_string(),
        );
        app.ui.status = "Cannot edit rules while a model is answering.".to_string();
        return;
    }

    match app.rules.prepare_edit(target) {
        Ok(path) => {
            app.commands
                .queue_external_action(ExternalAction::EditRules {
                    target,
                    path: path.clone(),
                });
            app.ui.status = format!(
                "Opening editor for {} at {}.",
                target.label(),
                path.display()
            );
        }
        Err(error) => {
            app.append_local_message(
                input,
                format!("Could not prepare {}: {error}", target.label()),
            );
            app.ui.status = format!("Failed to prepare {}.", target.label());
        }
    }
}

/// Update command state after an external rules edit finishes.
pub fn complete_rules_edit(
    app: &mut App,
    target: RulesTarget,
    path: PathBuf,
    editor_result: Result<(), String>,
) {
    match editor_result {
        Ok(()) => complete_successful_edit(app, target, path),
        Err(error) => complete_failed_edit(app, target, path, error),
    }
}

fn complete_successful_edit(app: &mut App, target: RulesTarget, path: PathBuf) {
    let rules_were_enabled = app.rules.enabled();
    reload_rules(app, rules_were_enabled);
    app.append_local_message(
        "/rules",
        format!(
            "Reloaded {} from {}.\nRules: {}",
            target.label(),
            path.display(),
            app.rules.status_line()
        ),
    );
    app.ui.status = format!("Reloaded {}.", target.label());
}

fn complete_failed_edit(app: &mut App, target: RulesTarget, path: PathBuf, error: String) {
    app.append_local_message(
        "/rules",
        format!(
            "Could not edit {} at {}.\n{}",
            target.label(),
            path.display(),
            error
        ),
    );
    app.ui.status = format!("Failed to edit {}.", target.label());
}

fn reload_rules(app: &mut App, enabled: bool) {
    app.rules = RulesState::load(app.runtime.paths()).with_enabled(enabled);
}
