use super::super::App;
use super::support::completed_message;
use crate::subcommands::tui::slash_commands::ExternalAction;

#[test]
fn clear_command_clears_history_without_model_request() {
    let mut app = App::new();
    app.session.history.push(completed_message(1));
    app.session.input = "/clear".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(app.session.history.is_empty());
    assert_eq!(app.ui.status, "Conversation cleared.");
}

#[test]
fn model_command_opens_picker_without_model_request() {
    let mut app = App::new();
    app.session.input = "/model".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(app.ui.show_models_picker);
    assert!(app.session.history.is_empty());
}

#[test]
fn singular_backend_command_adds_local_message_without_model_request() {
    let mut app = App::new();
    app.session.input = "/backend".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    let message = app.session.history.last().expect("local command message");
    assert_eq!(message.prompt, "/backend");
    assert_eq!(message.model_name, "ollama-me");
    assert!(!message.include_in_context);
    assert!(message.answer.contains("Ollama"));
}

#[test]
fn help_command_opens_help_without_model_request() {
    let mut app = App::new();
    app.session.input = "/help".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(app.ui.show_help);
    assert!(app.session.history.is_empty());
}

#[test]
fn cost_command_queues_external_action_without_model_request() {
    let mut app = App::new();
    app.session.input = "/cost".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(matches!(
        app.take_external_action(),
        Some(ExternalAction::CostReport)
    ));
    assert!(app.session.history.is_empty());
    assert_eq!(app.ui.status, "Opening cost tracker.");
}

#[test]
fn quit_command_exits_without_model_request() {
    let mut app = App::new();
    app.session.input = "/quit".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(app.should_quit);
    assert!(app.session.history.is_empty());
}
