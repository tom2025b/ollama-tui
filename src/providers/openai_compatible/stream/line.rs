use anyhow::{Context, Result};

use super::types::ChatCompletionStreamResponse;

/// Parse one OpenAI-compatible SSE line and emit any content delta.
pub(in crate::providers::openai_compatible) fn process_chat_completion_stream_line<F>(
    provider_name: &'static str,
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
    if data.is_empty() || data == "[DONE]" {
        return Ok(());
    }

    let frame = serde_json::from_str::<ChatCompletionStreamResponse>(data)
        .with_context(|| format!("{provider_name} returned an invalid stream frame: {data}"))?;

    for choice in frame.choices {
        if let Some(content) = choice.delta.content
            && !content.is_empty()
        {
            answer.push_str(&content);
            on_token(content);
        }
    }

    Ok(())
}
