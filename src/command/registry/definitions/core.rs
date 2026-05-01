use crate::command::handlers;

use super::super::CommandId;
use super::super::types::{CommandDefinition, CommandName};

pub(super) const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        id: CommandId::Clear,
        display_name: "/clear",
        hint: "Clear visible conversation",
        detail: "Clear the visible conversation.",
        names: &[CommandName {
            name: "/clear",
            visible: true,
        }],
        executor: handlers::session::clear_conversation_command,
    },
    CommandDefinition {
        id: CommandId::Model,
        display_name: "/models",
        hint: "Pick a model to pin",
        detail: "Open the model picker; choose Auto to resume routing.",
        names: &[
            CommandName {
                name: "/model",
                visible: true,
            },
            CommandName {
                name: "/models",
                visible: true,
            },
        ],
        executor: handlers::session::open_models_command,
    },
    CommandDefinition {
        id: CommandId::Backend,
        display_name: "/backends",
        hint: "List backend readiness",
        detail: "Show configured and unavailable backends.",
        names: &[
            CommandName {
                name: "/backend",
                visible: true,
            },
            CommandName {
                name: "/backends",
                visible: true,
            },
        ],
        executor: handlers::backends::handle_backends_command,
    },
    CommandDefinition {
        id: CommandId::Rules,
        display_name: "/rules",
        hint: "Edit or toggle rules",
        detail: "Edit, show, enable, disable, or toggle rule loading.",
        names: &[CommandName {
            name: "/rules",
            visible: true,
        }],
        executor: handlers::rules::handle_rules_command,
    },
    CommandDefinition {
        id: CommandId::Help,
        display_name: "/help",
        hint: "Open help overlay",
        detail: "Open the help overlay.",
        names: &[CommandName {
            name: "/help",
            visible: true,
        }],
        executor: handlers::session::open_help_command,
    },
    CommandDefinition {
        id: CommandId::History,
        display_name: "/history",
        hint: "Show, save, or email history",
        detail: "Show history, save it to a file, or send it by email.",
        names: &[CommandName {
            name: "/history",
            visible: true,
        }],
        executor: handlers::history::handle_history_command,
    },
    CommandDefinition {
        id: CommandId::Quit,
        display_name: "/quit",
        hint: "Quit the app",
        detail: "Quit the app.",
        names: &[
            CommandName {
                name: "/exit",
                visible: true,
            },
            CommandName {
                name: "/q",
                visible: false,
            },
            CommandName {
                name: "/quit",
                visible: true,
            },
        ],
        executor: handlers::session::quit_command,
    },
];
