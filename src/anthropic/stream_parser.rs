use anyhow::{Context, Result};
use serde::Deserialize;

/// One data payload from Anthropic's streaming Messages API.
#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    /// Event payload type.
    #[serde(rename = "type")]
    event_type: String,
    /// Text delta for `content_block_delta` events.
    delta: Option<AnthropicStreamDelta>,
}

/// Delta object inside a streaming event.
#[derive(Debug, Deserialize)]
struct AnthropicStreamDelta {
    /// Delta payload type.
    #[serde(rename = "type")]
    delta_type: String,
    /// Text chunk for `text_delta` events.
    text: Option<String>,
}

/// Process complete SSE lines currently in the stream buffer.
pub(super) fn process_anthropic_stream_buffer<F>(
    buffer: &mut String,
    answer: &mut String,
    on_token: &mut F,
) -> Result<()>
where
    F: FnMut(String),
{
    while let Some(newline_index) = buffer.find('\n') {
        let line = buffer.drain(..=newline_index).collect::<String>();
        process_anthropic_stream_line(line.trim(), answer, on_token)?;
    }

    Ok(())
}

/// Process any final unterminated SSE line left after the response ends.
pub(super) fn process_final_anthropic_stream_buffer<F>(
    buffer: &mut String,
    answer: &mut String,
    on_token: &mut F,
) -> Result<()>
where
    F: FnMut(String),
{
    let line = buffer.trim().to_string();
    buffer.clear();

    if !line.is_empty() {
        process_anthropic_stream_line(&line, answer, on_token)?;
    }

    Ok(())
}

/// Parse one Anthropic SSE line and emit any text delta.
pub(super) fn process_anthropic_stream_line<F>(
    line: &str,
    answer: &mut String,
    on_token: &mut F,
) -> Result<()>
where
    F: FnMut(String),
{
    let Some(data) = line.strip_prefix("data:") else {
        return Ok(());
    };
    let data = data.trim();
    if data.is_empty() {
        return Ok(());
    }

    let event = serde_json::from_str::<AnthropicStreamEvent>(data)
        .with_context(|| format!("Anthropic returned an invalid stream event: {data}"))?;
    if event.event_type != "content_block_delta" {
        return Ok(());
    }
    let Some(delta) = event.delta else {
        return Ok(());
    };
    if delta.delta_type != "text_delta" {
        return Ok(());
    }
    if let Some(text) = delta.text
        && !text.is_empty()
    {
        answer.push_str(&text);
        on_token(text);
    }

    Ok(())
}
