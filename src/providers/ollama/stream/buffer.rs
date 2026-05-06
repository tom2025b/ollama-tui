use anyhow::Result;

use super::line::process_ollama_stream_line;

/// Process complete newline-delimited JSON records currently in the stream buffer.
pub(in crate::providers::ollama) fn process_ollama_stream_buffer<F>(
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
pub(in crate::providers::ollama) fn process_final_ollama_stream_buffer<F>(
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
