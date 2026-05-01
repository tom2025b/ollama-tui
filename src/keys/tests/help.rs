use super::super::*;
use super::support::model_event_sender;

#[test]
fn q_closes_help_without_quitting() {
    let mut app = App::new();
    app.show_help = true;

    handle_key_event(
        KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
        &mut app,
        model_event_sender(),
    );

    assert!(!app.show_help);
    assert!(!app.should_quit);
}

#[test]
fn ctrl_c_closes_help_without_quitting() {
    let mut app = App::new();
    app.show_help = true;

    handle_key_event(
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        &mut app,
        model_event_sender(),
    );

    assert!(!app.show_help);
    assert!(!app.should_quit);
}

#[test]
fn question_mark_release_does_not_reopen_help() {
    let mut app = App::new();
    app.show_help = true;

    handle_key_event(
        KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
        &mut app,
        model_event_sender(),
    );
    handle_key_event(
        KeyEvent::new_with_kind(
            KeyCode::Char('?'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        ),
        &mut app,
        model_event_sender(),
    );

    assert!(!app.show_help);
    assert!(!app.should_quit);
}

#[test]
fn ctrl_c_release_after_closing_help_does_not_quit() {
    let mut app = App::new();
    app.show_help = true;

    handle_key_event(
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        &mut app,
        model_event_sender(),
    );
    handle_key_event(
        KeyEvent::new_with_kind(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
            KeyEventKind::Release,
        ),
        &mut app,
        model_event_sender(),
    );

    assert!(!app.show_help);
    assert!(!app.should_quit);
}
