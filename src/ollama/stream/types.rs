use serde::Deserialize;

/// One JSON line from Ollama's streaming chat endpoint.
#[derive(Debug, Deserialize)]
pub(super) struct ChatStreamChunk {
    /// Assistant message delta for this chunk.
    pub(super) message: Option<OllamaChatResponseMessage>,
    /// True when Ollama has finished the response.
    #[serde(default)]
    #[allow(dead_code)]
    done: bool,
}

/// Assistant message object inside a streaming chat chunk.
#[derive(Debug, Deserialize)]
pub(super) struct OllamaChatResponseMessage {
    /// Delta content for this chunk.
    pub(super) content: String,
}
