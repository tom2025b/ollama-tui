use crate::subcommands::tui::slash_commands::handlers;

use super::super::CommandId;
use super::super::types::{CommandDefinition, CommandName};

pub(super) const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        id: CommandId::Summary,
        display_name: "/summary",
        hint: "Summarize session",
        detail: "Show counts, models, and latest prompt for this session.",
        names: &[CommandName {
            name: "/summary",
            visible: true,
        }],
        executor: handlers::history_output::handle_summary_command,
    },
    CommandDefinition {
        id: CommandId::Export,
        display_name: "/export",
        hint: "Export history",
        detail: "Save a history report to a path or default history file.",
        names: &[CommandName {
            name: "/export",
            visible: true,
        }],
        executor: handlers::history_output::handle_export_command,
    },
];
