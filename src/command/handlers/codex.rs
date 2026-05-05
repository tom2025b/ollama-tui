use crate::command::parser::ParsedCommand;

use super::session::CommandContext;

pub fn handle_codex_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    let enabled = !context.codex_mode_enabled();
    context.set_codex_mode(enabled);

    if enabled {
        match context.best_coding_model() {
            Some(model) => {
                context.append_local_message(
                    command.raw(),
                    format!(
                        "Codex mode enabled. System prompt is pinned and routing prefers {}.",
                        model.display_label()
                    ),
                );
                context.set_status(format!("Codex mode enabled. Pinned to {}.", model.display_label()));
            }
            None => {
                context.append_local_message(
                    command.raw(),
                    "Codex mode enabled, but no coding-capable backend is currently available."
                        .to_string(),
                );
                context.set_status("Codex mode enabled.".to_string());
            }
        }
    } else {
        context.append_local_message(
            command.raw(),
            "Codex mode disabled. Routing returned to normal.".to_string(),
        );
        context.set_status("Codex mode disabled.".to_string());
    }
}
