use super::super::types::chat_messages_from_context;
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
