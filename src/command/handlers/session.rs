mod context;

use crate::command::parser::ParsedCommand;
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
