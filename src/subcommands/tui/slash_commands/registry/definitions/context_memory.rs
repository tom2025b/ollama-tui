use crate::subcommands::tui::slash_commands::handlers;

use super::super::CommandId;
use super::super::types::{CommandDefinition, CommandName};

pub(super) const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        id: CommandId::Context,
        display_name: "/context",
        hint: "Show context window",
        detail: "Show remembered turns that feed the next prompt.",
        names: &[CommandName {
            name: "/context",
            visible: true,
        }],
        executor: handlers::context_memory::handle_context_command,
    },
    CommandDefinition {
        id: CommandId::Tokens,
        display_name: "/tokens",
        hint: "Estimate context tokens",
        detail: "Estimate tokens in model history and next context.",
        names: &[CommandName {
            name: "/tokens",
            visible: true,
        }],
        executor: handlers::context_memory::handle_tokens_command,
    },
    CommandDefinition {
        id: CommandId::Bookmark,
        display_name: "/bookmark",
        hint: "Remember latest turn",
        detail: "Add or remove the latest completed turn from future context.",
        names: &[CommandName {
            name: "/bookmark",
            visible: true,
        }],
        executor: handlers::context_memory::handle_bookmark_command,
    },
    CommandDefinition {
        id: CommandId::Memory,
        display_name: "/memory",
        hint: "Show or clear memory",
        detail: "Show remembered turns or clear them from future context.",
        names: &[CommandName {
            name: "/memory",
            visible: true,
        }],
        executor: handlers::context_memory::handle_memory_command,
    },
];
