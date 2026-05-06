use super::super::{App, ChatMessage, MAX_CONTEXT_TURNS, MAX_STORED_TURNS, ModelEvent};
use super::support::completed_message;

#[test]
fn conversation_context_is_bounded_to_recent_completed_turns() {
    let mut app = App::new();
    for number in 0..10 {
        app.session.history.push(completed_message(number));
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
        app.session.history.push(completed_message(number));
    }

    app.trim_history();

    assert_eq!(app.session.history.len(), MAX_STORED_TURNS);
    assert_eq!(
        app.session
            .history
            .first()
            .expect("first stored turn")
            .prompt,
        "prompt 3"
    );
    assert_eq!(
        app.session.history.last().expect("last stored turn").prompt,
        format!("prompt {}", MAX_STORED_TURNS + 2)
    );
}

#[test]
fn token_events_update_active_message() {
    let mut app = App::new();
    app.session.history.push(ChatMessage {
        prompt: "hello".to_string(),
        model_name: "Ollama llama3".to_string(),
        route_reason: "test route".to_string(),
        answer: String::new(),
        in_progress: true,
        failed: false,
        include_in_context: true,
        is_local_message: false,
    });

    app.handle_model_event(ModelEvent::Token("hi".to_string()));
    app.handle_model_event(ModelEvent::Token(" there".to_string()));
    app.handle_model_event(ModelEvent::Finished);

    let message = app.session.history.last().expect("streamed message");
    assert_eq!(message.answer, "hi there");
    assert!(!message.in_progress);
}

#[test]
fn command_messages_are_not_sent_as_context() {
    let mut app = App::new();
    app.session.history.push(completed_message(1));
    app.append_local_message("/help", "local output".to_string());

    let context = app.conversation_context();

    assert_eq!(context.len(), 1);
    assert_eq!(context[0].user, "prompt 1");
}
