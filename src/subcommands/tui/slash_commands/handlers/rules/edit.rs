use std::path::PathBuf;

use crate::prompt_rules::RulesTarget;

use super::{CommandOutput, RulesContext};

/// Update command state after an external rules edit finishes.
pub fn complete_rules_edit<C>(
    context: &mut C,
    target: RulesTarget,
    path: PathBuf,
    editor_result: Result<(), String>,
) where
    C: CommandOutput + RulesContext + ?Sized,
{
    match editor_result {
        Ok(()) => complete_successful_edit(context, target, path),
        Err(error) => complete_failed_edit(context, target, path, error),
    }
}

fn complete_successful_edit<C>(context: &mut C, target: RulesTarget, path: PathBuf)
where
    C: CommandOutput + RulesContext + ?Sized,
{
    let rules_were_enabled = context.rules_enabled();
    context.reload_rules(rules_were_enabled);
    context.append_local_message(
        "/rules",
        format!(
            "Reloaded {} from {}.\nRules: {}",
            target.label(),
            path.display(),
            context.rules_status_line()
        ),
    );
    context.set_status(format!("Reloaded {}.", target.label()));
}

fn complete_failed_edit<C>(context: &mut C, target: RulesTarget, path: PathBuf, error: String)
where
    C: CommandOutput + ?Sized,
{
    context.append_local_message(
        "/rules",
        format!(
            "Could not edit {} at {}.\n{}",
            target.label(),
            path.display(),
            error
        ),
    );
    context.set_status(format!("Failed to edit {}.", target.label()));
}
