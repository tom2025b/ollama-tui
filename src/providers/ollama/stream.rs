use anyhow::{Context, Result};
use serde::Deserialize;

/// Process complete newline-delimited JSON records currently in the stream buffer.
pub(super) fn process_ollama_stream_buffer<F>(
    buffer: &mut String,
    answer: &mut String,
    on_token: &mut F,
) -> Result<()>
where
    F: FnMut(String),
{
    while let Some(newline_index) = buffer.find('\n') {
        let line = buffer.drain(..=newline_index).collect::<String>();
        process_ollama_stream_line(line.trim(), answer, on_token)?;
    }

    Ok(())
}

/// Process any final unterminated JSON record left after the response ends.
pub(super) fn process_final_ollama_stream_buffer<F>(
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
        process_ollama_stream_line(&line, answer, on_token)?;
    }

    Ok(())
}

/// Parse one Ollama streaming JSON line and emit its content delta.
pub(super) fn process_ollama_stream_line<F>(
    line: &str,
    answer: &mut String,
    on_token: &mut F,
) -> Result<()>
where
    F: FnMut(String),
{
    if line.is_empty() {
        return Ok(());
    }

    let chunk = serde_json::from_str::<ChatStreamChunk>(line)
        .with_context(|| format!("Ollama returned an invalid stream line: {line}"))?;

    if let Some(message) = chunk.message
        && !message.content.is_empty()
    {
        answer.push_str(&message.content);
        on_token(message.content);
    }

    Ok(())
}

/// One JSON line from Ollama's streaming chat endpoint.
#[derive(Debug, Deserialize)]
struct ChatStreamChunk {
    /// Assistant message delta for this chunk.
    message: Option<OllamaChatResponseMessage>,
}

/// Assistant message object inside a streaming chat chunk.
#[derive(Debug, Deserialize)]
struct OllamaChatResponseMessage {
    /// Delta content for this chunk.
    content: String,
}
