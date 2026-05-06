mod report;

use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

use self::report::{context_report, memory_report, preview, token_report};
use super::session::{CommandContext, CommandOutput, ContextMemory, HistoryView};

const BOOKMARK_USAGE: &str = "Usage: /bookmark [add|remove]";
const MEMORY_USAGE: &str = "Usage: /memory [show|clear]";

pub fn handle_context_command<C>(context: &mut C, command: &ParsedCommand)
where
    C: CommandOutput + HistoryView + ?Sized,
{
    context.append_local_message(command.raw(), context_report(context));
    context.set_status("Displayed context window.".to_string());
}

pub fn execute_context_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    handle_context_command(context, command);
}

pub fn handle_tokens_command<C>(context: &mut C, command: &ParsedCommand)
where
    C: CommandOutput + HistoryView + ?Sized,
{
    context.append_local_message(command.raw(), token_report(context));
    context.set_status("Estimated token usage.".to_string());
}

pub fn execute_tokens_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    handle_tokens_command(context, command);
}

pub fn handle_bookmark_command<C>(context: &mut C, command: &ParsedCommand)
where
    C: CommandOutput + ContextMemory + ?Sized,
{
    let action = first_arg_or(command, "add");

    match action.as_str() {
        "add" | "latest" | "on" => set_latest_bookmark(context, command.raw(), true),
        "remove" | "clear" | "off" => set_latest_bookmark(context, command.raw(), false),
        _ => show_usage(
            context,
            command.raw(),
            BOOKMARK_USAGE,
            "Unknown /bookmark command.",
        ),
    }
}

pub fn execute_bookmark_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    handle_bookmark_command(context, command);
}

pub fn handle_memory_command<C>(context: &mut C, command: &ParsedCommand)
where
    C: CommandOutput + ContextMemory + HistoryView + ?Sized,
{
    let action = first_arg_or(command, "show");

    match action.as_str() {
        "show" | "status" => {
            context.append_local_message(command.raw(), memory_report(context));
            context.set_status("Displayed memory.".to_string());
        }
        "clear" | "forget" => {
            let count = context.clear_context_memory();
            context.append_local_message(
                command.raw(),
                format!("Forgot {count} turn(s) from future context."),
            );
            context.set_status("Cleared context memory.".to_string());
        }
        _ => show_usage(
            context,
            command.raw(),
            MEMORY_USAGE,
            "Unknown /memory command.",
        ),
    }
}

pub fn execute_memory_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    handle_memory_command(context, command);
}

fn set_latest_bookmark<C>(context: &mut C, input: &str, remember: bool)
where
    C: CommandOutput + ContextMemory + ?Sized,
{
    match context.include_latest_history_entry(remember) {
        Some(prompt) => {
            let verb = if remember { "Bookmarked" } else { "Removed" };
            context.append_local_message(input, format!("{verb}: {}", preview(&prompt)));
            context.set_status(bookmark_status(remember).to_string());
        }
        None => {
            context.append_local_message(input, "No completed model turn to update.".to_string());
            context.set_status("No bookmark target.".to_string());
        }
    }
}

fn bookmark_status(remember: bool) -> &'static str {
    if remember {
        "Bookmarked latest turn."
    } else {
        "Removed latest bookmark."
    }
}

fn first_arg_or(command: &ParsedCommand, default: &str) -> String {
    command
        .args()
        .first()
        .map(|arg| arg.to_ascii_lowercase())
        .unwrap_or_else(|| default.to_string())
}

fn show_usage<C>(context: &mut C, input: &str, message: &str, status: &str)
where
    C: CommandOutput + ?Sized,
{
    context.append_local_message(input, message.to_string());
    context.set_status(status.to_string());
}
