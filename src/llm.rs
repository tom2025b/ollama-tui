mod model;
mod provider;
mod route;
mod turn;

use anyhow::{Context, Result, bail};

pub use model::LanguageModel;
pub use provider::Provider;
pub use route::RouteDecision;
pub use turn::ConversationTurn;

pub(crate) fn append_utf8_chunk(
    source: &str,
    pending: &mut Vec<u8>,
    output: &mut String,
    chunk: &[u8],
) -> Result<()> {
    pending.extend_from_slice(chunk);

    match std::str::from_utf8(pending.as_slice()) {
        Ok(decoded) => {
            output.push_str(decoded);
            pending.clear();
        }
        Err(error) if error.error_len().is_none() => {
            let valid_up_to = error.valid_up_to();
            if valid_up_to > 0 {
                let decoded = std::str::from_utf8(&pending[..valid_up_to])
                    .expect("valid_up_to marks a valid UTF-8 prefix");
                output.push_str(decoded);
                pending.drain(..valid_up_to);
            }
        }
        Err(error) => bail!("{source} stream returned invalid UTF-8: {error}"),
    }

    Ok(())
}

pub(crate) fn finish_utf8_stream(
    source: &str,
    pending: &mut Vec<u8>,
    output: &mut String,
) -> Result<()> {
    if pending.is_empty() {
        return Ok(());
    }

    let decoded = std::str::from_utf8(pending.as_slice())
        .with_context(|| format!("{source} stream ended mid UTF-8 character"))?;
    output.push_str(decoded);
    pending.clear();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{append_utf8_chunk, finish_utf8_stream};

    #[test]
    fn utf8_chunk_decoder_preserves_split_codepoint() {
        let mut pending = Vec::new();
        let mut output = String::new();

        append_utf8_chunk("test", &mut pending, &mut output, b"hi \xf0\x9f").unwrap();
        append_utf8_chunk("test", &mut pending, &mut output, b"\x98\x80").unwrap();
        finish_utf8_stream("test", &mut pending, &mut output).unwrap();

        assert_eq!(output, "hi \u{1f600}");
    }
}
