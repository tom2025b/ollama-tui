use anyhow::Result;

use super::line::process_chat_completion_stream_line;

/// Process complete SSE lines currently in the stream buffer.
pub(in crate::providers::openai_compatible) fn process_chat_completion_stream_buffer<F>(
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
pub(in crate::providers::openai_compatible) fn process_final_chat_completion_stream_buffer<F>(
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
