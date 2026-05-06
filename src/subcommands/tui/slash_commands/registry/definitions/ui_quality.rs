use crate::subcommands::tui::slash_commands::handlers;

use super::super::CommandId;
use super::super::types::{CommandDefinition, CommandName};

pub(super) const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        id: CommandId::Theme,
        display_name: "/theme",
        hint: "Change color theme",
        detail: "Cycle or set the theme: dark, light, or mono.",
        names: &[CommandName {
            name: "/theme",
            visible: true,
        }],
        executor: handlers::ui_quality::execute_theme_command,
    },
    CommandDefinition {
        id: CommandId::Resize,
        display_name: "/resize",
        hint: "Change layout density",
        detail: "Cycle or set layout density: compact, normal, or focus.",
        names: &[CommandName {
            name: "/resize",
            visible: true,
        }],
        executor: handlers::ui_quality::execute_resize_command,
    },
];
