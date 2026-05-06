use std::fs;

use super::super::{App, ChatMessage};
use super::support::completed_message;

#[test]
fn memory_clear_removes_turns_from_future_context() {
    let mut app = App::new();
    app.session.history.push(completed_message(1));
    app.session.history.push(completed_message(2));
    app.session.input = "/memory clear".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(
        app.session
            .history
            .iter()
            .take(2)
            .all(|message| !message.include_in_context)
    );
    assert_eq!(app.ui.status, "Cleared context memory.");
}

#[test]
fn memory_clear_does_not_remove_history_report_turns() {
    let mut app = App::new();
    app.session.history.push(completed_message(1));
    app.session.input = "/memory clear".to_string();
    app.submit_prompt();

    app.session.input = "/history".to_string();
    let request = app.submit_prompt();

    assert!(request.is_none());
    let report = &app.session.history.last().expect("history report").answer;
    assert!(report.contains("## Turn 1"));
    assert!(report.contains("prompt 1"));
    assert!(report.contains("answer 1"));
}

#[test]
fn context_report_ignores_empty_model_answers() {
    let mut app = App::new();
    let mut message = ChatMessage::streaming_model_turn(
        "prompt".to_string(),
        "Ollama llama3".to_string(),
        "test route".to_string(),
    );
    message.finish_streaming();
    app.session.history.push(message);
    app.session.input = "/context".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    let report = &app.session.history.last().expect("context report").answer;
    assert!(report.contains("Context window: 0/6 turn(s)"));
    assert!(report.contains("Remembered turns: 0/1"));
}

#[test]
fn bookmark_adds_latest_turn_back_to_context() {
    let mut app = App::new();
    let mut message = completed_message(1);
    message.forget_context();
    app.session.history.push(message);
    app.session.input = "/bookmark".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(app.session.history[0].include_in_context);
    assert_eq!(app.ui.status, "Bookmarked latest turn.");
}

#[test]
fn pin_command_persists_project_note() {
    let mut app = App::new();
    app.session.input = "/pin Keep every Rust module focused.".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert_eq!(app.memory.items().len(), 1);
    assert_eq!(
        app.memory.items()[0].display_content(),
        "Keep every Rust module focused."
    );
    assert_eq!(app.ui.status, "Pinned project memory.");
}

#[test]
fn summary_command_adds_local_summary() {
    let mut app = App::new();
    app.session.history.push(completed_message(1));
    app.session.input = "/summary".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    let message = app.session.history.last().expect("summary message");
    assert!(message.answer.contains("Session summary"));
    assert!(!message.include_in_context);
}

#[test]
fn export_command_writes_history_report() {
    let path = "/tmp/ai-suite-export-test.txt";
    let mut app = App::new();
    app.session.history.push(completed_message(1));
    app.session.input = format!("/export {path}");

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(
        fs::read_to_string(path)
            .unwrap()
            .contains("ai-suite history")
    );
    let _ = fs::remove_file(path);
}

#[test]
fn theme_and_resize_commands_change_ui_settings() {
    let mut app = App::new();
    app.session.input = "/theme light".to_string();
    app.submit_prompt();

    app.session.input = "/resize focus".to_string();
    app.submit_prompt();

    assert_eq!(app.theme_name(), "light");
    assert_eq!(app.layout_mode_name(), "focus");
}
