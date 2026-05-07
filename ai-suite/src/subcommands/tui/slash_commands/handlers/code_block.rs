use crate::subcommands::tui::app::App;

/// Most recent fenced code block from the visible history, walking newest first.
///
/// Both assistant answers and user prompts are scanned, since users frequently
/// paste code into prompts and ask the model to react to it.
pub fn last_code_block(app: &App) -> Option<String> {
    for entry in app.session.history.iter().rev() {
        if let Some(block) = extract_last_fenced_code_block(&entry.answer) {
            return Some(block);
        }
        if let Some(block) = extract_last_fenced_code_block(&entry.prompt) {
            return Some(block);
        }
    }

    None
}

/// Most recent completed assistant answer.
///
/// In-progress and failed turns are skipped so /fix never asks the model to
/// reason about a half-written or error-tagged response.
pub fn last_assistant_message(app: &App) -> Option<String> {
    app.session
        .history
        .iter()
        .rev()
        .find(|entry| !entry.in_progress && !entry.failed && !entry.answer.trim().is_empty())
        .map(|entry| entry.answer.clone())
}

fn extract_last_fenced_code_block(text: &str) -> Option<String> {
    let mut last_block = None;
    let mut in_block = false;
    let mut current: Vec<&str> = Vec::new();

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
            current.push(line);
        }
    }

    last_block
}

#[cfg(test)]
mod tests {
    use super::extract_last_fenced_code_block;

    #[test]
    fn returns_none_when_text_has_no_fence() {
        assert_eq!(extract_last_fenced_code_block("plain text only"), None);
    }

    #[test]
    fn extracts_single_fenced_block() {
        let text = "before\n```\nlet x = 1;\n```\nafter";
        assert_eq!(
            extract_last_fenced_code_block(text),
            Some("let x = 1;".to_string())
        );
    }

    #[test]
    fn extracts_last_block_when_multiple_present() {
        let text = "```rust\nfn first() {}\n```\nmid\n```py\nprint(\"second\")\n```";
        assert_eq!(
            extract_last_fenced_code_block(text),
            Some("print(\"second\")".to_string())
        );
    }

    #[test]
    fn unterminated_fence_is_ignored() {
        let text = "```\nstill open";
        assert_eq!(extract_last_fenced_code_block(text), None);
    }
}
