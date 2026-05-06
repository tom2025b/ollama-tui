use super::super::App;

#[test]
fn models_picker_navigates_and_pins_selection() {
    let mut app = App::new();
    app.open_models_picker();
    let expected = app.pickable_models()[0].clone();
    app.select_next_model();
    app.accept_model_selection();

    assert!(!app.ui.show_models_picker);
    assert!(app.is_pinned(&expected));
    assert!(app.has_pinned_model());
    assert_eq!(app.routing_mode_label(), "PINNED");
    assert!(app.current_model_label().contains("(pinned)"));
    assert!(
        app.current_model_label()
            .contains(&expected.display_label())
    );
}

#[test]
fn models_picker_auto_entry_clears_pin() {
    let mut app = App::new();
    app.open_models_picker();
    let first = app.pickable_models()[0].clone();
    app.select_next_model();
    app.accept_model_selection();
    assert!(app.is_pinned(&first));

    app.open_models_picker();
    while app.models_picker_index() != 0 {
        app.select_previous_model();
    }
    app.accept_model_selection();

    assert!(!app.is_pinned(&first));
    assert!(!app.has_pinned_model());
    assert_eq!(app.routing_mode_label(), "AUTO ROUTER");
    assert!(!app.current_model_label().contains("(pinned)"));
}

#[test]
fn pinned_model_overrides_router_for_new_prompts() {
    let mut app = App::new();
    app.open_models_picker();
    let pinned = app.pickable_models()[0].clone();
    app.select_next_model();
    app.accept_model_selection();

    app.session.input = "what is the latest news today".to_string();
    let request = app.submit_prompt().expect("submitted request");

    assert_eq!(request.route.model.display_label(), pinned.display_label());
    assert!(request.route.reason.contains("Pinned"));
}

#[test]
fn esc_on_picker_cancels_without_changing_pin() {
    let mut app = App::new();
    app.open_models_picker();
    let candidate = app.pickable_models()[0].clone();
    app.select_next_model();
    app.close_models_picker();

    assert!(!app.ui.show_models_picker);
    assert!(!app.is_pinned(&candidate));
}

#[test]
fn select_next_model_wraps_around_picker() {
    let mut app = App::new();
    app.open_models_picker();
    let total = app.models_picker_total();
    for _ in 0..total {
        app.select_next_model();
    }
    assert_eq!(app.models_picker_index(), 0);
}
