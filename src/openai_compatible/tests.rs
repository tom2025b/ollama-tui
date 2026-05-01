use super::stream::process_chat_completion_stream_line;
use super::types::chat_messages_from_context;
use crate::llm::ConversationTurn;

#[test]
fn chat_messages_include_bounded_context_then_current_prompt() {
    let context = vec![ConversationTurn {
        user: "old prompt".to_string(),
        assistant: "old answer".to_string(),
    }];

    let messages = chat_messages_from_context(&context, "new prompt");

    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[0].content, "old prompt");
    assert_eq!(messages[1].role, "assistant");
    assert_eq!(messages[1].content, "old answer");
    assert_eq!(messages[2].role, "user");
    assert_eq!(messages[2].content, "new prompt");
}

#[test]
fn stream_line_emits_chat_completion_delta() {
    let mut answer = String::new();
    let mut tokens = Vec::new();

    process_chat_completion_stream_line(
        "test provider",
        r#"data: {"choices":[{"delta":{"content":"hello"}}]}"#,
        &mut answer,
        &mut |token| tokens.push(token),
    )
    .expect("stream line should parse");

    assert_eq!(answer, "hello");
    assert_eq!(tokens, vec!["hello"]);
}
