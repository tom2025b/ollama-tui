mod context;

use crate::subcommands::tui::slash_commands::parser::ParsedCommand;
pub use context::{CommandContext, ExternalAction, HistoryEntry, Setting, SettingEdit};

pub fn clear_conversation_command(context: &mut dyn CommandContext, _command: &ParsedCommand) {
    if context.waiting_for_model() {
        context.set_status("Cannot clear while a model is answering.".to_string());
        return;
    }

    context.clear_conversation();
    context.set_status("Conversation cleared.".to_string());
}

pub fn open_models_command(context: &mut dyn CommandContext, _command: &ParsedCommand) {
    context.open_models_picker();
}

pub fn open_help_command(context: &mut dyn CommandContext, _command: &ParsedCommand) {
    context.open_help_overlay();
}

pub fn cost_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    if context.waiting_for_model() {
        context.append_local_message(
            command.raw(),
            "Wait for the current model response to finish before opening the cost tracker."
                .to_string(),
        );
        context.set_status("Cannot open cost tracker while a model is answering.".to_string());
        return;
    }

    context.queue_external_action(ExternalAction::CostReport);
    context.set_status("Opening cost tracker.".to_string());
}

pub fn complete_cost_report(context: &mut dyn CommandContext, result: Result<(), String>) {
    match result {
        Ok(()) => {
            context.append_local_message(
                "/cost",
                "Cost tracker finished. Returned to the TUI.".to_string(),
            );
            context.set_status("Cost tracker finished.".to_string());
        }
        Err(error) => {
            context.append_local_message("/cost", format!("Cost tracker failed.\n{error}"));
            context.set_status("Cost tracker failed.".to_string());
        }
    }
}

pub fn quit_command(context: &mut dyn CommandContext, _command: &ParsedCommand) {
    context.quit();
}

pub fn unknown_command(
    context: &mut dyn CommandContext,
    command: &ParsedCommand,
    available_commands: &str,
) {
    context.append_local_message(
        command.raw(),
        format!("Unknown command. Available commands: {available_commands}."),
    );
    context.set_status("Unknown command.".to_string());
}
