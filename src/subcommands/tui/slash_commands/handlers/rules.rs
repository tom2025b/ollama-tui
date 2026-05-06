mod edit;

use crate::prompt_rules::RulesTarget;
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

use super::session::{CommandContext, ExternalAction};
pub use edit::complete_rules_edit;

pub fn handle_rules_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    let mut args = command.args().iter().map(String::as_str);
    let subcommand = args.next().map(|arg| arg.to_ascii_lowercase());

    match subcommand.as_deref() {
        None => {
            let target = context.default_rules_target();
            queue_rules_edit(context, command.raw(), target);
        }
        Some("global") => queue_rules_edit(context, command.raw(), RulesTarget::Global),
        Some("project") => queue_rules_edit(context, command.raw(), RulesTarget::Project),
        Some("show") | Some("status") => {
            context.append_local_message(command.raw(), context.rules_report());
            context.set_status("Displayed rules status.".to_string());
        }
        Some("off") | Some("disable") => {
            context.set_rules_enabled(false);
            context.append_local_message(
                command.raw(),
                "All rules are off for new prompts.".to_string(),
            );
            context.set_status("Rules turned off.".to_string());
        }
        Some("on") | Some("enable") => {
            context.reload_rules(true);
            context.append_local_message(command.raw(), context.rules_report());
            context.set_status("Rules turned on and reloaded.".to_string());
        }
        Some("toggle") => {
            toggle_rules(context, command.raw());
        }
        _ => {
            context.append_local_message(
                command.raw(),
                "Usage: /rules [global|project|show|off|on|toggle]".to_string(),
            );
            context.set_status("Unknown /rules command.".to_string());
        }
    }
}

fn toggle_rules(context: &mut dyn CommandContext, input: &str) {
    let enabled = !context.rules_enabled();
    if enabled {
        context.reload_rules(true);
    } else {
        context.set_rules_enabled(false);
    }

    context.append_local_message(
        input,
        format!(
            "Rules are now {}.\nRules: {}",
            if enabled { "on" } else { "off" },
            context.rules_status_line()
        ),
    );
    context.set_status(if enabled {
        "Rules turned on.".to_string()
    } else {
        "Rules turned off.".to_string()
    });
}

fn queue_rules_edit(context: &mut dyn CommandContext, input: &str, target: RulesTarget) {
    if context.waiting_for_model() {
        context.append_local_message(
            input,
            "Wait for the current model response to finish before editing rules.".to_string(),
        );
        context.set_status("Cannot edit rules while a model is answering.".to_string());
        return;
    }

    match context.prepare_rules_edit(target) {
        Ok(path) => {
            context.queue_external_action(ExternalAction::EditRules {
                target,
                path: path.clone(),
            });
            context.set_status(format!(
                "Opening nano for {} at {}.",
                target.label(),
                path.display()
            ));
        }
        Err(error) => {
            context.append_local_message(
                input,
                format!("Could not prepare {}: {error}", target.label()),
            );
            context.set_status(format!("Failed to prepare {}.", target.label()));
        }
    }
}
