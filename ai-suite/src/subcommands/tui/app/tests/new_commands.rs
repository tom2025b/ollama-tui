use std::fs;

use super::super::App;
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
fn bookmark_adds_latest_turn_back_to_context() {
    let mut app = App::new();
    let mut message = completed_message(1);
    message.include_in_context = false;
    app.session.history.push(message);
    app.session.input = "/bookmark".to_string();

    let request = app.submit_prompt();

    assert!(request.is_none());
    assert!(app.session.history[0].include_in_context);
    assert_eq!(app.ui.status, "Bookmarked latest turn.");
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
