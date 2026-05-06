use super::super::{App, ChatMessage};

fn message_with_answer(prompt: &str, answer: &str) -> ChatMessage {
    ChatMessage {
        prompt: prompt.to_string(),
        model_name: "Ollama llama3".to_string(),
        route_reason: "test route".to_string(),
        answer: answer.to_string(),
        in_progress: false,
        failed: false,
        include_in_context: true,
        is_local_message: false,
    }
}

#[test]
fn explain_with_code_block_sends_prompt_to_model() {
    let mut app = App::new();
    app.session.history.push(message_with_answer(
        "show me a snippet",
        "Sure:\n```rust\nfn hi() {}\n```",
    ));
    app.session.input = "/explain".to_string();

    let request = app
        .submit_prompt()
        .expect("explain should produce a model request");

    assert!(request.prompt.contains("Explain this code"));
    assert!(request.prompt.contains("fn hi() {}"));
    assert!(app.session.waiting_for_model);
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
fn review_with_code_block_sends_prompt_to_model() {
    let mut app = App::new();
    app.session
        .history
        .push(message_with_answer("draft", "```py\nprint('hi')\n```"));
    app.session.input = "/review".to_string();

    let request = app
        .submit_prompt()
        .expect("review should produce a model request");

    assert!(request.prompt.contains("Review the following code"));
    assert!(request.prompt.contains("print('hi')"));
}

#[test]
fn fix_prefers_code_block_over_assistant_message() {
    let mut app = App::new();
    app.session.history.push(message_with_answer(
        "patch",
        "Here:\n```rust\nlet n = 0; n + 1\n```",
    ));
    app.session.input = "/fix".to_string();

    let request = app
        .submit_prompt()
        .expect("fix should produce a model request");

    assert!(request.prompt.contains("Analyze this code for bugs"));
    assert!(request.prompt.contains("let n = 0; n + 1"));
}

#[test]
fn fix_falls_back_to_last_assistant_message_when_no_code_block() {
    let mut app = App::new();
    app.session.history.push(message_with_answer(
        "claim",
        "The capital of Australia is Sydney.",
    ));
    app.session.input = "/fix".to_string();

    let request = app
        .submit_prompt()
        .expect("fix should produce a model request");

    assert!(
        request
            .prompt
            .contains("Check the following message for factual errors")
    );
    assert!(request.prompt.contains("Sydney"));
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
    let mut in_flight = message_with_answer("ask", "partial");
    in_flight.in_progress = true;
    app.session.history.push(in_flight);
    app.session.input = "/fix".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    let last = app.session.history.last().expect("local message");
    assert!(last.answer.contains("Nothing to fix"));
}
