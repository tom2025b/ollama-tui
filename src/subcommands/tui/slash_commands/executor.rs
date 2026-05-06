use super::dispatcher::{CommandInvocation, DispatchResult};
use super::handlers::{self, CommandContext};
use super::parser::ParsedCommand;

/// Execute a dispatched command.
pub fn execute(context: &mut dyn CommandContext, invocation: CommandInvocation) {
    invocation.registered.execute(context, &invocation.parsed);
}

/// Execute the result of dispatching a parsed command.
pub fn execute_dispatch(
    context: &mut dyn CommandContext,
    dispatch: DispatchResult,
    available_commands: &str,
) {
    match dispatch {
        DispatchResult::Execute(invocation) => execute(context, invocation),
        DispatchResult::Unknown(parsed) => execute_unknown(context, &parsed, available_commands),
    }
}

/// Execute the fallback handler for syntactically valid but unregistered
/// commands.
pub fn execute_unknown(
    context: &mut dyn CommandContext,
    parsed: &ParsedCommand,
    available_commands: &str,
) {
    handlers::session::unknown_command(context, parsed, available_commands);
}
