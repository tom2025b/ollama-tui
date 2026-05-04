use serde::Deserialize;

/// One data payload from Anthropic's streaming Messages API.
#[derive(Debug, Deserialize)]
pub(super) struct AnthropicStreamEvent {
    /// Event payload type.
    #[serde(rename = "type")]
    pub(super) event_type: String,
    /// Text delta for `content_block_delta` events.
    pub(super) delta: Option<AnthropicStreamDelta>,
}

/// Delta object inside a streaming event.
#[derive(Debug, Deserialize)]
pub(super) struct AnthropicStreamDelta {
    /// Delta payload type.
    #[serde(rename = "type")]
    pub(super) delta_type: String,
    /// Text chunk for `text_delta` events.
    pub(super) text: Option<String>,
}
