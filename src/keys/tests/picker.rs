use super::super::*;
use super::support::model_event_sender;

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
