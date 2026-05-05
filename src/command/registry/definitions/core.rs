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
        executor: handlers::clear::handle_clear_command,
    },
    CommandDefinition {
        id: CommandId::Codex,
        display_name: "/codex",
        hint: "Toggle Codex mode",
        detail: "Toggle a persistent coding system prompt and pin the best coding-capable backend.",
        names: &[CommandName {
            name: "/codex",
            visible: true,
        }],
        executor: handlers::codex::handle_codex_command,
    },
    CommandDefinition {
        id: CommandId::Explain,
        display_name: "/explain",
        hint: "Explain last code block",
        detail: "Explain the last code block in simple terms.",
        names: &[CommandName {
            name: "/explain",
            visible: true,
        }],
        executor: handlers::explain::handle_explain_command,
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
        executor: handlers::session::open_models_command,
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
        executor: handlers::backends::handle_backends_command,
    },
    CommandDefinition {
        id: CommandId::Cost,
        display_name: "/cost",
        hint: "Open cost tracker",
        detail: "Run the Python cost tracker, then return to the TUI.",
        names: &[CommandName {
            name: "/cost",
            visible: true,
        }],
        executor: handlers::session::cost_command,
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
        id: CommandId::Fix,
        display_name: "/fix",
        hint: "Fix last message",
        detail: "Ask the model to fix any obvious bugs in the last message.",
        names: &[CommandName {
            name: "/fix",
            visible: true,
        }],
        executor: handlers::fix::handle_fix_command,
    },
    CommandDefinition {
        id: CommandId::Review,
        display_name: "/review",
        hint: "Review last code block",
        detail: "Analyze the most recent fenced code block from the conversation.",
        names: &[CommandName {
            name: "/review",
            visible: true,
        }],
        executor: handlers::review::handle_review_command,
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
