use std::fmt::Write as _;

use crate::subcommands::tui::app::{App, ChatMessage, MAX_CONTEXT_TURNS};
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

const BOOKMARK_USAGE: &str = "Usage: /bookmark [add|remove]";
const MEMORY_USAGE: &str = "Usage: /memory [show|clear]";

pub fn context_command(app: &mut App, command: &ParsedCommand) {
    app.append_local_message(command.raw(), context_report(app));
    app.ui.status = "Displayed context window.".to_string();
}

pub fn tokens_command(app: &mut App, command: &ParsedCommand) {
    app.append_local_message(command.raw(), token_report(app));
    app.ui.status = "Estimated token usage.".to_string();
}

pub fn bookmark_command(app: &mut App, command: &ParsedCommand) {
    let action = first_arg_or(command, "add");

    match action.as_str() {
        "add" | "latest" | "on" => set_latest_bookmark(app, command.raw(), true),
        "remove" | "clear" | "off" => set_latest_bookmark(app, command.raw(), false),
        _ => show_usage(
            app,
            command.raw(),
            BOOKMARK_USAGE,
            "Unknown /bookmark command.",
        ),
    }
}

pub fn memory_command(app: &mut App, command: &ParsedCommand) {
    let action = first_arg_or(command, "show");

    match action.as_str() {
        "show" | "status" => {
            app.append_local_message(command.raw(), memory_report(app));
            app.ui.status = "Displayed memory.".to_string();
        }
        "clear" | "forget" => {
            let count = app.clear_context_memory();
            app.append_local_message(
                command.raw(),
                format!("Forgot {count} turn(s) from future context."),
            );
            app.ui.status = "Cleared context memory.".to_string();
        }
        _ => show_usage(app, command.raw(), MEMORY_USAGE, "Unknown /memory command."),
    }
}

fn set_latest_bookmark(app: &mut App, input: &str, remember: bool) {
    match app.include_latest_history_entry(remember) {
        Some(prompt) => {
            let verb = if remember { "Bookmarked" } else { "Removed" };
            app.append_local_message(input, format!("{verb}: {}", preview(&prompt)));
            app.ui.status = bookmark_status(remember).to_string();
        }
        None => {
            app.append_local_message(input, "No completed model turn to update.".to_string());
            app.ui.status = "No bookmark target.".to_string();
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

fn show_usage(app: &mut App, input: &str, message: &str, status: &str) {
    app.append_local_message(input, message.to_string());
    app.ui.status = status.to_string();
}

fn context_report(app: &App) -> String {
    let entries = &app.session.history;
    let model_turns = entries.iter().filter(|entry| is_model_turn(entry)).count();
    let remembered = entries
        .iter()
        .filter(|entry| ready_for_context(entry))
        .count();
    let next = entries
        .iter()
        .rev()
        .filter(|entry| ready_for_context(entry))
        .take(MAX_CONTEXT_TURNS)
        .count();

    let mut report = String::new();
    let _ = writeln!(
        report,
        "Context window: {next}/{} turn(s)",
        MAX_CONTEXT_TURNS
    );
    let _ = writeln!(report, "Remembered turns: {remembered}/{model_turns}");
    if let Some(last) = entries.iter().rev().find(|entry| ready_for_context(entry)) {
        let _ = writeln!(report, "Latest remembered: {}", preview(&last.prompt));
    }
    report
}

fn memory_report(app: &App) -> String {
    let mut report = context_report(app);
    report.push_str("\nUse /bookmark to remember the latest turn.\n");
    report.push_str("Use /memory clear to forget remembered turns.");
    report
}

fn token_report(app: &App) -> String {
    let entries = &app.session.history;
    let next_tokens: usize = entries
        .iter()
        .rev()
        .filter(|entry| ready_for_context(entry))
        .take(MAX_CONTEXT_TURNS)
        .map(entry_tokens)
        .sum();
    let total_tokens: usize = entries
        .iter()
        .filter(|entry| is_model_turn(entry))
        .map(entry_tokens)
        .sum();

    format!(
        "Estimated tokens:\nNext request context: {next_tokens}\nModel history: {total_tokens}\nEstimator: characters / 4."
    )
}

fn preview(text: &str) -> String {
    let preview = text.chars().take(80).collect::<String>();
    if text.chars().count() > 80 {
        format!("{preview}...")
    } else {
        preview
    }
}

fn entry_tokens(entry: &ChatMessage) -> usize {
    estimate_tokens(&entry.prompt) + estimate_tokens(&entry.answer)
}

fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

fn ready_for_context(entry: &ChatMessage) -> bool {
    entry.include_in_context && is_model_turn(entry) && !entry.in_progress && !entry.failed
}

fn is_model_turn(entry: &ChatMessage) -> bool {
    !entry.is_local_message
}
