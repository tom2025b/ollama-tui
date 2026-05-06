use std::fmt::Write as _;

use crate::subcommands::tui::app::{App, ChatMessage, MAX_CONTEXT_TURNS};
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

const BOOKMARK_USAGE: &str = "Usage: /bookmark [add|remove]";
const MEMORY_USAGE: &str = "Usage: /memory [show|clear]";
const PIN_USAGE: &str = "Usage: /pin <project memory note>";

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
            let session_count = app.clear_context_memory();
            let persistent_count = match app.clear_persistent_memory() {
                Ok(count) => count,
                Err(error) => {
                    app.append_local_message(command.raw(), error);
                    app.ui.status = "Could not clear project memory.".to_string();
                    return;
                }
            };
            app.append_local_message(
                command.raw(),
                format!(
                    "Forgot {session_count} session turn(s) and {persistent_count} project memory item(s)."
                ),
            );
            app.ui.status = "Cleared context memory.".to_string();
        }
        _ => show_usage(app, command.raw(), MEMORY_USAGE, "Unknown /memory command."),
    }
}

pub fn pin_command(app: &mut App, command: &ParsedCommand) {
    let note = command.args().join(" ");
    if note.trim().is_empty() {
        show_usage(app, command.raw(), PIN_USAGE, "Missing memory note.");
        return;
    }

    match app.pin_memory_note(note.trim()) {
        Ok(()) => {
            app.append_local_message(command.raw(), format!("Pinned memory: {}", preview(&note)));
            app.ui.status = "Pinned project memory.".to_string();
        }
        Err(error) => {
            app.append_local_message(command.raw(), error);
            app.ui.status = "Could not pin project memory.".to_string();
        }
    }
}

fn set_latest_bookmark(app: &mut App, input: &str, remember: bool) {
    let result = if remember {
        app.remember_latest_history_entry()
    } else {
        app.forget_latest_history_entry()
    };

    match result {
        Ok(Some(prompt)) => {
            let verb = if remember {
                "Bookmarked"
            } else {
                "Removed bookmark"
            };
            app.append_local_message(input, format!("{verb}: {}", preview(&prompt)));
            app.ui.status = bookmark_status(remember).to_string();
        }
        Ok(None) => {
            app.append_local_message(input, "No completed model turn to update.".to_string());
            app.ui.status = "No bookmark target.".to_string();
        }
        Err(error) => {
            app.append_local_message(input, error);
            app.ui.status = "Could not update project memory.".to_string();
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
    let next = app.conversation_context().len();
    let project_memory = app.memory.items().len();

    let mut report = String::new();
    let _ = writeln!(
        report,
        "Context window: {next}/{} turn(s)",
        MAX_CONTEXT_TURNS
    );
    let _ = writeln!(report, "Remembered turns: {remembered}/{model_turns}");
    let _ = writeln!(report, "Project memory items: {project_memory}");
    if let Some(last) = entries.iter().rev().find(|entry| ready_for_context(entry)) {
        let _ = writeln!(report, "Latest remembered: {}", preview(&last.prompt));
    }
    report
}

fn memory_report(app: &App) -> String {
    let mut report = context_report(app);
    report.push('\n');
    if app.memory.items().is_empty() {
        report.push_str("Project memory: empty\n");
    } else {
        let _ = writeln!(
            report,
            "Project memory: {} item(s)",
            app.memory.items().len()
        );
        for (index, item) in app.memory.items().iter().rev().take(8).enumerate() {
            let _ = writeln!(
                report,
                "{}. [{}] {}",
                index + 1,
                item.label(),
                preview(item.display_content())
            );
        }
    }
    report.push_str("\nUse /bookmark to persist the latest turn.\n");
    report.push_str("Use /pin <note> to persist a project note.\n");
    report.push_str("Use /memory clear to forget session and project memory.");
    report
}

fn token_report(app: &App) -> String {
    let entries = &app.session.history;
    let next_tokens: usize = app
        .conversation_context()
        .iter()
        .map(|turn| estimate_tokens(&turn.user) + estimate_tokens(&turn.assistant))
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
