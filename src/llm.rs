//! Small data types that describe a language model and the backend that serves it.
//!
//! The rest of the app can talk about "models" without needing to know exact
//! HTTP details for Ollama, Anthropic, OpenAI, or xAI.

/// The backend service that knows how to run a model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Provider {
    /// A local Ollama server, usually listening on http://localhost:11434.
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
///
/// Each model has a provider-specific model name, a provider so the app knows
/// how to call it, and a few plain-English strengths that make the UI easier to
/// understand later.
#[derive(Clone, Debug)]
pub struct LanguageModel {
    /// The provider-specific model name.
    ///
    /// For Ollama this must match the model name shown by `ollama list`.
    /// Example: `llama3`, `llama3.1`, or `codellama`.
    pub name: String,

    /// The backend that serves this model.
    pub provider: Provider,

    /// Human-readable notes used by the TUI.
    ///
    /// These do not drive the router. They are just there so you can see why a
    /// model exists in the list.
    pub strengths: Vec<String>,

    /// Whether the router is allowed to choose this model right now.
    ///
    /// Cloud models are disabled when their API key environment variable is not
    /// present. Local Ollama models stay enabled because the Ollama backend
    /// performs the real installed-model check before generation.
    pub enabled: bool,

    /// Short setup note shown when a model is not currently usable.
    pub disabled_reason: Option<String>,
}

impl LanguageModel {
    /// Build a model entry backed by Ollama.
    ///
    /// This helper keeps model declarations short and easy to scan in
    /// `router.rs`.
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

    /// Build a model entry from the individual pieces.
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

/// One completed user/assistant pair used as bounded conversation context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationTurn {
    /// Text originally typed by the user.
    pub user: String,

    /// Assistant answer shown for that user prompt.
    pub assistant: String,
}

/// The router's final decision for a prompt.
///
/// Keeping the model and explanation together makes the app transparent: every
/// answer can show both what was selected and why it was selected.
#[derive(Clone, Debug)]
pub struct RouteDecision {
    /// The model that should answer the prompt.
    pub model: LanguageModel,

    /// Short explanation written for a human, not for another program.
    pub reason: String,
}
