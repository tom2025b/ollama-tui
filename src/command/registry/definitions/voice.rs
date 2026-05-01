use crate::command::handlers;

use super::super::CommandId;
use super::super::types::{CommandDefinition, CommandName};

pub(super) const COMMANDS: &[CommandDefinition] = &[CommandDefinition {
    id: CommandId::Voice,
    display_name: "/voice",
    hint: "Configure voice",
    detail: "Configure voice on/off, speed, and mode.",
    names: &[CommandName {
        name: "/voice",
        visible: true,
    }],
    executor: handlers::voice::handle_voice_command,
}];
