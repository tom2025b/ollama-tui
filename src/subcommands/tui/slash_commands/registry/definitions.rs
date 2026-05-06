use crate::subcommands::tui::slash_commands::handlers;

use super::types::{CommandDefinition, CommandName};

// Public build only. Private slash commands (`/claude`, `/codex`, `/cost`) must
// not be added here; their absence is enforced by registry tests.
pub(super) const COMMAND_GROUPS: &[&[CommandDefinition]] = &[
    CORE_COMMANDS,
    CONTEXT_COMMANDS,
    HISTORY_OUTPUT_COMMANDS,
    UI_QUALITY_COMMANDS,
];

const CORE_COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        display_name: "/clear",
        hint: "Clear visible conversation",
        detail: "Clear the visible conversation.",
        names: &[CommandName {
            name: "/clear",
            visible: true,
        }],
        execute: handlers::clear::clear_command,
    },
    CommandDefinition {
        display_name: "/explain",
        hint: "Explain last code block",
        detail: "Ask the active model to walk through the most recent fenced code block.",
        names: &[CommandName {
            name: "/explain",
            visible: true,
        }],
        execute: handlers::explain::explain_command,
    },
    CommandDefinition {
        display_name: "/model",
        hint: "Pick a model to pin",
        detail: "Open the model picker; choose Auto to resume routing.",
        names: &[CommandName {
            name: "/model",
            visible: true,
        }],
        execute: handlers::session::open_models_command,
    },
    CommandDefinition {
        display_name: "/backend",
        hint: "List backend readiness",
        detail: "Show local and terminal route targets.",
        names: &[CommandName {
            name: "/backend",
            visible: true,
        }],
        execute: handlers::backends::backends_command,
    },
    CommandDefinition {
        display_name: "/rules",
        hint: "Edit or toggle rules",
        detail: "Edit, show, enable, disable, or toggle rule loading.",
        names: &[CommandName {
            name: "/rules",
            visible: true,
        }],
        execute: handlers::rules::rules_command,
    },
    CommandDefinition {
        display_name: "/help",
        hint: "Open help overlay",
        detail: "Open the help overlay.",
        names: &[CommandName {
            name: "/help",
            visible: true,
        }],
        execute: handlers::session::open_help_command,
    },
    CommandDefinition {
        display_name: "/history",
        hint: "Show or save history",
        detail: "Show history or save it to a file.",
        names: &[CommandName {
            name: "/history",
            visible: true,
        }],
        execute: handlers::history::history_command,
    },
    CommandDefinition {
        display_name: "/fix",
        hint: "Fix bugs in last code or message",
        detail: "Ask the active model to find and fix bugs in the last code block, \
                 falling back to the last assistant message.",
        names: &[CommandName {
            name: "/fix",
            visible: true,
        }],
        execute: handlers::fix::fix_command,
    },
    CommandDefinition {
        display_name: "/review",
        hint: "Review last code block",
        detail: "Ask the active model for a brutal-but-fair review of the most recent \
                 fenced code block.",
        names: &[CommandName {
            name: "/review",
            visible: true,
        }],
        execute: handlers::review::review_command,
    },
    CommandDefinition {
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
        execute: handlers::session::quit_command,
    },
];

const CONTEXT_COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        display_name: "/context",
        hint: "Show context window",
        detail: "Show remembered turns that feed the next prompt.",
        names: &[CommandName {
            name: "/context",
            visible: true,
        }],
        execute: handlers::context_memory::context_command,
    },
    CommandDefinition {
        display_name: "/tokens",
        hint: "Estimate context tokens",
        detail: "Estimate tokens in model history and next context.",
        names: &[CommandName {
            name: "/tokens",
            visible: true,
        }],
        execute: handlers::context_memory::tokens_command,
    },
    CommandDefinition {
        display_name: "/bookmark",
        hint: "Remember latest turn",
        detail: "Add or remove the latest completed turn from future context.",
        names: &[CommandName {
            name: "/bookmark",
            visible: true,
        }],
        execute: handlers::context_memory::bookmark_command,
    },
    CommandDefinition {
        display_name: "/memory",
        hint: "Show or clear memory",
        detail: "Show or clear session and persisted project memory.",
        names: &[CommandName {
            name: "/memory",
            visible: true,
        }],
        execute: handlers::context_memory::memory_command,
    },
    CommandDefinition {
        display_name: "/pin",
        hint: "Persist project context",
        detail: "Persist a project note that is injected into future prompts.",
        names: &[CommandName {
            name: "/pin",
            visible: true,
        }],
        execute: handlers::context_memory::pin_command,
    },
];

const HISTORY_OUTPUT_COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        display_name: "/summary",
        hint: "Summarize session",
        detail: "Show counts, models, and latest prompt for this session.",
        names: &[CommandName {
            name: "/summary",
            visible: true,
        }],
        execute: handlers::history_output::summary_command,
    },
    CommandDefinition {
        display_name: "/export",
        hint: "Export history",
        detail: "Save a history report to a path or default history file.",
        names: &[CommandName {
            name: "/export",
            visible: true,
        }],
        execute: handlers::history_output::export_command,
    },
];

const UI_QUALITY_COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        display_name: "/theme",
        hint: "Change color theme",
        detail: "Cycle or set the theme: dark, light, or mono.",
        names: &[CommandName {
            name: "/theme",
            visible: true,
        }],
        execute: handlers::ui_quality::theme_command,
    },
    CommandDefinition {
        display_name: "/resize",
        hint: "Change layout density",
        detail: "Cycle or set layout density: compact, normal, or focus.",
        names: &[CommandName {
            name: "/resize",
            visible: true,
        }],
        execute: handlers::ui_quality::resize_command,
    },
];
