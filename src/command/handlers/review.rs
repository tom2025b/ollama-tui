use crate::command::parser::ParsedCommand;

use super::session::CommandContext;

pub fn handle_review_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    match last_code_block(context) {
        Some(code_block) => {
            context.append_local_message(command.raw(), review_report(&code_block));
            context.set_status("Reviewed the last code block.".to_string());
        }
        None => {
            context.append_local_message(
                command.raw(),
                "No fenced code block was found in the recent conversation.".to_string(),
            );
            context.set_status("No code block to review.".to_string());
        }
    }
}

fn last_code_block(context: &dyn CommandContext) -> Option<String> {
    let mut history = context.history_entries();
    history.reverse();

    for entry in history {
        if let Some(block) = extract_last_fenced_code_block(entry.answer) {
            return Some(block);
        }
        if let Some(block) = extract_last_fenced_code_block(entry.prompt) {
            return Some(block);
        }
    }

    None
}

fn extract_last_fenced_code_block(text: &str) -> Option<String> {
    let mut last_block = None;
    let mut in_block = false;
    let mut current = Vec::new();

    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            if in_block {
                last_block = Some(current.join("\n"));
                current.clear();
            }
            in_block = !in_block;
            continue;
        }

        if in_block {
            current.push(line.to_string());
        }
    }

    last_block
}

fn review_report(code_block: &str) -> String {
    let line_count = code_block.lines().count();
    let non_empty_lines = code_block.lines().filter(|line| !line.trim().is_empty()).count();

    format!(
        "Last code block review\nLines: {line_count}\nNon-empty lines: {non_empty_lines}\n\n{code_block}"
    )
}
