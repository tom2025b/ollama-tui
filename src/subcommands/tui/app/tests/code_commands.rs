use super::super::{App, ChatMessage};
use crate::subcommands::tui::slash_commands::ExternalAction;

fn message_with_answer(prompt: &str, answer: &str) -> ChatMessage {
    let mut message = ChatMessage::streaming_model_turn(
        prompt.to_string(),
        "Ollama llama3".to_string(),
        "test route".to_string(),
    );
    message.append_token(answer);
    message.finish_streaming();
    message
}

fn assert_claude_launch(app: &mut App) -> String {
    let request = app.submit_prompt();
    let action = app.take_external_action();

    assert!(request.is_none());
    assert!(matches!(action, Some(ExternalAction::ClaudeCode { .. })));
    assert!(!app.session.waiting_for_model);

    app.session
        .history
        .last()
        .expect("launch message")
        .prompt
        .clone()
}

#[test]
fn explain_with_code_block_launches_claude() {
    let mut app = App::new();
    app.session.history.push(message_with_answer(
        "show me a snippet",
        "Sure:\n```rust\nfn hi() {}\n```",
    ));
    app.session.input = "/explain".to_string();

    let prompt = assert_claude_launch(&mut app);

    assert!(prompt.contains("Explain this code"));
    assert!(prompt.contains("fn hi() {}"));
}

#[test]
fn explain_without_code_block_appends_local_message_only() {
    let mut app = App::new();
    app.session.input = "/explain".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(!app.session.waiting_for_model);
    let last = app.session.history.last().expect("local message");
    assert_eq!(last.model_name, "ai-suite");
    assert!(last.answer.contains("No fenced code block"));
}

#[test]
fn review_with_code_block_launches_claude() {
    let mut app = App::new();
    app.session
        .history
        .push(message_with_answer("draft", "```py\nprint('hi')\n```"));
    app.session.input = "/review".to_string();

    let prompt = assert_claude_launch(&mut app);

    assert!(prompt.contains("Review the following code"));
    assert!(prompt.contains("print('hi')"));
}

#[test]
fn fix_prefers_code_block_over_assistant_message_and_launches_claude() {
    let mut app = App::new();
    app.session.history.push(message_with_answer(
        "patch",
        "Here:\n```rust\nlet n = 0; n + 1\n```",
    ));
    app.session.input = "/fix".to_string();

    let prompt = assert_claude_launch(&mut app);

    assert!(prompt.contains("Analyze this code for bugs"));
    assert!(prompt.contains("let n = 0; n + 1"));
}

#[test]
fn fix_falls_back_to_last_assistant_message_and_launches_claude() {
    let mut app = App::new();
    app.session.history.push(message_with_answer(
        "claim",
        "The capital of Australia is Sydney.",
    ));
    app.session.input = "/fix".to_string();

    let prompt = assert_claude_launch(&mut app);

    assert!(prompt.contains("Check the following message for factual errors"));
    assert!(prompt.contains("Sydney"));
}

#[test]
fn fix_with_empty_history_appends_local_message_only() {
    let mut app = App::new();
    app.session.input = "/fix".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(!app.session.waiting_for_model);
    let last = app.session.history.last().expect("local message");
    assert!(last.answer.contains("Nothing to fix"));
}

#[test]
fn fix_skips_in_progress_assistant_message() {
    let mut app = App::new();
    let mut in_flight = ChatMessage::streaming_model_turn(
        "ask".to_string(),
        "Ollama llama3".to_string(),
        "test route".to_string(),
    );
    in_flight.append_token("partial");
    app.session.history.push(in_flight);
    app.session.input = "/fix".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    let last = app.session.history.last().expect("local message");
    assert!(last.answer.contains("Nothing to fix"));
}
