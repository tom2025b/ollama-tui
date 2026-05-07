use anyhow::{Context, Result, bail};

/// One completed user/assistant pair used as bounded conversation context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationTurn {
    /// Text originally typed by the user.
    pub user: String,
    /// Assistant answer shown for that user prompt.
    pub assistant: String,
}

/// The backend service that knows how to run a model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Provider {
    /// A local Ollama server.
    Ollama,
    /// Anthropic's Claude API.
    Anthropic,
    /// OpenAI's API.
    OpenAi,
    /// xAI's Grok API.
    Xai,
}

impl Provider {
    /// Human-readable provider name for status messages and the TUI.
    pub fn label(&self) -> &'static str {
        match self {
            Provider::Ollama => "Ollama",
            Provider::Anthropic => "Anthropic",
            Provider::OpenAi => "OpenAI",
            Provider::Xai => "xAI",
        }
    }
}

/// A language model the router is allowed to choose.
#[derive(Clone, Debug)]
pub struct LanguageModel {
    /// The provider-specific model name.
    pub name: String,
    /// The backend that serves this model.
    pub provider: Provider,
    /// Human-readable notes used by the TUI.
    pub strengths: Vec<String>,
    /// Whether the router is allowed to choose this model right now.
    pub enabled: bool,
    /// Short setup note shown when a model is not currently usable.
    pub disabled_reason: Option<String>,
}

impl LanguageModel {
    /// Build a model entry backed by Ollama.
    pub fn ollama(name: &str, strengths: &[&str]) -> Self {
        Self::new(Provider::Ollama, name, strengths, true, None)
    }

    /// Build a Claude model entry.
    pub fn anthropic(
        name: &str,
        strengths: &[&str],
        enabled: bool,
        disabled_reason: Option<String>,
    ) -> Self {
        Self::new(
            Provider::Anthropic,
            name,
            strengths,
            enabled,
            disabled_reason,
        )
    }

    /// Build an OpenAI model entry.
    pub fn openai(
        name: &str,
        strengths: &[&str],
        enabled: bool,
        disabled_reason: Option<String>,
    ) -> Self {
        Self::new(Provider::OpenAi, name, strengths, enabled, disabled_reason)
    }

    /// Build an xAI model entry.
    pub fn xai(
        name: &str,
        strengths: &[&str],
        enabled: bool,
        disabled_reason: Option<String>,
    ) -> Self {
        Self::new(Provider::Xai, name, strengths, enabled, disabled_reason)
    }

    fn new(
        provider: Provider,
        name: &str,
        strengths: &[&str],
        enabled: bool,
        disabled_reason: Option<String>,
    ) -> Self {
        Self {
            name: name.to_string(),
            provider,
            strengths: strengths
                .iter()
                .map(|strength| strength.to_string())
                .collect(),
            enabled,
            disabled_reason,
        }
    }

    /// Label used in history and status messages.
    pub fn display_label(&self) -> String {
        format!("{} {}", self.provider.label(), self.name)
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
                // SAFETY: `Utf8Error::valid_up_to` is documented to return the
                // number of leading bytes that form valid UTF-8, so slicing
                // `pending` at that boundary always yields a valid UTF-8
                // prefix. The `expect` only fires if the standard library
                // contract is violated, which is a programmer error in `std`,
                // not a runtime condition we should try to recover from.
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
