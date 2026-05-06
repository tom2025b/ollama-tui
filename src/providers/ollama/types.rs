use serde::Serialize;

use crate::llm::ConversationTurn;

/// Request body for Ollama's `/api/chat` endpoint.
#[derive(Debug, Serialize)]
pub(super) struct ChatRequest {
    /// Ollama model name, such as `llama3`.
    model: String,
    /// Bounded conversation plus the current user prompt.
    messages: Vec<OllamaChatMessage>,
    /// `true` lets the TUI render text as it arrives.
    stream: bool,
}

impl ChatRequest {
    pub(super) fn new(model_name: &str, context: &[ConversationTurn], prompt: &str) -> Self {
        Self {
            model: model_name.to_string(),
            messages: chat_messages_from_context(context, prompt),
            stream: true,
        }
    }
}

/// One chat message sent to Ollama.
#[derive(Clone, Debug, Serialize)]
pub(super) struct OllamaChatMessage {
    /// Chat role: `user` or `assistant`.
    pub(super) role: &'static str,
    /// Plain text message content.
    pub(super) content: String,
}

/// Convert bounded conversation context into Ollama chat messages.
pub(super) fn chat_messages_from_context(
    context: &[ConversationTurn],
    prompt: &str,
) -> Vec<OllamaChatMessage> {
    let mut messages = Vec::with_capacity(context.len() * 2 + 1);

    for turn in context {
        messages.push(OllamaChatMessage {
            role: "user",
            content: turn.user.clone(),
        });
        messages.push(OllamaChatMessage {
            role: "assistant",
            content: turn.assistant.clone(),
        });
    }

    messages.push(OllamaChatMessage {
        role: "user",
        content: prompt.to_string(),
    });
    messages
}
