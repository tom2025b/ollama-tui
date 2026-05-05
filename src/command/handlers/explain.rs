use crate::command::parser::ParsedCommand;

use super::session::CommandContext;

pub fn handle_explain_command(context: &mut dyn CommandContext, command: &ParsedCommand) {
    let Some(code_block) = last_code_block(context) else {
        context.append_local_message(
            command.raw(),
            "No fenced code block was found to explain.".to_string(),
        );
        context.set_status("No code block to explain.".to_string());
        return;
    };

    let prompt = format!(
        "Explain this code in simple terms, like I'm a junior developer. Be concise and clear:\n\n{code_block}"
    );
    context.append_local_message(command.raw(), prompt);
    context.set_status("Explain command received".to_string());
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
