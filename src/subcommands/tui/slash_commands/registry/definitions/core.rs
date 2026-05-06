use crate::subcommands::tui::slash_commands::handlers;

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
        executor: handlers::clear::execute_clear_command,
    },
    CommandDefinition {
        id: CommandId::Explain,
        display_name: "/explain",
        hint: "Explain last code block",
        detail: "Ask the active model to walk through the most recent fenced code block.",
        names: &[CommandName {
            name: "/explain",
            visible: true,
        }],
        executor: handlers::explain::execute_explain_command,
    },
    CommandDefinition {
        id: CommandId::Model,
        display_name: "/model",
        hint: "Pick a model to pin",
        detail: "Open the model picker; choose Auto to resume routing.",
        names: &[CommandName {
            name: "/model",
            visible: true,
        }],
        executor: handlers::session::execute_open_models_command,
    },
    CommandDefinition {
        id: CommandId::Backend,
        display_name: "/backend",
        hint: "List backend readiness",
        detail: "Show configured and unavailable backends.",
        names: &[CommandName {
            name: "/backend",
            visible: true,
        }],
        executor: handlers::backends::execute_backends_command,
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
        executor: handlers::rules::execute_rules_command,
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
        executor: handlers::session::execute_open_help_command,
    },
    CommandDefinition {
        id: CommandId::History,
        display_name: "/history",
        hint: "Show or save history",
        detail: "Show history or save it to a file.",
        names: &[CommandName {
            name: "/history",
            visible: true,
        }],
        executor: handlers::history::execute_history_command,
    },
    CommandDefinition {
        id: CommandId::Fix,
        display_name: "/fix",
        hint: "Fix bugs in last code or message",
        detail: "Ask the active model to find and fix bugs in the last code block, \
                 falling back to the last assistant message.",
        names: &[CommandName {
            name: "/fix",
            visible: true,
        }],
        executor: handlers::fix::execute_fix_command,
    },
    CommandDefinition {
        id: CommandId::Review,
        display_name: "/review",
        hint: "Review last code block",
        detail: "Ask the active model for a brutal-but-fair review of the most recent \
                 fenced code block.",
        names: &[CommandName {
            name: "/review",
            visible: true,
        }],
        executor: handlers::review::execute_review_command,
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
        executor: handlers::session::execute_quit_command,
    },
];
