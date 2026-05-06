use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

use super::session::{CommandContext, CommandOutput, ConversationControl, ModelActivity};

pub fn handle_clear_command<C>(context: &mut C, _command: &ParsedCommand)
where
    C: CommandOutput + ConversationControl + ModelActivity + ?Sized,
{
    if context.waiting_for_model() {
        context.set_status("Cannot clear while a model is answering.".to_string());
        return;
    }

    context.clear_conversation();
    context.set_status("Conversation cleared.".to_string());
}

pub fn execute_clear_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    handle_clear_command(context, command);
}
