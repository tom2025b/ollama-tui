use super::parser::ParsedCommand;
use super::registry::{CommandRegistry, RegisteredCommand};

/// A parsed slash command that has been resolved against the registry.
#[derive(Clone, Debug)]
pub struct CommandInvocation {
    pub parsed: ParsedCommand,
    pub registered: RegisteredCommand,
}

/// Outcome of the Parse -> Dispatch stages.
#[derive(Clone, Debug)]
pub enum DispatchResult {
    Execute(CommandInvocation),
    Unknown(ParsedCommand),
}

/// Routes parsed slash commands to registered command metadata.
#[derive(Clone, Copy, Debug, Default)]
pub struct CommandDispatcher {
    registry: CommandRegistry,
}

impl CommandDispatcher {
    pub fn new(registry: CommandRegistry) -> Self {
        Self { registry }
    }

    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    /// Human-readable list of registered commands for fallback errors.
    pub fn available_commands(&self) -> String {
        self.registry.available_commands()
    }

    /// Resolve a parsed command against the command registry.
    pub fn dispatch(&self, parsed: ParsedCommand) -> DispatchResult {
        match self.registry.resolve(&parsed) {
            Some(registered) => DispatchResult::Execute(CommandInvocation { parsed, registered }),
            None => DispatchResult::Unknown(parsed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::parser::{ParseResult, parse_slash_command};
    use crate::command::registry::CommandId;

    fn parse(input: &str) -> ParsedCommand {
        match parse_slash_command(input) {
            ParseResult::Command(command) => command,
            ParseResult::NotCommand => panic!("expected command"),
        }
    }

    #[test]
    fn dispatcher_resolves_registered_command() {
        let dispatcher = CommandDispatcher::default();
        let result = dispatcher.dispatch(parse("/model"));

        match result {
            DispatchResult::Execute(invocation) => {
                assert_eq!(invocation.parsed.name(), "/model");
                assert_eq!(invocation.registered.id, CommandId::Model);
            }
            other => panic!("unexpected dispatch result: {other:?}"),
        }
    }

    #[test]
    fn dispatcher_preserves_unknown_command_for_error_handling() {
        let dispatcher = CommandDispatcher::default();
        let result = dispatcher.dispatch(parse("/missing arg"));

        match result {
            DispatchResult::Unknown(parsed) => {
                assert_eq!(parsed.raw(), "/missing arg");
                assert_eq!(parsed.name(), "/missing");
            }
            other => panic!("unexpected dispatch result: {other:?}"),
        }
    }
}
