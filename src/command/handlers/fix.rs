use crate::command::parser::ParsedCommand;

use super::session::CommandContext;

pub fn handle_fix_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    context.append_local_message(command.raw(), "Fix command received".to_string());
    context.set_status("Fix command received".to_string());
}
