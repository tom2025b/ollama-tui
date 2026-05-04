use serde::Deserialize;

/// Streaming response frame returned as a server-sent event data payload.
#[derive(Debug, Deserialize)]
pub(super) struct ChatCompletionStreamResponse {
    /// Candidate completion deltas.
    pub(super) choices: Vec<ChatCompletionStreamChoice>,
}

/// One streaming choice.
#[derive(Debug, Deserialize)]
pub(super) struct ChatCompletionStreamChoice {
    /// Assistant delta for this frame.
    pub(super) delta: ChatCompletionStreamDelta,
}

/// Assistant message delta.
#[derive(Debug, Deserialize)]
pub(super) struct ChatCompletionStreamDelta {
    /// Text content for this frame, when present.
    pub(super) content: Option<String>,
}
