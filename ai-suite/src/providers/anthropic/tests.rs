use super::config::DEFAULT_ANTHROPIC_MODEL;
use super::stream;
use super::stream_parser::process_anthropic_stream_line;
use super::types::anthropic_messages_from_context;
use crate::llm::ConversationTurn;

#[test]
fn anthropic_messages_include_bounded_context_then_current_prompt() {
    let context = vec![ConversationTurn {
        user: "old prompt".to_string(),
        assistant: "old answer".to_string(),
    }];

    let messages = anthropic_messages_from_context(&context, "new prompt");

    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[0].content, "old prompt");
    assert_eq!(messages[1].role, "assistant");
    assert_eq!(messages[1].content, "old answer");
    assert_eq!(messages[2].role, "user");
    assert_eq!(messages[2].content, "new prompt");
}

#[test]
fn stream_line_emits_anthropic_text_delta() {
    let mut answer = String::new();
    let mut tokens = Vec::new();

    process_anthropic_stream_line(
        r#"data: {"type":"content_block_delta","delta":{"type":"text_delta","text":"hello"}}"#,
        &mut answer,
        &mut |token| tokens.push(token),
    )
    .expect("stream line should parse");

    assert_eq!(answer, "hello");
    assert_eq!(tokens, vec!["hello"]);
}

#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY and makes a live Anthropic API call"]
async fn live_anthropic_stream_smoke_test() {
    let answer = stream(
        DEFAULT_ANTHROPIC_MODEL,
        &[],
        "Reply with one short sentence confirming Claude is working.",
        |_| {},
    )
    .await
    .expect("Anthropic streaming should work");

    assert!(!answer.trim().is_empty());
}
