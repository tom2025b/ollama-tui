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
