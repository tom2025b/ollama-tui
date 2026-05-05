use crate::command::parser::ParsedCommand;

use super::session::CommandContext;

pub fn handle_clear_command(context: &mut dyn CommandContext, _command: &ParsedCommand) {
    if context.waiting_for_model() {
        context.set_status("Cannot clear while a model is answering.".to_string());
        return;
    }

    context.clear_conversation();
    context.set_codex_mode(false);
    context.set_status("Conversation cleared.".to_string());
}
