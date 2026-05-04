use anyhow::{Context, Result};

use super::types::AnthropicStreamEvent;

/// Parse one Anthropic SSE line and emit any text delta.
pub(in crate::anthropic) fn process_anthropic_stream_line<F>(
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
