use super::super::App;
use crate::llm::Provider;
use crate::subcommands::tui::slash_commands::ExternalAction;

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
fn pinned_claude_launches_terminal_app_instead_of_model_request() {
    let mut app = App::new();
    let claude = app
        .pickable_models()
        .into_iter()
        .find(|model| model.provider == Provider::ClaudeCode)
        .expect("Claude Code target is pickable")
        .clone();
    app.routing.pinned_model = Some(claude);
    app.session.input = "compare these implementation approaches".to_string();

    let request = app.submit_prompt();
    let action = app.take_external_action();

    assert!(request.is_none());
    assert!(matches!(action, Some(ExternalAction::ClaudeCode { .. })));
    assert!(!app.session.waiting_for_model);
}

#[test]
fn terminal_launch_forwards_prompt_to_action() {
    let mut app = App::new();
    let claude = app
        .pickable_models()
        .into_iter()
        .find(|model| model.provider == Provider::ClaudeCode)
        .expect("Claude Code target is pickable")
        .clone();
    app.routing.pinned_model = Some(claude);
    app.session.input = "debug the allocator".to_string();

    app.submit_prompt();
    let action = app.take_external_action();

    match action.expect("action should be queued") {
        ExternalAction::ClaudeCode { prompt, .. } => {
            assert_eq!(prompt, "debug the allocator");
        }
        other => panic!("expected ClaudeCode action, got {other:?}"),
    }
}

#[test]
fn terminal_launch_is_not_bookmarked_as_model_output() {
    let mut app = App::new();
    let claude = app
        .pickable_models()
        .into_iter()
        .find(|model| model.provider == Provider::ClaudeCode)
        .expect("Claude Code target is pickable")
        .clone();
    app.routing.pinned_model = Some(claude);
    app.session.input = "debug the allocator".to_string();

    app.submit_prompt();

    let message = app
        .session
        .history
        .last()
        .expect("terminal launch should be visible");
    assert!(message.is_local_message);
    assert!(!message.include_in_context);
    assert!(
        app.remember_latest_history_entry()
            .expect("bookmark lookup should not fail")
            .is_none()
    );
}

#[test]
fn complex_auto_route_launches_claude_terminal_app() {
    let mut app = App::new();
    app.session.input =
        "complex: compare architectures and recommend a careful implementation approach"
            .to_string();

    let request = app.submit_prompt();
    let action = app.take_external_action();

    assert!(request.is_none());
    assert!(matches!(action, Some(ExternalAction::ClaudeCode { .. })));
    assert!(!app.session.waiting_for_model);
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
