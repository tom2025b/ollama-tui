use std::fmt::Write as _;

use super::super::session::{HistoryEntry, HistoryView};

pub(super) fn context_report<C>(context: &C) -> String
where
    C: HistoryView + ?Sized,
{
    let entries = context.history_entries();
    let model_turns = entries.iter().filter(|entry| is_model_turn(entry)).count();
    let remembered = entries
        .iter()
        .filter(|entry| ready_for_context(entry))
        .count();
    let next = entries
        .iter()
        .rev()
        .filter(|entry| ready_for_context(entry))
        .take(context.context_turn_limit())
        .count();

    let mut report = String::new();
    let _ = writeln!(
        report,
        "Context window: {next}/{} turn(s)",
        context.context_turn_limit()
    );
    let _ = writeln!(report, "Remembered turns: {remembered}/{model_turns}");
    if let Some(last) = entries.iter().rev().find(|entry| ready_for_context(entry)) {
        let _ = writeln!(report, "Latest remembered: {}", preview(last.prompt));
    }
    report
}

pub(super) fn memory_report<C>(context: &C) -> String
where
    C: HistoryView + ?Sized,
{
    let mut report = context_report(context);
    report.push_str("\nUse /bookmark to remember the latest turn.\n");
    report.push_str("Use /memory clear to forget remembered turns.");
    report
}

pub(super) fn token_report<C>(context: &C) -> String
where
    C: HistoryView + ?Sized,
{
    let entries = context.history_entries();
    let next_tokens: usize = entries
        .iter()
        .rev()
        .filter(|entry| ready_for_context(entry))
        .take(context.context_turn_limit())
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

pub(super) fn preview(text: &str) -> String {
    let preview = text.chars().take(80).collect::<String>();
    if text.chars().count() > 80 {
        format!("{preview}...")
    } else {
        preview
    }
}

fn entry_tokens(entry: &HistoryEntry<'_>) -> usize {
    estimate_tokens(entry.prompt) + estimate_tokens(entry.answer)
}

fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

fn ready_for_context(entry: &HistoryEntry<'_>) -> bool {
    entry.include_in_context && is_model_turn(entry) && !entry.in_progress && !entry.failed
}

fn is_model_turn(entry: &HistoryEntry<'_>) -> bool {
    !entry.is_local_message
}
