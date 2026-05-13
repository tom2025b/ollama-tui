use serde::Deserialize;

use crate::{Error, Result};

/// Process complete SSE lines currently in the stream buffer.
pub(super) fn process_chat_completion_stream_buffer<F>(
    provider_name: &'static str,
    buffer: &mut String,
    answer: &mut String,
    on_token: &mut F,
) -> Result<()>
where
    F: FnMut(String),
{
    while let Some(newline_index) = buffer.find('\n') {
        let line = buffer.drain(..=newline_index).collect::<String>();
        process_chat_completion_stream_line(provider_name, line.trim(), answer, on_token)?;
    }

    Ok(())
}

/// Process a final unterminated SSE line left after the stream ends.
pub(super) fn process_final_chat_completion_stream_buffer<F>(
    provider_name: &'static str,
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
        process_chat_completion_stream_line(provider_name, &line, answer, on_token)?;
    }

    Ok(())
}

/// Parse one OpenAI-compatible SSE line and emit any content delta.
#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn process_chat_completion_stream_line<F>(
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

    let frame = serde_json::from_str::<ChatCompletionStreamResponse>(data).map_err(|source| {
        Error::provider_response(
            provider_name,
            format!("invalid stream frame: {source}. Frame: {data}"),
        )
    })?;

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

/// Streaming response frame returned as a server-sent event data payload.
#[derive(Debug, Deserialize)]
struct ChatCompletionStreamResponse {
    /// Candidate completion deltas.
    choices: Vec<ChatCompletionStreamChoice>,
}

/// One streaming choice.
#[derive(Debug, Deserialize)]
struct ChatCompletionStreamChoice {
    /// Assistant delta for this frame.
    delta: ChatCompletionStreamDelta,
}

/// Assistant message delta.
#[derive(Debug, Deserialize)]
struct ChatCompletionStreamDelta {
    /// Text content for this frame, when present.
    content: Option<String>,
}
