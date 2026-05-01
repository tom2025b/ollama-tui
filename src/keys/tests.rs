use super::*;

fn model_event_sender() -> mpsc::UnboundedSender<ModelEvent> {
    let (sender, _receiver) = mpsc::unbounded_channel();
    sender
}

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

#[test]
fn models_picker_consumes_navigation_keys() {
    let mut app = App::new();
    app.open_models_picker();

    let starting_index = app.models_picker_index();
    handle_key_event(
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        &mut app,
        model_event_sender(),
    );
    assert_ne!(app.models_picker_index(), starting_index);
    assert_eq!(app.scroll_offset, 0);
}

#[test]
fn esc_in_models_picker_closes_overlay_without_quitting() {
    let mut app = App::new();
    app.open_models_picker();

    handle_key_event(
        KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        &mut app,
        model_event_sender(),
    );

    assert!(!app.show_models_picker);
    assert!(!app.should_quit);
}

#[test]
fn enter_in_models_picker_pins_selection() {
    let mut app = App::new();
    app.open_models_picker();
    let expected = app.pickable_models()[0].clone();

    handle_key_event(
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        &mut app,
        model_event_sender(),
    );
    handle_key_event(
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        &mut app,
        model_event_sender(),
    );

    assert!(!app.show_models_picker);
    assert!(app.is_pinned(&expected));
}
