use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::subcommands::tui::app::{App, ChatMessage};
use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

use super::history::history_report;

pub fn summary_command(app: &mut App, command: &ParsedCommand) {
    app.append_local_message(command.raw(), summary_report(app));
    app.ui.status = "Displayed summary.".to_string();
}

pub fn export_command(app: &mut App, command: &ParsedCommand) {
    let requested_path = command.args().first().map(String::as_str);
    let report = history_report(app);

    match crate::storage::history::save_report(app.runtime.paths(), &report, requested_path) {
        Ok(path) => {
            app.append_local_message(
                command.raw(),
                format!("Exported history to {}.", path.display()),
            );
            app.ui.status = "Exported history.".to_string();
        }
        Err(error) => {
            app.append_local_message(command.raw(), format!("Could not export: {error}"));
            app.ui.status = "Failed to export history.".to_string();
        }
    }
}

fn summary_report(app: &App) -> String {
    let model_turns = app
        .session
        .history
        .iter()
        .filter(|entry| is_model_turn(entry))
        .collect::<Vec<_>>();
    let completed = model_turns
        .iter()
        .filter(|entry| !entry.in_progress && !entry.failed)
        .count();
    let failed = model_turns.iter().filter(|entry| entry.failed).count();
    let streaming = model_turns.iter().filter(|entry| entry.in_progress).count();
    let remembered = model_turns
        .iter()
        .filter(|entry| entry.include_in_context)
        .count();
    let models: BTreeSet<&str> = model_turns
        .iter()
        .map(|entry| entry.model_name.as_str())
        .collect();

    let mut report = String::new();
    let _ = writeln!(report, "Session summary");
    let _ = writeln!(report, "Turns: {}", model_turns.len());
    let _ = writeln!(
        report,
        "Completed: {completed}, failed: {failed}, streaming: {streaming}"
    );
    let _ = writeln!(report, "Remembered for context: {remembered}");
    let _ = writeln!(report, "Models used: {}", join_models(&models));
    if let Some(last) = model_turns.last() {
        let _ = writeln!(report, "Latest prompt: {}", preview(&last.prompt));
    }
    report
}

fn join_models(models: &BTreeSet<&str>) -> String {
    if models.is_empty() {
        "none".to_string()
    } else {
        models.iter().copied().collect::<Vec<_>>().join(", ")
    }
}

fn is_model_turn(entry: &ChatMessage) -> bool {
    !entry.is_local_message
}

fn preview(text: &str) -> String {
    let preview = text.chars().take(80).collect::<String>();
    if text.chars().count() > 80 {
        format!("{preview}...")
    } else {
        preview
    }
}
