use serde::Serialize;

use crate::llm::ConversationTurn;

/// Request body for OpenAI-style chat completions.
#[derive(Debug, Serialize)]
pub(super) struct ChatCompletionRequest {
    /// Model ID, such as `gpt-4o` or `grok-4.20-reasoning`.
    model: String,
    /// Bounded conversation plus the current user prompt.
    messages: Vec<ChatCompletionMessage>,
    /// `true` lets the TUI render text as it arrives.
    stream: bool,
}

impl ChatCompletionRequest {
    pub(super) fn new(model_name: &str, context: &[ConversationTurn], prompt: &str) -> Self {
        Self {
            model: model_name.to_string(),
            messages: chat_messages_from_context(context, prompt),
            stream: true,
        }
    }
}

/// One chat message sent to an OpenAI-compatible backend.
#[derive(Clone, Debug, Serialize)]
pub(super) struct ChatCompletionMessage {
    /// Chat role: `user` or `assistant`.
    pub(super) role: &'static str,
    /// Plain text message content.
    pub(super) content: String,
}

/// Convert bounded conversation context into chat-completions messages.
pub(super) fn chat_messages_from_context(
    context: &[ConversationTurn],
    prompt: &str,
) -> Vec<ChatCompletionMessage> {
    let mut messages = Vec::with_capacity(context.len() * 2 + 1);

    for turn in context {
        messages.push(ChatCompletionMessage {
            role: "user",
            content: turn.user.clone(),
        });
        messages.push(ChatCompletionMessage {
            role: "assistant",
            content: turn.assistant.clone(),
        });
    }

    messages.push(ChatCompletionMessage {
        role: "user",
        content: prompt.to_string(),
    });
    messages
}
