use anyhow::{Context, Result};

use super::types::ChatStreamChunk;

/// Parse one Ollama streaming JSON line and emit its content delta.
pub(in crate::ollama) fn process_ollama_stream_line<F>(
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
