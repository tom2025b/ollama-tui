use super::*;
use crate::command::CommandRegistry;

fn completed_message(number: usize) -> ChatMessage {
    ChatMessage {
        prompt: format!("prompt {number}"),
        model_name: "Ollama llama3".to_string(),
        route_reason: "test route".to_string(),
        answer: format!("answer {number}"),
        in_progress: false,
        failed: false,
        include_in_context: true,
    }
}

#[test]
fn conversation_context_is_bounded_to_recent_completed_turns() {
    let mut app = App::new();
    for number in 0..10 {
        app.history.push(completed_message(number));
    }

    let context = app.conversation_context();

    assert_eq!(context.len(), MAX_CONTEXT_TURNS);
    assert_eq!(
        context.first().expect("first context turn").user,
        "prompt 4"
    );
    assert_eq!(context.last().expect("last context turn").user, "prompt 9");
}

#[test]
fn trim_history_keeps_recent_turns_only() {
    let mut app = App::new();
    for number in 0..(MAX_STORED_TURNS + 3) {
        app.history.push(completed_message(number));
    }

    app.trim_history();

    assert_eq!(app.history.len(), MAX_STORED_TURNS);
    assert_eq!(
        app.history.first().expect("first stored turn").prompt,
        "prompt 3"
    );
    assert_eq!(
        app.history.last().expect("last stored turn").prompt,
        format!("prompt {}", MAX_STORED_TURNS + 2)
    );
}

#[test]
fn token_events_update_active_message() {
    let mut app = App::new();
    app.history.push(ChatMessage {
        prompt: "hello".to_string(),
        model_name: "Ollama llama3".to_string(),
        route_reason: "test route".to_string(),
        answer: String::new(),
        in_progress: true,
        failed: false,
        include_in_context: true,
    });

    app.handle_model_event(ModelEvent::Token("hi".to_string()));
    app.handle_model_event(ModelEvent::Token(" there".to_string()));
    app.handle_model_event(ModelEvent::Finished);

    let message = app.history.last().expect("streamed message");
    assert_eq!(message.answer, "hi there");
    assert!(!message.in_progress);
}

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
fn models_picker_navigates_and_pins_selection() {
    let mut app = App::new();
    app.open_models_picker();
    let expected = app.pickable_models()[0].clone();
    app.select_next_model();
    app.accept_model_selection();

    assert!(!app.show_models_picker);
    assert!(app.is_pinned(&expected));
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
    assert!(!app.current_model_label().contains("(pinned)"));
}

#[test]
fn pinned_model_overrides_router_for_new_prompts() {
    let mut app = App::new();
    app.open_models_picker();
    let pinned = app.pickable_models()[0].clone();
    app.select_next_model();
    app.accept_model_selection();

    app.input = "what is the latest news today".to_string();
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

    assert!(!app.show_models_picker);
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

#[test]
fn command_messages_are_not_sent_as_context() {
    let mut app = App::new();
    app.history.push(completed_message(1));
    app.append_local_message("/help", "local output".to_string());

    let context = app.conversation_context();

    assert_eq!(context.len(), 1);
    assert_eq!(context[0].user, "prompt 1");
}

#[test]
fn command_suggestions_empty_for_normal_input() {
    let mut app = App::new();
    app.input = "hello".to_string();
    assert!(app.command_suggestions().is_empty());
}

#[test]
fn command_suggestions_show_all_commands_for_slash_alone() {
    let mut app = App::new();
    app.input = "/".to_string();
    let suggestions = app.command_suggestions();
    assert_eq!(suggestions, CommandRegistry::default().suggestions("/"));
}

#[test]
fn command_suggestions_filter_by_prefix() {
    let mut app = App::new();
    app.input = "/m".to_string();
    let names: Vec<&str> = app
        .command_suggestions()
        .into_iter()
        .map(|suggestion| suggestion.name)
        .collect();
    assert_eq!(names, vec!["/model", "/models"]);
}

#[test]
fn command_suggestions_hidden_after_whitespace() {
    let mut app = App::new();
    app.input = "/rules ".to_string();
    assert!(app.command_suggestions().is_empty());
}

#[test]
fn accept_suggestion_replaces_input_and_appends_space() {
    let mut app = App::new();
    app.input = "/h".to_string();
    let accepted = app.accept_suggestion();
    assert!(accepted);
    assert_eq!(app.input, "/help ");
    assert!(app.command_suggestions().is_empty());
}

#[test]
fn select_next_wraps_around_match_list() {
    let mut app = App::new();
    app.input = "/m".to_string();
    app.select_next_suggestion();
    assert_eq!(app.suggestion_index(), 1);
    app.select_next_suggestion();
    assert_eq!(app.suggestion_index(), 0);
}

#[test]
fn select_previous_wraps_to_end() {
    let mut app = App::new();
    app.input = "/m".to_string();
    app.select_previous_suggestion();
    assert_eq!(app.suggestion_index(), 1);
}

#[test]
fn select_next_is_noop_without_suggestions() {
    let mut app = App::new();
    app.input = "hello".to_string();
    app.select_next_suggestion();
    assert_eq!(app.suggestion_index(), 0);
}

#[test]
fn dismiss_suggestions_hides_popup_until_input_changes() {
    let mut app = App::new();
    app.input = "/".to_string();
    assert!(!app.command_suggestions().is_empty());

    app.dismiss_suggestions();
    assert!(app.command_suggestions().is_empty());

    app.push_input_char('h');
    assert!(!app.command_suggestions().is_empty());
}

#[test]
fn suggestion_index_clamps_when_match_list_shrinks() {
    let mut app = App::new();
    app.input = "/m".to_string();
    app.select_next_suggestion();
    assert_eq!(app.suggestion_index(), 1);

    app.input.push('o');
    let suggestions = app.command_suggestions();
    assert_eq!(suggestions.len(), 2);
    app.input = "/model".to_string();
    let suggestions = app.command_suggestions();
    assert_eq!(suggestions.len(), 2);
    app.input = "/models".to_string();
    let suggestions = app.command_suggestions();
    assert_eq!(suggestions.len(), 1);
    assert_eq!(app.suggestion_index(), 0);
}
