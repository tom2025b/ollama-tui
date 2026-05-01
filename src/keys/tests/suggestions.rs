use super::super::*;
use super::support::model_event_sender;

#[test]
fn tab_with_suggestions_accepts_selection() {
    let mut app = App::new();
    app.input = "/he".to_string();

    handle_key_event(
        KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
        &mut app,
        model_event_sender(),
    );

    assert_eq!(app.input, "/help ");
}

#[test]
fn tab_without_suggestions_does_nothing() {
    let mut app = App::new();
    app.input = "hello".to_string();

    handle_key_event(
        KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
        &mut app,
        model_event_sender(),
    );

    assert_eq!(app.input, "hello");
}

#[test]
fn down_with_suggestions_moves_highlight() {
    let mut app = App::new();
    app.input = "/m".to_string();

    handle_key_event(
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        &mut app,
        model_event_sender(),
    );

    assert_eq!(app.suggestion_index(), 1);
}

#[test]
fn esc_with_suggestions_dismisses_without_quitting() {
    let mut app = App::new();
    app.input = "/".to_string();

    handle_key_event(
        KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        &mut app,
        model_event_sender(),
    );

    assert!(!app.should_quit);
    assert!(app.command_suggestions().is_empty());
}

#[test]
fn enter_with_suggestions_runs_local_command() {
    let mut app = App::new();
    app.input = "/he".to_string();

    handle_key_event(
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        &mut app,
        model_event_sender(),
    );

    assert!(app.show_help);
    assert!(app.input.is_empty());
}
