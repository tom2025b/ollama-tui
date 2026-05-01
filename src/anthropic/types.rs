use serde::Serialize;

use crate::llm::ConversationTurn;

/// Request body for Anthropic's Messages API.
#[derive(Debug, Serialize)]
pub(super) struct AnthropicRequest {
    /// Claude model ID.
    model: String,
    /// Maximum number of tokens Claude may generate.
    max_tokens: u32,
    /// Bounded conversation plus the current user prompt.
    messages: Vec<AnthropicMessage>,
    /// Whether Anthropic should return server-sent events.
    stream: bool,
}

impl AnthropicRequest {
    pub(super) fn new(
        model_name: &str,
        context: &[ConversationTurn],
        prompt: &str,
        max_tokens: u32,
    ) -> Self {
        Self {
            model: model_name.to_string(),
            max_tokens,
            messages: anthropic_messages_from_context(context, prompt),
            stream: true,
        }
    }
}

/// One message sent to Claude.
#[derive(Clone, Debug, Serialize)]
pub(super) struct AnthropicMessage {
    /// Chat role: `user` or `assistant`.
    pub(super) role: &'static str,
    /// Plain text message content.
    pub(super) content: String,
}

/// Convert bounded conversation context into Anthropic messages.
pub(super) fn anthropic_messages_from_context(
    context: &[ConversationTurn],
    prompt: &str,
) -> Vec<AnthropicMessage> {
    let mut messages = Vec::with_capacity(context.len() * 2 + 1);

    for turn in context {
        messages.push(AnthropicMessage {
            role: "user",
            content: turn.user.clone(),
        });
        messages.push(AnthropicMessage {
            role: "assistant",
            content: turn.assistant.clone(),
        });
    }

    messages.push(AnthropicMessage {
        role: "user",
        content: prompt.to_string(),
    });
    messages
}
