//! Shared provider metadata plus UTF-8 stream decoding helpers used by routing
//! and streaming code.

use anyhow::anyhow;

use crate::Result;

/// One completed user/assistant pair used as bounded conversation context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationTurn {
    /// Text originally typed by the user.
    pub user: String,
    /// Assistant answer shown for that user prompt.
    pub assistant: String,
}

/// A language model the router is allowed to choose.
#[derive(Clone, Debug)]
pub struct LanguageModel {
    /// The provider-specific model name.
    pub name: String,
    /// Human-readable notes used by the TUI.
    pub strengths: Vec<String>,
}

impl LanguageModel {
    /// Build a model entry backed by Ollama.
    pub fn ollama(name: &str, strengths: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            strengths: strengths.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Label used in history and status messages.
    pub fn display_label(&self) -> String {
        format!("Ollama {}", self.name)
    }
}

/// The router's final decision for a prompt.
#[derive(Clone, Debug)]
pub struct RouteDecision {
    /// The model that should answer the prompt.
    pub model: LanguageModel,
    /// Short explanation written for a human.
    pub reason: String,
}

/// Append one raw streaming chunk, buffering any incomplete UTF-8 sequence
/// until enough bytes arrive to decode it safely.
pub(crate) fn append_utf8_chunk(
    source: &'static str,
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
                let decoded = std::str::from_utf8(&pending[..valid_up_to]).map_err(|e| {
                    anyhow!(
                        "{source} stream: internal invariant violated decoding valid UTF-8 prefix at byte {valid_up_to}: {e}"
                    )
                })?;
                output.push_str(decoded);
                pending.drain(..valid_up_to);
            }
        }
        Err(error) => {
            return Err(anyhow!("invalid UTF-8 while {source}: {error}"));
        }
    }

    Ok(())
}

/// Flush any buffered bytes after the provider stream ends.
pub(crate) fn finish_utf8_stream(
    source: &'static str,
    pending: &mut Vec<u8>,
    output: &mut String,
) -> Result<()> {
    if pending.is_empty() {
        return Ok(());
    }

    let decoded = std::str::from_utf8(pending.as_slice()).map_err(|e| {
        anyhow!("{source} streaming error: stream ended mid UTF-8 character: {e}")
    })?;
    output.push_str(decoded);
    pending.clear();
    Ok(())
}
