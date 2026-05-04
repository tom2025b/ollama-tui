mod context_memory;
mod core;
mod history_output;
mod ui_quality;

use super::types::CommandDefinition;

pub(super) const COMMAND_GROUPS: &[&[CommandDefinition]] = &[
    core::COMMANDS,
    context_memory::COMMANDS,
    history_output::COMMANDS,
    ui_quality::COMMANDS,
];
