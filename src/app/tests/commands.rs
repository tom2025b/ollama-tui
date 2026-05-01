use super::super::App;
use super::support::completed_message;

#[test]
fn clear_command_clears_history_without_model_request() {
    let mut app = App::new();
    app.history.push(completed_message(1));
    app.input = "/clear".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(app.history.is_empty());
    assert_eq!(app.status, "Conversation cleared.");
}

#[test]
fn models_command_opens_picker_without_model_request() {
    let mut app = App::new();
    app.input = "/models".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(app.show_models_picker);
    assert!(app.history.is_empty());
}

#[test]
fn singular_model_command_opens_picker_without_model_request() {
    let mut app = App::new();
    app.input = "/model".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(app.show_models_picker);
    assert!(app.history.is_empty());
}

#[test]
fn singular_backend_command_adds_local_message_without_model_request() {
    let mut app = App::new();
    app.input = "/backend".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    let message = app.history.last().expect("local command message");
    assert_eq!(message.prompt, "/backend");
    assert_eq!(message.model_name, "ollama-me");
    assert!(!message.include_in_context);
    assert!(message.answer.contains("Ollama"));
}

#[test]
fn help_command_opens_help_without_model_request() {
    let mut app = App::new();
    app.input = "/help".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(app.show_help);
    assert!(app.history.is_empty());
}

#[test]
fn quit_command_exits_without_model_request() {
    let mut app = App::new();
    app.input = "/quit".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(app.should_quit);
    assert!(app.history.is_empty());
}
