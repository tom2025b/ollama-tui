use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::subcommands::tui::slash_commands::parser::ParsedCommand;

use super::history::history_report;
use super::session::{
    CommandContext, CommandOutput, HistoryEntry, HistoryExport, HistoryView, RulesContext,
};

pub fn handle_summary_command<C>(context: &mut C, command: &ParsedCommand)
where
    C: CommandOutput + HistoryView + ?Sized,
{
    context.append_local_message(command.raw(), summary_report(context));
    context.set_status("Displayed summary.".to_string());
}

pub fn execute_summary_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    handle_summary_command(context, command);
}

pub fn handle_export_command<C>(context: &mut C, command: &ParsedCommand)
where
    C: CommandOutput + HistoryView + HistoryExport + RulesContext + ?Sized,
{
    let requested_path = command.args().first().map(String::as_str);
    let report = history_report(context);

    match context.save_history_report(&report, requested_path) {
        Ok(path) => {
            context.append_local_message(
                command.raw(),
                format!("Exported history to {}.", path.display()),
            );
            context.set_status("Exported history.".to_string());
        }
        Err(error) => {
            context.append_local_message(command.raw(), format!("Could not export: {error}"));
            context.set_status("Failed to export history.".to_string());
        }
    }
}

pub fn execute_export_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    handle_export_command(context, command);
}

fn summary_report<C>(context: &C) -> String
where
    C: HistoryView + ?Sized,
{
    let entries = context.history_entries();
    let model_turns = entries
        .iter()
        .copied()
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
    let models: BTreeSet<&str> = model_turns.iter().map(|entry| entry.model_name).collect();

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
        let _ = writeln!(report, "Latest prompt: {}", preview(last.prompt));
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

fn is_model_turn(entry: &HistoryEntry<'_>) -> bool {
    entry.model_name != "ollama-me"
}

fn preview(text: &str) -> String {
    let preview = text.chars().take(80).collect::<String>();
    if text.chars().count() > 80 {
        format!("{preview}...")
    } else {
        preview
    }
}
